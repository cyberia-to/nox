---
tags: nox, jets
crystal-type: entity
crystal-domain: comp
---
# jet registry

jets across five algebras + state transitions + the universal decider. each jet has an equivalent pure Layer 1 program. remove all jets: identical results, orders of magnitude slower.

see [[reference/jets|jets spec]] for principles, semantic contract, recognition mechanism, and hardware mapping.

## registry

| group | algebra | jets | [[lens]] | spec |
|-------|---------|------|----------|------|
| [[nebu]] | F_p (Goldilocks) | 5 | Brakedown | [[nebu]] |
| [[kuro]] | F₂ (binary tower) | 8 | Binius | [[kuro]] |
| [[jali]] | R_q (polynomial ring) | 5 | Ring-aware | [[jali]] |
| [[genies]] | F_q (isogeny curves) | 5 (incl. secret_hash boundary) | Isogeny | [[genies]] |
| [[trop]] | (min,+) (tropical) | 6 (incl. witness_commit boundary) | Tropical | [[trop]] |
| [[state]] | F_p (polynomial nouns) | level 1-3 (unlimited) | Brakedown | [[state]] |
| [[decider]] | F_p (accumulator) | 1 | Brakedown | [[decider]] |

boundary jets (quantize, dequantize, gadget_decompose, secret_hash, witness_commit) live within their parent algebra — kuro, genies, and trop respectively.

## counts

| algebra | count | speedup target |
|---------|-------|---------------|
| nebu | 5 | 8× recursive proof composition |
| kuro | 8 | 32-90× quantized inference (incl. 3 boundary jets) |
| jali | 5 | ~log(n)-n× FHE bootstrapping |
| genies | 5 | native F_q privacy (incl. secret_hash boundary) |
| trop | 6 | O(|problem|) → O(|witness|) proof cost (incl. witness_commit boundary) |
| state | unlimited | 500× state transition proving |
| decider | 1 | 90× all-history verification (89 constraints) |

## hardware mapping

```
GFP primitive    jets                                        algebras
────────────     ────                                        ────────
fma              poly_eval, key_switch, noise_track,         nebu, jali, genies
                 group_action, isogeny_walk, vrf_eval
ntt              ntt, ntt_batch, blind_rotate                nebu, jali
p2r              hash, merkle_verify                         nebu
lut              activation_lut, trop comparisons            kuro, trop
```
