---
status: draft
date: 2026-06-09
author: jet taxonomy review session
---

> superseded: this page describes the retired zheng design (Merkle-free Brakedown, unchecked HyperNova fold, ~2 KiB / ~5 μs figures). It is unsound and its numbers do not hold. The repair and the measured numbers live in [[soft3/proposals/proof-system-repair|soft3 proposals/proof-system-repair]].

# decider — fold recursion in

## Abstract

The decider settles all accumulated chain history in one check. The four
jets grouped as `recursion` (`poly_eval`, `ntt`, `fri_fold`,
`merkle_verify`) are not a peer capability — they are field-computation
ingredients. This proposal dissolves the `recursion` group: `poly_eval`
serves the decider, `ntt` moves to the FHE product, and `fri_fold` /
`merkle_verify` are flagged for a deliberate genesis-inclusion decision.
Circuit ownership follows the domain — the verifier is zheng's.

Status (2026-10): the decider design this proposal was written against
(an unchecked fold, a Brakedown opening without Merkle authentication, constraint
counts and byte sizes for both) is retired as unsound. The legacy nox
decider jet verifies nothing (launch #40). Phase 3 of
[[soft3/proposals/proof-system-repair|the proof-system repair]] replaces it;
the ownership conclusion below survives, the figures do not.

## 1. The decider

In the repaired design the nox machine is one uniform step relation
(zheng `specs/machine.md`), accumulated step by step: each step's
Reed–Solomon evaluation claims are batched with a sumcheck into one root
plus a fixed number of claims, independent of the number of steps. The
decider is one WHIR opening (lens `specs/whir.md`) of the final
accumulator and returns accept / reject. A light client joins with a
checkpoint = state root + the decider proof: ≤ 64 KB goal, measured size
TODO(F-numbers); verification ≤ 1 ms goal, measured time TODO(F-numbers).
It verifies all history, not a window.

## 2. recursion is the decider's ingredients

A caller does not want `poly_eval` as an end — it wants a verified proof.
Multilinear evaluation is an ingredient of the decider's opening check, not
a separate product, so `poly_eval` belongs under the decider, not in a
separate `recursion` group. The decider itself is zheng code, not a frozen
nox circuit.

## 3. The other three recursion jets

- `ntt` — serves the FHE product (`polynomial-ring`: `ntt_batch`,
  `blind_rotate`) and ring multiplication generally. File it there, or as a
  shared primitive alongside `hash` — it is cross-cutting.
- `fri_fold`, `merkle_verify` — no genesis product consumes them as nox
  jets. The repaired PCS (WHIR, hemera Merkle trees) does use Merkle
  authentication paths, but inside zheng, not through a nox jet. Freezing a
  primitive at genesis (A3, append-only) that no product uses is permanent
  cost with nothing shipped behind it. Their inclusion is a deliberate
  decision, not a default — see open questions.

## 4. Ownership

Recognition is the VM's job: nox holds the decider's formula-hash anchor and
dispatches on it at reduction time. The implementation belongs to the repo
that owns the domain — and the verifier is the proof system's. The decider
and its soundness argument live in zheng (`specs/soundness.md`); nox keeps
at most the anchor. This is the boundary already applied across the
jet set:

- verifier circuit → zheng
- field arithmetic (`poly_eval`, `ntt`) → strata-compute / honeycrisp
- hash → hemera
- state → bbg ([[state-jets-redesign]])

## 5. Proposed structure

```
decider                            zheng implements it (one WHIR opening of the accumulator)
  ingredient:  poly_eval           strata-compute / honeycrisp implement; nox anchors

ntt              → file under the FHE product, or as a shared primitive
fri_fold         → interop primitive; genesis inclusion under review
merkle_verify    → interop primitive; genesis inclusion under review
```

1. Dissolve `recursion` as a top-level group in `specs/jets/README.md`. The
   decider becomes the group: `poly_eval` (ingredient) + the zheng decider.
2. `specs/jets/decider.md` records the legacy jet and its retirement; the
   decider and its soundness argument live in `zheng/specs/`.
3. Reassign `ntt` to the FHE product or a shared-primitive entry; park
   `fri_fold` and `merkle_verify` pending the inclusion decision.
4. `rs/jets/decider.rs` is the legacy jet (fixed charge, verifies nothing);
   it is retired in phase 3 rather than wired to zheng.

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
