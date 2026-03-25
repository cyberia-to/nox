# noun specification

version: 0.2
status: canonical

## overview

everything in nox is a noun. a noun is either an atom or a cell. there is nothing else.

```
noun = atom(F)
     | cell(noun, noun)
```

atom: single element of the instantiated field F.
cell: ordered pair of two nouns — a binary tree.

a program is a noun. an object is a noun. the result is a noun. a cyberlink is a noun. a stark proof serialized for verification is a noun. one structure for everything.

the noun model is parameterized by the field F. in the canonical instantiation (nox<Goldilocks, Z/2^32, Hemera>), F = F_p where p = 2^64 - 2^32 + 1. see vm.md for the instantiation model.

## type tags

atoms carry a type tag distinguishing uses of the underlying field element. the value tower is per-instantiation — the number of type tags, their ranges, and their semantics depend on the field F and word width W.

### canonical value tower (nox<Goldilocks, Z/2^32, Hemera>)

```
┌───────────────────────────────────────────────────────────────────────┐
│  TYPE TAG    │  REPRESENTATION     │  VALID RANGE    │  USE          │
├──────────────┼─────────────────────┼─────────────────┼───────────────┤
│  0x00: field │  Single F_p element │  [0, p)         │  Arithmetic   │
│  0x01: word  │  Single F_p element │  [0, 2^32)      │  Bitwise      │
│  0x02: hash  │  4 × F_p elements   │  32-byte digest │  Identity     │
└───────────────────────────────────────────────────────────────────────┘
```

field and word share the same representation (one Goldilocks element) but different operations. a field element wraps around modulo p; a word wraps around modulo 2^32. the distinction is semantic, enforced by the type system. the 32-bit word range guarantees every word value is a valid field element ([0, 2^32) ⊂ [0, p)), and every bitwise operation produces a representable result. heavy 64-bit binary computation belongs in Bt (FRI-Binius, characteristic 2), not in nox's prime field bitwise patterns.

the hash type (four field elements, 32 bytes) is the identity primitive. `H(noun)` produces a hash. `axis(s, 0)` returns `H(s)` — a noun can introspect its own identity.

the type tag costs nothing in the stark — it is a constraint selector, not runtime data.

### value tower across instantiations

the three-type tower (field, word, hash) is specific to the Goldilocks instantiation. in other instantiations the tower adapts:

```
nox<F₂, Z/2^1, Grøstl>:     atom = 1 bit, word = 1 bit (field = word in char 2)
nox<F_{p³}, Z/2^32, Hemera>: atom = 3 × F_p, word = [0, 2^32), hash = 4 × F_p
```

whether the three-type value tower generalizes cleanly across all fields is an open question. what is invariant: the distinction between field operations (patterns 5-10), bitwise operations (patterns 11-14), and hash (pattern 15) — these map to algebraically distinct domains in every instantiation.

## coercion rules

### canonical (Goldilocks)

```
field → word:  valid when value < 2^32 (range check)
word → field:  always valid (injection, [0, 2^32) ⊂ [0, p))
hash → field:  extract first element (lossy, for compatibility only)
field → hash:  forbidden (use HASH pattern 15)
```

## type errors

```
bitwise op on hash → ⊥_error
arithmetic on hash (except equality) → ⊥_error
```

## structural hash

every noun has a canonical hash computed by H (the instantiated hash function). type and structure information is embedded in the hash function's capacity region — not prepended to the input.

### canonical (Hemera)

domain separation via Hemera's sponge capacity — the same mechanism Hemera uses for leaf/node/root distinction in Merkle trees.

```
H(atom a)     = hemera_leaf(encode(a), capacity[14] = type_tag(a))
H(cell(l, r)) = hemera_node(H(l), H(r))
```

capacity layout for noun hashing:
- capacity[14] = atom type tag (0x00 field, 0x01 word, 0x02 hash) — atoms only
- capacity[9]  = FLAG_CHUNK for atoms, FLAG_PARENT for cells (Hemera tree flags)

the hash output is 32 bytes (4 field elements). no prefix bytes, no framing — the type is inside the permutation.

NOTE: the recursive hemera hash described above is the legacy structural hash. the canonical identity computation uses PCS commitments — see "polynomial representation" section. the recursive hash remains as the semantic definition (what the identity means). the PCS-based computation is the implementation (how it is computed efficiently). both produce the same identity for the same noun.

properties:
- deterministic: same noun always produces same hash
- collision-resistant: distinct nouns produce distinct hashes (Poseidon2 security)
- composable: cell hash depends only on child hashes, enabling incremental computation
- domain-separated: different atom types produce different hashes for the same value, enforced by the sponge capacity — not by input framing

## polynomial representation

every noun is a multilinear polynomial over {0,1}^k where k = ceil(log₂(leaves)).

```
atom(v)     → constant polynomial v
cell(a, b)  → g(x₁, x₂, ...) = (1 - x₁)·a(x₂, ...) + x₁·b(x₂, ...)
```

cell construction is variable prepend: the first variable selects which subtree (0 = left = a, 1 = right = b), and the remaining variables address within that subtree. an atom is the base case — a constant polynomial with no variables.

### identity

every noun's identity is computed as:

```
identity = hemera(PCS.commit(noun_polynomial) ‖ domain_tag)     32 bytes
```

one hemera call wraps the PCS commitment with a domain separation tag. the PCS commitment itself is O(d × N) field operations where d = expander degree (~6-10) and N = number of leaves. this is the Brakedown linear-time commitment.

for small nouns (≤56 bytes / ≤7 field elements): cost is comparable to a direct hemera absorption — the PCS commitment over a few elements is negligible.

for large nouns (>56 bytes): cheaper than recursive hemera hashing. field operations replace multiple hemera permutation calls. a 4 KiB noun: ~512 leaves × ~8 field ops = ~4,096 field ops for PCS.commit, plus 1 hemera call for the identity wrap. recursive hemera would require ~64 permutations × ~200 field ops = ~12,800 field ops.

one identity scheme for ALL nouns. no size threshold. no dual paths. atom or cell, 8 bytes or 8 MiB — same computation: PCS.commit the polynomial, hemera-wrap the commitment.

### axis as polynomial evaluation

axis(s, n) on a polynomial noun is a polynomial evaluation at a binary point in {0,1}^k. the binary encoding of the axis address selects the evaluation point. PCS opening proves the evaluation in O(1) — a ~75 byte proof regardless of noun depth. this replaces O(depth) tree traversal with O(1) polynomial evaluation.

## formulas

a formula is a noun of the form `cell(tag, body)` where tag is an atom encoding the pattern number (0-16).

```
cell(0, a)              axis
cell(1, c)              quote
cell(2, cell(x, y))    compose
cell(3, cell(a, b))    cons
cell(4, cell(t, cell(y, n)))  branch
cell(5, cell(a, b))    add
cell(6, cell(a, b))    sub
cell(7, cell(a, b))    mul
cell(8, a)              inv
cell(9, cell(a, b))    eq
cell(10, cell(a, b))   lt
cell(11, cell(a, b))   xor
cell(12, cell(a, b))   and
cell(13, a)             not
cell(14, cell(a, n))   shl
cell(15, a)             hash
cell(16, constraint)    hint
```

the distinction between code and data is purely contextual — the same noun can be an object in one reduction and a formula in another. this homoiconicity extends to the proof system: the stark proves that a specific noun (the formula) was applied to a specific noun (the object). the proof refers to the same binary tree structure that the execution operated on.

## axis addressing

pattern 0 navigates the noun binary tree using a numeric address.

```
axis(s, 0)   = H(s)                    hash introspection
axis(s, 1)   = s                        identity (root)
axis(s, 2)   = head(s)                  left child
axis(s, 3)   = tail(s)                  right child
axis(s, 2n)  = head(axis(s, n))         left of subtree
axis(s, 2n+1)= tail(axis(s, n))         right of subtree
```

axis on an atom (except 0 and 1) produces ⊥_error.

the binary encoding of the axis number traces a path from root to leaf: after the leading 1-bit, each 0 means "go left" and each 1 means "go right".

```
       1
      / \
     2   3
    / \ / \
   4  5 6  7
```

see patterns.md for the full reduction rule.

## memory representation

nouns are stored in a flat arena indexed by u32 references. no heap allocation, no pointer chasing — pure index arithmetic.

```
type NounRef = u32;

enum NounInner {
    Atom { value: F, tag: u8 },
    Cell { left: NounRef, right: NounRef },
}
```

### bounds

| parameter | value | rationale |
|-----------|-------|-----------|
| max depth | 64 | covers 2^64 leaves — more than particle count in cybergraph. axis path = 64 bits max |
| max count | 2^24 (16M nouns) | 16M × 16 bytes = 256 MB arena. configurable compile-time const. phone mode: 2^20 (16 MB). server: 2^28 (4 GB) |
| max atom size | 4 field elements (32 bytes) | hash type = 4 × F_p. field and word = 1 × F_p |

### structural sharing (DAG)

nouns are DAGs, not trees. hash-consing deduplicates structurally identical sub-nouns:

```
insert(arena, cell(l, r)):
  h = H(cell(l, r))
  if arena.hash_index[h] exists:
    return arena.hash_index[h]     // reuse existing node
  ref = arena.alloc(Cell { left: l, right: r })
  arena.hash_index[h] = ref
  return ref
```

properties:
- identical sub-expressions share one arena slot
- memory proportional to unique structure, not total size
- hash-consing cost: one hemera hash per cell construction
- lookup: O(1) via hash index (BoundedMap)
- DAG is safe because nouns are immutable — no mutation, no aliasing hazard

hash-consing is required, not optional. it ensures that `H(noun)` = arena identity — the same noun always has the same NounRef. this is the foundation of memoization correctness.

### arena lifecycle

one arena per ask() invocation. the arena is allocated at entry, all nouns live in it, and it is freed when ask() returns. no cross-computation noun sharing — each computation is isolated.

the memo cache stores (H(object), H(formula)) → H(result) — hashes, not NounRefs. NounRefs are arena-local and meaningless outside their computation.
