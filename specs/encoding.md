# noun encoding specification

version: 0.3
status: canonical

## overview

this document specifies three encoding layers for nox nouns:

1. **storage encoding** — canonical binary serialization of individual nouns
2. **content-addressed identity** — how noun identity is derived from encoded content
3. **wire format** — how nouns are transmitted between nodes

all concrete sizes refer to the canonical instantiation: nox<Goldilocks, Z/2^32, Hemera>.

## storage encoding

every noun is either an atom or a cell. the first byte is a tag that determines interpretation. the encoding is deterministic — one valid byte sequence per noun.

### tag byte

```
0x00   atom (field element)
0x01   atom (word element)
0x02   atom (hash element)
0x03   cell
```

three atom tags distinguish runtime types. the tag byte makes stored nouns self-describing — a resolver does not need execution context to parse a noun.

### atom: field element (tag 0x00)

```
offset  size   field
──────  ─────  ──────────────────────────────
0       1      tag = 0x00
1       8      value: canonical little-endian Goldilocks element

total: 9 bytes
```

the 8-byte value MUST be in [0, p) where p = 2^64 - 2^32 + 1 = 18446744069414584321. values >= p are invalid. the canonical encoding is the unique little-endian representation of the reduced value.

### atom: word element (tag 0x01)

```
offset  size   field
──────  ─────  ──────────────────────────────
0       1      tag = 0x01
1       8      value: canonical little-endian Goldilocks element
                (word value zero-extended to field element, MUST be in [0, 2^32))

total: 9 bytes
```

word atoms store 32-bit values as field elements. the value MUST be in [0, 2^32). values >= 2^32 are invalid for word-typed atoms. the encoding is 8 bytes little-endian, same as field atoms — the upper 4 bytes are zero.

### atom: hash element (tag 0x02)

```
offset  size   field
──────  ─────  ──────────────────────────────
0       1      tag = 0x02
1       32     value: 4 consecutive Goldilocks elements, each 8 bytes little-endian

total: 33 bytes
```

each of the 4 field elements MUST be in [0, p). this encodes the output of a hemera hash (4 x F_p).

### cell (tag 0x03)

```
offset  size   field
──────  ─────  ──────────────────────────────
0       1      tag = 0x03
1       32     left:  NounId (32 bytes, identity of the left child)
33      32     right: NounId (32 bytes, identity of the right child)

total: 65 bytes
```

left before right. a cell does not contain its children inline — it contains their identities. to materialize a cell, resolve both NounIds recursively.

### encoding summary

```
tag    type          size (bytes)   payload
────   ────────────  ─────────────  ────────────────────────────────
0x00   field atom     9             8-byte LE Goldilocks element
0x01   word atom      9             8-byte LE value (must be < 2^32)
0x02   hash atom     33             4 × 8-byte LE Goldilocks elements
0x03   cell          65             NounId(left) ‖ NounId(right)
```

four sizes: 9, 9, 33, 65. tag byte disambiguates the two 9-byte variants.

## NounId — content-addressed identity

every noun has a 32-byte identity derived from its content:

```
NounId = hemera(encoded_noun)     32 bytes (4 × F_p elements)
```

`encoded_noun` is the canonical binary encoding defined above (tag byte + payload). hemera absorbs the byte sequence and squeezes 4 field elements. the identity is deterministic — same noun always produces the same NounId.

### identity computation

```
fn noun_id(noun: &Noun) -> NounId {
    let bytes = encode(noun);          // tag byte + payload
    hemera::hash(&bytes)               // → [F_p; 4] = 32 bytes
}
```

hemera uses the default domain tag (capacity[11] = 0x00) for structural noun hashing. other domain tags (COMMITMENT, NULLIFIER, etc.) are protocol-level — they do not affect NounId.

### polynomial identity (optimization)

for large nouns (many leaves), the polynomial identity is more efficient:

```
identity = hemera(Lens.commit(noun_polynomial) ‖ domain_tag)    32 bytes
```

the noun is encoded as a multilinear polynomial (see nouns.md polynomial representation). Lens commitment costs O(N) field operations where N = number of leaves (Brakedown linear-time commitment). the hemera wrap adds 1 hemera call for domain separation.

both methods produce the same NounId for the same noun. the polynomial path is an optimization for nouns with many leaves — it replaces recursive hashing with O(N) field ops + 1 hemera call. for small nouns (< 56 bytes), cost is equivalent to direct hemera hashing.

an implementation MUST support the direct path (hemera of encoded bytes). the polynomial path is optional.

### computation key

the memoization key for a computation (formula applied to object):

```
computation_key = NounId(object) ‖ NounId(formula)    64 bytes
computation_val = NounId(result)                       32 bytes
```

this is the axon in the cybergraph (see reduction.md global memoization).

## content-addressed store

nouns are stored in a content-addressed key-value store:

```
store: NounId (32 bytes) → encoded_noun (9, 9, 33, or 65 bytes)
```

to retrieve a noun, query the store by its NounId. the tag byte in the stored value determines how to parse the payload.

### resolution

to materialize a full noun tree from a NounId:

```
fn resolve(store: &Store, id: NounId) -> Result<Noun, Unavailable> {
    let bytes = store.get(id)?;
    match bytes[0] {
        0x00 => {
            assert(bytes.len() == 9);
            let value = u64_le(bytes[1..9]);
            assert(value < p);
            Atom::Field(value)
        }
        0x01 => {
            assert(bytes.len() == 9);
            let value = u64_le(bytes[1..9]);
            assert(value < p);
            assert(value < 2^32);
            Atom::Word(value as u32)
        }
        0x02 => {
            assert(bytes.len() == 33);
            let elems = [u64_le(bytes[1..9]),
                         u64_le(bytes[9..17]),
                         u64_le(bytes[17..25]),
                         u64_le(bytes[25..33])];
            assert(all elems < p);
            Atom::Hash(elems)
        }
        0x03 => {
            assert(bytes.len() == 65);
            let left_id  = bytes[1..33];
            let right_id = bytes[33..65];
            let left  = resolve(store, left_id)?;
            let right = resolve(store, right_id)?;
            Cell(left, right)
        }
        _ => error("invalid tag byte")
    }
}
```

resolution is recursive for cells. an implementation SHOULD use iterative deepening or a stack to avoid call-stack overflow on deep nouns.

### store verification

when receiving nouns from an untrusted source, verify:

```
assert(hemera::hash(encoded_noun) == claimed_noun_id);
```

this is the only check needed. if the hash matches, the content is authentic. content addressing eliminates the need for signatures on individual nouns.

## wire format

nouns are transmitted between nodes as length-prefixed messages containing content-addressed noun entries.

### message framing

```
offset  size       field
──────  ─────────  ──────────────────────────────
0       4          message_length: u32 little-endian (byte count of payload)
4       variable   payload: one or more noun entries

max message_length: 2^24 (16 MiB). implementations MUST reject messages > 16 MiB.
```

### noun entry within a message

each entry in the payload is:

```
offset  size       field
──────  ─────────  ──────────────────────────────
0       32         NounId (expected identity)
32      1          entry_length: encoded noun size (9, 9, 33, or 65)
33      variable   encoded_noun (tag byte + payload, as specified above)
```

the receiver:
1. reads the NounId (32 bytes)
2. reads the entry_length (1 byte)
3. reads entry_length bytes of encoded noun data
4. verifies: hemera(encoded_noun) == NounId
5. stores the entry if valid, rejects the message if any entry fails verification

### message types

```
type byte (first byte of payload):
0x10   noun_push    — sender pushes noun entries (no request)
0x11   noun_request — request nouns by NounId list
0x12   noun_response — response to a noun_request

noun_push payload:
  [type=0x10] [entry_count: u32 LE] [entry_0] [entry_1] ...

noun_request payload:
  [type=0x11] [count: u32 LE] [NounId_0] [NounId_1] ...

noun_response payload:
  [type=0x12] [entry_count: u32 LE] [entry_0] [entry_1] ...
```

a noun_push for a cell MUST include all transitive children before the cell itself (topological order). the receiver can verify and store each entry as it arrives — no forward references.

### particle CID

a particle in the cybergraph is identified by its NounId:

```
particle_cid = NounId(noun) = hemera(encode(noun))
```

the CID (content identifier) is the hemera hash of the canonical encoding. any node can independently compute the CID from the noun content. two nodes with the same noun always agree on its CID.

## variable-length atom encoding (optional optimization)

for atoms whose value fits in fewer than 8 bytes, an implementation MAY use a compact encoding within internal storage. this is an implementation optimization — the canonical wire format and identity computation always use the fixed-width encoding defined above.

```
compact atom encoding (internal use only):

tag     size   value range         encoding
─────   ─────  ──────────────────  ─────────────────────
0x80    2      [0, 256)            tag + 1 byte
0x81    3      [0, 65536)          tag + 2 bytes LE
0x82    5      [0, 2^32)           tag + 4 bytes LE
0x00    9      [0, p)              tag + 8 bytes LE (canonical)
```

the compact encoding MUST NOT be used for identity computation or wire transmission. it is strictly an internal storage optimization for implementations that want to reduce memory for small atoms.

an implementation that does not support compact encoding is fully conforming.

## field element encoding

```
Goldilocks field element:
  8 bytes, little-endian
  canonical value in [0, p) where p = 2^64 - 2^32 + 1
  values >= p are invalid
  reduction: a mod p = a_lo - a_hi × (2^32 - 1) + correction
```

## formula encoding

a formula is a noun of the form `cell(tag, body)` where tag is a field atom with value 0-17 (the pattern number).

```
[0 a]           axis     — navigate object tree
[1 c]           quote    — return literal
[2 [x y]]       compose  — reduce x, reduce y, apply
[3 [a b]]       cons     — construct cell
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
[17 a]          look     — deterministic BBG state read
```

formulas are nouns. they are stored and resolved the same way as any other noun — by NounId lookup, recursively. the tag atom uses field type (0x00) in the encoding.

## canonical invariants

1. tag byte MUST be 0x00, 0x01, 0x02, or 0x03
2. field element values MUST be in [0, p)
3. word element values MUST be in [0, 2^32)
4. hash elements: each of the 4 field elements MUST be in [0, p)
5. cell content: left NounId before right NounId
6. encoded noun length MUST match the tag (9, 9, 33, or 65 bytes)
7. NounId MUST equal hemera(encoded_noun) — no aliasing
8. one valid encoding per noun — no alternative representations
9. wire messages: topological order (children before parents)
10. wire messages MUST NOT exceed 16 MiB

violation of any invariant is a rejection. there is no recovery, no fallback, no "best effort" parsing. invalid data is discarded.

## test vectors (canonical instantiation)

### atom encoding

```
field atom with value 0:
  encoded: 00 00 00 00 00 00 00 00 00
  NounId:  hemera(00 00 00 00 00 00 00 00 00)

field atom with value 1:
  encoded: 00 01 00 00 00 00 00 00 00
  NounId:  hemera(00 01 00 00 00 00 00 00 00)

field atom with value p-1 (max valid):
  p-1 = 18446744069414584320 = 0xFFFFFFFF00000000
  little-endian of p-1: 00 00 00 00 FF FF FF FF
  encoded: 00 00 00 00 00 FF FF FF FF   (tag=0x00, then 8 bytes LE)
  NounId:  hemera(00 00 00 00 00 FF FF FF FF)

word atom with value 42:
  encoded: 01 2A 00 00 00 00 00 00 00
  NounId:  hemera(01 2A 00 00 00 00 00 00 00)
```

### cell encoding

```
cell(atom(0), atom(1)):
  left  = NounId(atom(0))  = hemera(00 00 00 00 00 00 00 00 00)
  right = NounId(atom(1))  = hemera(00 01 00 00 00 00 00 00 00)
  encoded: 03 ‖ left ‖ right  (65 bytes)
  NounId:  hemera(03 ‖ left ‖ right)
```

### invalid encodings (must be rejected)

```
tag byte 0x04 or higher           — unknown tag
field atom with value = p          — value out of range
field atom with value = p + 1      — value out of range
word atom with value = 2^32        — word out of range
cell with only 64 bytes total      — truncated
cell with 66 bytes total           — trailing bytes
hash atom with 3 elements          — truncated
empty payload (0 bytes)            — missing tag
```
