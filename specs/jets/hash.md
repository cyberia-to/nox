---
tags: nox, jets
crystal-type: entity
crystal-domain: comp
alias: hash jet, hemera jet, Poseidon2 jet
---
# hash jet — universal genesis jet

the one jet every algebra needs. Hemera (Poseidon2-Goldilocks) is the anchor — all algebras settle through F_p via hemera. hash is the only jet that crosses every instantiation boundary.

committed in genesis BBG state.

## jet

| name | signature | exec cost | constraints | pure cost |
|------|-----------|-----------|-------------|-----------|
| hash | hash(x) → 4×F_p | 200 | ~736 | ~1,000 |

## where hash is used

| context | frequency | purpose |
|---------|-----------|---------|
| content addressing | every particle | identity = H(content) |
| Fiat-Shamir | ~3 per proof | verifier challenges |
| domain separation | every commitment, nullifier, Merkle op | collision isolation |
| structural hash | every noun | H(cell) = H(H(left) ‖ H(right)) |
| cross-algebra boundary | every type transition | hemera commitment at algebra crossing |

hash is the most frequently executed jet in the system. without it, one Poseidon2 permutation = ~1,000 Layer 1 patterns. with it, 200 execution cost, ~736 STARK constraints.

## the Hemera anchor

all algebras compute natively in their own field. all algebras settle through Goldilocks via Hemera:

- nox<F_p>: hash is native (200 budget cost)
- nox<F_p²>, nox<F_p³>, nox<F_p⁴>: hash operates on base field (native)
- nox<F₂>: hash deferred to settlement boundary (~736 constraints at boundary)
- nox<R_q>: hash operates on coefficient field F_p (native)
- nox<F_q>: hash deferred to settlement boundary (~736 constraints at boundary)

the hash jet is the same across all instantiations — the Hemera permutation over Goldilocks. what changes is when it executes (native vs deferred).

## hardware mapping

hash → p2r (Poseidon2 round) on GFP

## parameters

from hemera spec (frozen at genesis):

```
state:    16 field elements
rate:     8 elements
capacity: 8 elements
rounds:   8 full (4+4) + 16 partial = 24 total
full-round S-box:    x^7
partial-round S-box: x^(-1) (field inversion, 0^(-1) = 0)
output:   4 field elements (32 bytes)
```
