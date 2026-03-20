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

properties:
- deterministic: same noun always produces same hash
- collision-resistant: distinct nouns produce distinct hashes (Poseidon2 security)
- composable: cell hash depends only on child hashes, enabling incremental computation
- domain-separated: different atom types produce different hashes for the same value, enforced by the sponge capacity — not by input framing

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
