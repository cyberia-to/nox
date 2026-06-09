---
status: draft
date: 2026-06-09
author: jet taxonomy review session
---

# genesis jets as products

## Abstract

A jet exists to deliver a capability. The genesis registry should be
organized and documented by that capability — the *product* each jet
delivers: a tangible output, powerful, complete. Under this principle the
decider is the verification product, and the field-computation primitives
currently grouped as `recursion` are its ingredients, not a peer group.
`poly_eval` folds in as the decider's core ingredient; `ntt`, `fri_fold`,
and `merkle_verify` are primitives serving other products or none. Circuit
ownership follows the domain: the verifier circuit is zheng's; nox holds the
formula-hash anchor.

## 1. A jet is a product

A jet replaces a pure Layer 1 formula with a fast implementation that
produces identical output. The jet earns its place in genesis only because
some capability is worth accelerating. That capability is the jet's product.

Three marks of a product:

- **tangible** — you can name the output and hand it to a caller.
- **powerful** — large leverage: a big speedup, or a capability that does
  not otherwise exist.
- **complete** — a whole capability, not a fragment of one.

Decision procedure: name the product. If you can name a complete capability
a caller wants as an *end*, it is a product and earns a top-level registry
entry. If you can only name a tool used *inside* something else, it is an
ingredient and belongs under the product it serves.

Record this in the spec. Each entry in `specs/jets/` carries a `product:`
line stating the capability it delivers; ingredients state the product they
compose into. The registry is grouped by product, and a group's jets are
its product plus the ingredients that build it.

## 2. The decider is the verification product

```
product: verify all chain history in one step
```

- **tangible** — accept / reject against the HyperNova accumulator
  (~200 bytes folding every block from genesis).
- **powerful** — 89 constraints, ~100 ns. A light client joins by
  downloading a 240-byte checkpoint and verifying it — less work than one
  hemera permutation (736 constraints).
- **complete** — all history from genesis, not a window.

This is the most tangible, powerful, complete output in the registry. It is
a product by every mark.

## 3. recursion is ingredients

The four jets grouped as `recursion` — `poly_eval`, `ntt`, `fri_fold`,
`merkle_verify` — name techniques, not capabilities. A caller does not want
`poly_eval` as an end; it wants a verified proof, a polynomial product, a
settled commitment. Each is an ingredient.

`poly_eval` is the decider's core ingredient. The decider's 89-constraint
breakdown (`specs/jets/decider.md`):

```
sumcheck replay:         20 constraints
CCS evaluation:          34 constraints
Brakedown (batched):     35 constraints   ← poly_eval, fused
hemera:                   0 constraints   (algebraic Fiat-Shamir)
────────────────         ──────────────
total:                   89 constraints
```

The 35-constraint Brakedown step is `poly_eval`'s operation — multilinear
opening — fused and specialized into the verifier. The decider *is*
`poly_eval` + sumcheck + CCS evaluation, frozen as one exact-match circuit.
So `poly_eval` belongs under the decider product, not in a separate group.

## 4. The other primitives

- **`ntt`** — serves the FHE product (`polynomial-ring`: `ntt_batch`,
  `blind_rotate`) and ring multiplication generally. File it under FHE, or
  as a named shared primitive — like `hash`, it is cross-cutting.
- **`fri_fold`, `merkle_verify`** — no genesis product consumes them.
  Brakedown is Merkle-free and FRI-free. They were retained for cross-system
  interoperability. Freezing a primitive at genesis (A3, append-only) that
  no product uses is permanent cost with nothing shipped behind it. Their
  inclusion is a deliberate decision, not a default — see open questions.

## 5. Ownership follows the domain

Recognition is the VM's job: nox holds each jet's formula-hash anchor and
dispatches on it at reduction time. The implementation belongs to the repo
that owns the domain:

- verifier circuit → zheng (the proof system)
- field arithmetic (`poly_eval`, `ntt`) → strata-compute / honeycrisp
- hash → hemera
- state → bbg ([[state-jets-redesign]])

The decider's CCS circuit, its three optimizations, and its soundness proofs
live in zheng. nox keeps the anchor so the jet is recognized; zheng
implements the product. Grouping is by product; ownership is by domain repo
— two independent axes.

## 6. Proposed structure

```
decider (verification product)     zheng implements the circuit; nox anchors
  ingredient:  poly_eval           strata-compute / honeycrisp implement; nox anchors
  product:     decider circuit     = poly_eval + sumcheck + CCS, fused

ntt              → file under the FHE product, or as a shared primitive
fri_fold         → interop primitive; genesis inclusion under review
merkle_verify    → interop primitive; genesis inclusion under review
```

1. Add a `product:` line to every `specs/jets/` entry.
2. Dissolve `recursion` as a top-level group in `specs/jets/README.md`. The
   `decider` group is the verification product: `poly_eval` (ingredient) +
   the decider circuit (product).
3. `specs/jets/decider.md` states the formula, its hash, the contract, and
   the `product:` line; the constraint breakdown, the optimizations, and the
   soundness proofs migrate to `zheng/specs/`.
4. Reassign `ntt` to the FHE product or a shared-primitive entry; park
   `fri_fold` and `merkle_verify` pending the inclusion decision.
5. `rs/jets/poly_eval.rs` and `rs/jets/decider.rs` keep their wrappers; wire
   the decider wrapper to zheng's verifier when exposed.

## 7. Open questions

1. Genesis-digest source of truth: does the decider's formula hash live in
   nox's `compute_genesis_digests` or in zheng's genesis manifest? The hash
   must be identical either way.
2. `fri_fold` and `merkle_verify` serve no genesis product. Freeze them at
   genesis as interop primitives, or defer them post-genesis? Freezing has a
   permanent A3 cost; deferring risks an interop gap. A concrete interop
   requirement decides it.
3. Is `ntt` better filed under the FHE product (its primary consumer) or as
   a standalone shared primitive alongside `hash`? The registry has no
   "shared primitive" category today.
