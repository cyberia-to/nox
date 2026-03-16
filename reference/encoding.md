# noun encoding specification

version: 0.1
status: canonical

## identity

every noun has a 64-byte identity computed by Hemera (see nouns.md structural hash). the nox protocol operates exclusively on 64-byte identities.

```
identity = H(noun)                          64 bytes
computation_key = (H(object), H(formula))  128 bytes
computation_val = H(result)                 64 bytes
```

## content-addressed storage

there is no reason for a prefix byte in a system where every value is looked up by its hash. nouns are stored and transmitted as content-addressed entries. no prefix bytes. no self-describing framing. the content byte length determines interpretation.

```
store: Identity (64 bytes) → Content

Content variants (determined by byte length):
   8 bytes     atom value (single field element, canonical LE)
  64 bytes     hash value (8 field elements, canonical LE)
 128 bytes     cell (H(left) ‖ H(right), two 64-byte identities)
```

three sizes: 2³, 2⁶, 2⁷. all powers of 2.

## resolution

to materialize a noun from its identity:

```
resolve(id):
  content = store.get(id)
  match content.len():
    8   → atom(decode_field_element(content))
    64  → hash_atom(decode_8_field_elements(content))
    128 → cell(resolve(content[0..64]), resolve(content[64..128]))
```

## atom type distinction

field atoms and word atoms store the same 8 bytes. the type distinction is a runtime property — which arithmetic algebra applies (modular vs bitwise). the structural hash distinguishes them (capacity[14] produces different identities for the same value with different types). the store does not need to know the type. the VM knows from execution context. the STARK proof verifies type correctness.

## field element encoding

```
8 bytes, little-endian, canonical value in [0, p)
p = 2^64 - 2^32 + 1
values ≥ p are invalid
```

## hash encoding

```
64 bytes = 8 × 8 bytes
each element is a canonical field element in [0, p), little-endian
```

## canonical invariants

1. field element values MUST be in [0, p)
2. hash values: each of the 8 elements MUST be in [0, p)
3. cell content is deterministic: left identity before right identity
4. no trailing bytes after content
5. content length MUST be exactly 8, 64, or 128 bytes

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

formulas are nouns. they are stored and resolved the same way as any other noun — by identity lookup, recursively.
