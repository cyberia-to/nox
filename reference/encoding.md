# noun encoding specification

version: 0.2
status: canonical

## identity

every noun has an identity derived from its polynomial commitment (see nouns.md polynomial representation). the nox protocol operates exclusively on these identities.

in the canonical instantiation (nox<Goldilocks, Z/2^32, Hemera>), the identity is 32 bytes (4 × F_p elements). identity size is per-instantiation — it depends on H's output size. all concrete sizes below refer to the canonical instantiation.

```
identity = hemera(PCS.commit(noun_polynomial) ‖ domain_tag)    32 bytes
computation_key = (identity(object), identity(formula))         64 bytes
computation_val = identity(result)                              32 bytes
```

the identity computation is: encode the noun as a multilinear polynomial, compute the PCS commitment (O(N) field operations where N = number of leaves, using Brakedown linear-time commitment), then wrap with hemera for domain separation (1 hemera call). total cost: O(N) field ops + 1 hemera call. for small nouns (<56 bytes): same cost as a direct hemera hash. for large nouns: cheaper — field operations replace multiple hemera absorptions.

## content-addressed storage

there is no reason for a prefix byte in a system where every value is looked up by its hash. nouns are stored and transmitted as content-addressed entries. no prefix bytes. no self-describing framing. the content byte length determines interpretation.

### canonical (nox<Goldilocks, Z/2^32, Hemera>)

```
store: Identity (32 bytes) → Content

Content variants (determined by byte length):
   8 bytes     atom value (single field element, canonical LE)
  32 bytes     hash value (4 field elements, canonical LE)
  64 bytes     cell (H(left) ‖ H(right), two 32-byte identities)
```

three sizes: 2³, 2⁵, 2⁶. all powers of 2.

content sizes are per-instantiation. atom size = sizeof(F). hash size = sizeof(H output). cell size = 2 × hash size. in nox<F₂>, an atom is 1 bit. the storage model is the same — only the element widths change.

## resolution

to materialize a noun from its identity:

```
resolve(id):
  content = store.get(id)
  match content.len():
    8   → atom(decode_field_element(content))
    32  → hash_atom(decode_4_field_elements(content))
    64  → cell(resolve(content[0..32]), resolve(content[32..64]))
```

## atom type distinction

field atoms and word atoms store the same 8 bytes. the type distinction is a runtime property — which arithmetic algebra applies (modular vs bitwise). the structural hash distinguishes them (capacity[14] produces different identities for the same value with different types). the store does not need to know the type. the VM knows from execution context. the STARK proof verifies type correctness.

## field element encoding (canonical: Goldilocks)

```
8 bytes, little-endian, canonical value in [0, p)
p = 2^64 - 2^32 + 1
values ≥ p are invalid
```

## hash encoding (canonical: Hemera)

```
32 bytes = 4 × 8 bytes
each element is a canonical field element in [0, p), little-endian
```

## canonical invariants

1. field element values MUST be in [0, p)
2. hash values: each of the 4 elements MUST be in [0, p)
3. cell content is deterministic: left identity before right identity
4. no trailing bytes after content
5. content length MUST be exactly 8, 32, or 64 bytes

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
