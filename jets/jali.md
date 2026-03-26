---
tags: nox, jets
crystal-type: entity
crystal-domain: comp
alias: ring jets, jali jets, R_q jets, FHE jets
---
# jali jets — ring (R_q)

five jets for polynomial ring operations over R_q = F_p[x]/(x^n+1). FHE bootstrapping, lattice KEM, and ring convolution. proved via Ikat (ring-aware) in zheng.

## jets

| # | name | input → output | pure cost | jet cost | primary workload |
|---|------|----------------|-----------|----------|------------------|
| 0 | ntt_batch | n polys sharing ring → batched NTT | n×N×log(N) | n×N | blind rotation (n poly muls batched) |
| 1 | key_switch | ct, ks_key, k → switched ct | k×N | k×log(N) | FHE key switching via automorphisms |
| 2 | gadget_decomp | coeff, base, digits → digit sequence | ~k² | ~k | FHE bootstrapping phase 1 (→ Binius boundary) |
| 3 | noise_track | bound, operation → updated bound | ~64n per op | ~30 per fold | FHE noise budget tracking |
| 4 | blind_rotate | decomposed, bsk → rotated ct | n×N×log(N) | n×N (batched) | full blind rotation step |

## key optimization: NTT batching

n polynomial multiplies sharing the same evaluation domain commit as one batch instead of n independent commitments. the NTT structure IS the evaluation domain — no separate encoding step.

```
generic:     n independent commitments, n × O(N log N) constraints
ring-aware:  1 batch commitment, n × O(N) constraints
savings:     ~log(n) factor on commitment, ~n factor on NTT correctness
```

## cross-algebra

gadget_decomp (jet 2) crosses to F₂ (Binius boundary). the digit sequence is binary — proved via Binius. HyperNova folds the F₂ sub-trace back into the F_p accumulator (~766 constraints per crossing).

FHE bootstrapping crosses three algebras:
```
step 1: gadget_decomp    → kuro (F₂, Binius)
step 2: blind_rotate     → jali (R_q, ring-aware)
step 3: key_switch       → nebu (F_p, Brakedown)
step 4: mod_switch        → nebu (F_p, Brakedown)
```

## hardware mapping

- ntt_batch, blind_rotate → ntt (NTT butterfly engine)
- key_switch, noise_track → fma (field multiply-accumulate)

## lens

Ikat: Ikat (Brakedown with NTT batching, automorphism exploitation, noise accumulator)
