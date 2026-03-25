---
tags: nox, jets
crystal-type: entity
crystal-domain: comp
alias: isogeny jets, genies jets, F_q jets, privacy jets
---
# genies jets — isogeny (F_q)

four jets for commutative group action on supersingular curves over F_q. privacy primitives. proved via PCS₄ (isogeny) in zheng.

## jets

| # | name | input → output | pure cost | jet cost | primary workload |
|---|------|----------------|-----------|----------|------------------|
| 0 | group_action | secret, curve → resulting curve | ~1000 F_q muls | ~500 | NIKE shared secret derivation |
| 1 | isogeny_walk | start_curve, path → end_curve | ~|path|×100 | ~|path|×50 | stealth address computation |
| 2 | vrf_eval | sk, input → (output, proof) | ~500 F_q muls | ~250 | verifiable random function |
| 3 | vdf_step | prev, T → (result, proof) | T sequential | T sequential | verifiable delay (inherently sequential) |
| 4 | secret_hash | F_q shared secret → F_p hemera digest | ~736 | ~736 | genies → nebu boundary |

## vdf_step is unique

vdf_step CANNOT be accelerated in execution time — VDF is inherently sequential. the jet optimizes the PROOF: fast verification of T sequential steps, not faster execution. the sequentiality IS the security property.

## cross-algebra

all genies jets produce F_q results. shared secrets are hashed into Goldilocks via hemera at the algebra boundary:

```
genies computation (F_q) → shared_secret (F_q element)
  ↓
secret_hash boundary jet: H(shared_secret) → F_p hemera digest
  ↓
nebu continuation (F_p)
```

zheng folds F_q sub-traces into the F_p accumulator via PCS₄ (~766 constraints per crossing).

## hardware mapping

- group_action, isogeny_walk, vrf_eval → fma (multi-limb field multiply-accumulate)
- vdf_step → fma (sequential, cannot parallelize)

## PCS backend

PCS₄: Isogeny (Brakedown instantiated over F_q)
