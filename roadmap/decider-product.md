---
status: draft
date: 2026-06-09
author: jet taxonomy review session
---

# decider — fold recursion in

## Abstract

The decider verifies all accumulated chain history in one step. The four
jets grouped as `recursion` (`poly_eval`, `ntt`, `fri_fold`,
`merkle_verify`) are not a peer capability — they are field-computation
ingredients, and the decider is what `poly_eval` builds into. This proposal
dissolves the `recursion` group: `poly_eval` folds into the decider as its
core ingredient, `ntt` moves to the FHE product, and `fri_fold` /
`merkle_verify` are flagged for a deliberate genesis-inclusion decision.
Circuit ownership follows the domain — the verifier is zheng's; nox holds
the formula-hash anchor.

## 1. The decider

The decider checks the HyperNova accumulator (~200 bytes folding every block
from genesis) and returns accept / reject. 89 constraints, ~100 ns — a light
client joins by downloading a 240-byte checkpoint and verifying it, less
work than one hemera permutation (736 constraints). It verifies all history,
not a window. It is a complete, standalone capability.

## 2. recursion is the decider's ingredients

A caller does not want `poly_eval` as an end — it wants a verified proof.
`poly_eval` is the decider's core ingredient. Its 89-constraint breakdown
(`specs/jets/decider.md`):

```
sumcheck replay:         20 constraints
CCS evaluation:          34 constraints
Brakedown (batched):     35 constraints   ← poly_eval, fused
hemera:                   0 constraints   (algebraic Fiat-Shamir)
────────────────         ──────────────
total:                   89 constraints
```

The 35-constraint Brakedown step is `poly_eval`'s operation — multilinear
opening — fused into the verifier. The decider *is* `poly_eval` + sumcheck +
CCS evaluation, frozen as one exact-match circuit. `poly_eval` belongs under
the decider, not in a separate `recursion` group.

## 3. The other three recursion jets

- **`ntt`** — serves the FHE product (`polynomial-ring`: `ntt_batch`,
  `blind_rotate`) and ring multiplication generally. File it there, or as a
  shared primitive alongside `hash` — it is cross-cutting.
- **`fri_fold`, `merkle_verify`** — no genesis product consumes them.
  Brakedown is Merkle-free and FRI-free. They were retained for cross-system
  interoperability. Freezing a primitive at genesis (A3, append-only) that
  no product uses is permanent cost with nothing shipped behind it. Their
  inclusion is a deliberate decision, not a default — see open questions.

## 4. Ownership

Recognition is the VM's job: nox holds the decider's formula-hash anchor and
dispatches on it at reduction time. The implementation belongs to the repo
that owns the domain — and the verifier is the proof system's. The decider's
CCS circuit, its three optimizations, and its soundness proofs live in
zheng; nox keeps the anchor. This is the boundary already applied across the
jet set:

- verifier circuit → zheng
- field arithmetic (`poly_eval`, `ntt`) → strata-compute / honeycrisp
- hash → hemera
- state → bbg ([[state-jets-redesign]])

## 5. Proposed structure

```
decider                            zheng implements the circuit; nox anchors
  ingredient:  poly_eval           strata-compute / honeycrisp implement; nox anchors
  circuit:     decider             = poly_eval + sumcheck + CCS, fused

ntt              → file under the FHE product, or as a shared primitive
fri_fold         → interop primitive; genesis inclusion under review
merkle_verify    → interop primitive; genesis inclusion under review
```

1. Dissolve `recursion` as a top-level group in `specs/jets/README.md`. The
   decider becomes the group: `poly_eval` (ingredient) + the decider circuit.
2. `specs/jets/decider.md` keeps the formula, its hash, and the contract; the
   constraint breakdown, the optimizations, and the soundness proofs migrate
   to `zheng/specs/`.
3. Reassign `ntt` to the FHE product or a shared-primitive entry; park
   `fri_fold` and `merkle_verify` pending the inclusion decision.
4. `rs/jets/poly_eval.rs` and `rs/jets/decider.rs` keep their wrappers; wire
   the decider wrapper to zheng's verifier when exposed.

## 6. Note: jets as products

The lens behind this proposal generalizes and is worth recording in
`specs/jets/`: a jet earns its genesis slot because it delivers a *product*
— a capability a caller wants as an end (tangible, powerful, complete). What
a caller only uses *inside* something else is an ingredient and belongs under
the product it serves. `recursion` named a technique, which is why it
bundled one verification ingredient (`poly_eval`) with three unrelated
primitives. Adding a `product:` line to each `specs/jets/` entry makes this
explicit and keeps the registry honest about what each entry is for. This is
a documentation convention, separable from the decider change above.

## 7. Open questions

1. Genesis-digest source of truth: does the decider's formula hash live in
   nox's `compute_genesis_digests` or in zheng's genesis manifest? The hash
   must be identical either way.
2. `fri_fold` and `merkle_verify` serve no genesis product. Freeze them at
   genesis as interop primitives, or defer them post-genesis? Freezing has a
   permanent A3 cost; deferring risks an interop gap. A concrete interop
   requirement decides it.
3. Is `ntt` better filed under the FHE product or as a standalone shared
   primitive alongside `hash`? The registry has no "shared primitive"
   category today.
