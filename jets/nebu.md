---
tags: nox, jets
crystal-type: entity
crystal-domain: comp
alias: verifier jets, nebu jets, Goldilocks jets
---
# nebu jets — verifier (F_p)

five jets for nox<Goldilocks> + WHIR. make recursive proof composition practical: ~400K patterns → ~50K. 8× reduction.

## jets

| # | name | signature | exec cost | constraints | pure cost |
|---|------|-----------|-----------|-------------|-----------|
| 0 | hash | hash(x) → 4×F_p | 200 | ~736 | ~1,000 |
| 1 | poly_eval | poly_eval(coeffs, point) → F_p | N | ~N | ~2N |
| 2 | merkle_verify | merkle_verify(root, leaf, path, index) → {0,1} | d×200 | ~d×736 | d×~210 |
| 3 | fri_fold | fri_fold(poly_layer, challenge) → poly_layer_next | N/2 | ~N/2 | ~N |
| 4 | ntt | ntt(values, direction) → transformed values | N×log(N) | ~N×log(N) | ~2N×log(N) |

## verifier cost breakdown

```
Component               | Layer 1 only | With jets  | Reduction
------------------------+--------------+------------+----------
Parse proof             |     ~1,000   |    ~1,000  |  1×
Fiat-Shamir challenges  |    ~20,000   |    ~3,000  |  7×
Merkle verification     |   ~330,000   |   ~33,000  | 10×
Constraint evaluation   |    ~10,000   |    ~3,000  |  3×
WHIR verification       |    ~35,000   |    ~7,000  |  5×
------------------------+--------------+------------+----------
TOTAL                   |   ~400,000   |   ~50,000  | ~8×
```

## hardware mapping

- hash, merkle_verify → p2r (Poseidon2 round)
- poly_eval → fma (field multiply-accumulate, Horner = iterated FMA)
- ntt → ntt (direct correspondence)
- fri_fold → ntt + fma

## PCS backend

PCS₁: Brakedown (expander-graph linear codes, Merkle-free)

## note on merkle_verify

with polynomial nouns, most data authenticated via PCS openings (O(1) field ops) rather than Merkle paths. merkle_verify remains for backward compatibility and cross-system interop.
