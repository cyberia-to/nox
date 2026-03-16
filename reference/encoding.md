# noun encoding specification

version: 0.1
status: canonical

## overview

canonical binary encoding of nox nouns. deterministic — exactly one valid encoding per noun. used for hashing, serialization, and content addressing.

## wire format

```
noun = 0x00 type_tag encode(atom)         atom
     | 0x01 encode(left) encode(right)    cell

type_tag:
  0x00  field element
  0x01  word
  0x02  hash (4 elements follow)

field element encoding:
  8 bytes, little-endian, value in [0, p)

word encoding:
  8 bytes, little-endian, value in [0, 2^64)

hash encoding:
  32 bytes (4 × 8 bytes, each a field element, little-endian)
```

## canonical invariants

1. no trailing bytes after the last noun
2. field element values MUST be in [0, p). values ≥ p are invalid
3. word values MUST be in [0, 2^64). always true for u64
4. hash values: each of the 4 elements MUST be in [0, p)
5. cell encoding is deterministic: left before right

## formula encoding

a formula is a noun of the form `cell(tag, body)` where tag is an atom (pattern number 0-16).

```
[0 a]        axis
[1 c]        quote
[2 [x y]]    compose
[3 [a b]]    cons
[4 [t [y n]]]  branch
[5 [a b]]    add
...
[15 a]       hash
[16 c]       hint
```

## content-addressed identity

```
particle_id = H(encoded_noun)

where H = Hemera hash (Poseidon2 sponge over Goldilocks)
```

two identical nouns always produce the same encoding, therefore the same hash. this is the foundation of content-addressed computation:

```
computation_key = (H(object), H(formula))
computation_val = H(result)
```

## size bounds

```
atom:  9 bytes (1 tag + 8 value) for field/word
       33 bytes (1 tag + 32 value) for hash
cell:  1 byte overhead + encode(left) + encode(right)
```

worst case: deeply nested cells with atom leaves. tree of depth d with 2^d atoms: 2^d × 9 + (2^d - 1) × 1 bytes.
