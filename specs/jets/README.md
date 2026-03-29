---
tags: nox, jets
crystal-type: entity
crystal-domain: comp
---
# genesis jet registry

genesis jets across five algebras + state transitions + the universal decider. each jet has an equivalent pure Layer 1 program. committed in genesis BBG state. remove all jets: identical results, orders of magnitude slower.

see [[jets|jets spec]] for principles, semantic contract, recognition mechanism, and hardware mapping.

## registry

| group | algebra | jets | PCS | spec |
|-------|---------|------|-----|------|
| hash | all (Hemera anchor) | 1 | — | [[hash]] |
| recursion | F_p (Goldilocks) | 4 | Brakedown | [[recursion]] |
| binary-tower | F₂ (binary tower) | 8 | Binius | [[binary-tower]] |
| polynomial-ring | R_q (polynomial ring) | 5 | Ikat | [[polynomial-ring]] |
| isogeny-curves | F_q (isogeny curves) | 5 (incl. secret_hash boundary) | Porphyry | [[isogeny-curves]] |
| tropical-semiring | (min,+) (tropical) | 6 (incl. witness_commit boundary) | Assayer | [[tropical-semiring]] |
| state | F_p (polynomial nouns) | 6 (1 exact + 5 templates) | Brakedown | [[state]] |
| decider | F_p (accumulator) | 1 | Brakedown | [[decider]] |

boundary jets (quantize, dequantize, gadget_decompose, secret_hash, witness_commit) live within their parent algebra.

## counts

| group | count | speedup target |
|-------|-------|---------------|
| hash | 1 | ~5× every hemera call (universal) |
| recursion | 4 | recursive proof composition |
| binary-tower | 8 | 32-90× quantized inference (incl. 3 boundary jets) |
| polynomial-ring | 5 | ~log(n)-n× FHE bootstrapping |
| isogeny-curves | 5 | native F_q privacy (incl. secret_hash boundary) |
| tropical-semiring | 6 | O(|problem|) → O(|witness|) proof cost (incl. witness_commit boundary) |
| state | 6 | 500× state transition proving (1 exact circuit + 5 templates) |
| decider | 1 | 89 constraints — all-history verification |

## hardware mapping

```
GFP primitive    jets                                        groups
────────────     ────                                        ──────
fma              poly_eval, key_switch, noise_track,         recursion, polynomial-ring, isogeny-curves
                 group_action, isogeny_walk, vrf_eval
ntt              ntt, ntt_batch, blind_rotate                recursion, polynomial-ring
p2r              hash, merkle_verify                         recursion
lut              activation_lut, tropical comparisons        binary-tower, tropical-semiring
```
