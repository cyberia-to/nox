# data encoding specification

version: 0.5
status: canonical

## overview

nox uses **Model B**: data *is* its field leaves. there is no tag byte and no
separate "encoding" identity scheme — a data node's `particle` is the tree hash
of its content (`data/hash.md`), the same bytes the in-order hash-cons table
keys on. this document specifies only the byte layout for storage and wire
transmission; identity lives in `data/hash.md`.

all concrete sizes refer to the canonical instantiation: nox<Goldilocks, Z/2^32, Hemera>.

## storage layout

every data node is an atom or a pair. node type is read from **length**, not a
tag byte:

```
atom   8 bytes    one field, little-endian
pair  64 bytes    left particle (32) ‖ right particle (32)
```

- **atom** — the 8-byte little-endian Goldilocks value. MUST be in `[0, p)`
  where `p = 2^64 - 2^32 + 1 = 18446744069414584321`. there is no `word` or
  `hash` atom variant: a `word` is an atom whose value is proven in `[0, 2^32)`
  (a refinement, not an encoding), and a hemera hash result is a pair-tree of
  atoms, not a leaf.
- **pair** — its two children's `particle`s, left before right. a pair does not
  inline its children; it references them by identity. to materialize a pair,
  resolve both particles recursively.

data is sized `8·N` for `N` leaves. the size spectrum is the full ladder — `8`
(atom), `16` would be a flattened pair-of-atoms commitment, `32` (a particle),
`64` (a pair of particles), … — and a particle is 4 field leaves, not a tree of
tagged nodes.

## particle — content-addressed identity

a node's particle is its tree hash (full definition in `data/hash.md`):

```
particle(atom v)        = hash_atom(v)               // hemera leaf hash of 8 bytes
particle(pair l r)      = hash_pair(p(l), p(r))      // hemera node hash of two particles
```

both yield 32 bytes (4 Goldilocks limbs, little-endian). identity is the **same**
computation as the in-order hash-cons key — one hash, no second scheme. the tag
byte that older drafts prepended is gone: `field 5` and `word 5` have the same
particle (identity reflects content, ignores refinement); `pair(domain, value)`
gives nominal distinctness through content, never a tag.

### equivalence to the leaf commitment (optional)

for data with many leaves, `particle = hemera(lens.commit(leaves))` is an
equivalent O(N) path (Brakedown linear-time commitment + one hemera wrap for
domain separation). it produces the same particle as the recursive tree hash for
the same data. an implementation MUST support the direct tree-hash path; the
commitment path is an optimization.

### computation key (axon)

the memoization key for a computation (formula applied to object):

```
computation_key = particle(object) ‖ particle(formula)    64 bytes
computation_val = particle(result)                         32 bytes
```

this is the axon in the cybergraph (see `reduction.md` global memoization).

## content-addressed store

```
store: Particle (32 bytes) → encoded_data (8 or 64 bytes)
```

to retrieve a node, query by its particle and read the payload. length
disambiguates:

```
fn resolve(store: &Store, id: Particle) -> Result<Data, Unavailable> {
    let bytes = store.get(id)?;
    match bytes.len() {
        8 => {
            let value = u64_le(bytes);
            assert(value < p);
            Atom(value)
        }
        64 => {
            let left  = bytes[0..32];
            let right = bytes[32..64];
            Pair(resolve(store, left)?, resolve(store, right)?)
        }
        _ => error("invalid length")
    }
}
```

resolution is recursive for pairs. an implementation SHOULD use an explicit stack
to avoid call-stack overflow on deep data.

### store verification

when receiving a node from an untrusted source, recompute its tree hash and
compare:

```
assert(particle(decoded_bytes) == claimed_particle);
```

this is the only check needed — content addressing eliminates per-node
signatures.

## wire format

data is transmitted as length-prefixed messages containing content-addressed
entries.

### message framing

```
offset  size       field
──────  ─────────  ──────────────────────────────
0       4          message_length: u32 little-endian (byte count of payload)
4       variable   payload: one or more entries

max message_length: 2^24 (16 MiB). implementations MUST reject larger messages.
```

### entry within a message

```
offset  size       field
──────  ─────────  ──────────────────────────────
0       32         Particle (expected identity)
32      1          entry_length: 8 (atom) or 64 (pair)
33      variable   encoded_data (8 or 64 bytes, as above)
```

the receiver:
1. reads the Particle (32 bytes)
2. reads the entry_length (1 byte; MUST be 8 or 64)
3. reads entry_length bytes
4. verifies: `particle(encoded_data) == Particle`
5. stores the entry if valid; rejects the whole message if any entry fails

### message types

```
type byte (first byte of payload):
0x10   data_push     — sender pushes entries (no request)
0x11   data_request  — request data by particle list
0x12   data_response — response to a data_request

data_push     payload: [0x10] [entry_count: u32 LE] [entry_0] [entry_1] ...
data_request  payload: [0x11] [count: u32 LE]       [Particle_0] ...
data_response payload: [0x12] [entry_count: u32 LE] [entry_0] [entry_1] ...
```

a data_push for a pair MUST include all transitive children before the pair
itself (topological order). the receiver verifies and stores each entry as it
arrives — no forward references.

transport-level framing (length varints, batching) is tape's concern, never
identity.

## formula encoding

a formula is a pair `pair(tag, body)` where tag is an atom with value 0–17 (the
pattern number).

```
[0 a]           axis     — navigate object tree
[1 c]           quote    — return literal
[2 [x y]]       compose  — reduce x, reduce y, apply
[3 [a b]]       cons     — construct pair
[4 [t [y n]]]   branch   — conditional evaluation
[5 [a b]]       add      — field addition
[6 [a b]]       sub      — field subtraction
[7 [a b]]       mul      — field multiplication
[8 a]           inv      — field inversion (Fermat)
[9 [a b]]       eq       — equality test
[10 [a b]]      lt       — less-than test
[11 [a b]]      xor      — bitwise exclusive-or
[12 [a b]]      and      — bitwise conjunction
[13 a]          not      — bitwise complement
[14 [a n]]      shl      — bitwise left shift
[15 a]          hash     — hemera hash
[16 [t c]]      call     — non-deterministic witness injection
[17 [n k]]      look     — deterministic BBG state read (namespace, key)
```

formulas are data — stored and resolved like any other node, by particle lookup.
the tag is an ordinary 8-byte atom.

## field element encoding

```
Goldilocks field element:
  8 bytes, little-endian
  canonical value in [0, p) where p = 2^64 - 2^32 + 1
  values >= p are invalid
  reduction: a mod p = a_lo - a_hi × (2^32 - 1) + correction
```

## canonical invariants

1. node payload is 8 bytes (atom) or 64 bytes (pair) — no tag byte; type is read from length
2. atom value MUST be in `[0, p)`
3. a `word` is an atom whose value is in `[0, 2^32)` — a refinement, not a distinct encoding
4. pair content: left particle before right particle
5. particle MUST equal the tree hash of the content — no aliasing, no second scheme
6. one canonical layout per node — no alternative representations
7. wire messages: topological order (children before parents)
8. wire messages MUST NOT exceed 16 MiB

violation of any invariant is a rejection. there is no recovery, no fallback, no
"best effort" parsing. invalid data is discarded.

## test vectors (canonical instantiation)

### atom layout

```
atom value 0:
  bytes:    00 00 00 00 00 00 00 00          (8 bytes, LE)
  particle: hash_atom(0)

atom value 1:
  bytes:    01 00 00 00 00 00 00 00
  particle: hash_atom(1)

atom value 42 (a word-range value — same as the field 42):
  bytes:    2A 00 00 00 00 00 00 00
  particle: hash_atom(42)

atom value p-1 (max valid):
  p-1 = 18446744069414584320 = 0xFFFFFFFF00000000
  bytes:    00 00 00 00 FF FF FF FF
  particle: hash_atom(p-1)
```

### pair layout

```
pair(atom(0), atom(1)):
  left  = particle(atom(0)) = hash_atom(0)
  right = particle(atom(1)) = hash_atom(1)
  bytes:    left ‖ right                     (64 bytes)
  particle: hash_pair(left, right)
```

### invalid layouts (must be rejected)

```
9-byte payload                    — invalid length (legacy tagged atom, no longer accepted)
65-byte payload                   — invalid length (legacy tagged pair)
33-byte payload                   — invalid length (legacy hash atom)
atom with value = p               — value out of range
atom with value = p + 1           — value out of range
pair with only 63 bytes           — invalid length
pair with 65 bytes                — invalid length
empty payload (0 bytes)           — invalid length
```
