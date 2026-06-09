---
status: draft
date: 2026-06-09
author: jet taxonomy review session
---

# decider ownership — the verifier belongs to zheng

## Abstract

The genesis jet registry (`specs/jets/README.md`) groups jets by algebra:
one group each for F₂, R_q, F_q, and tropical (min,+). F_p is the exception
— it is split three ways: `recursion`, `state`, and `decider`. This
proposal addresses the `recursion` / `decider` split.

The `decider` is not a peer of the `recursion` jets. It is the fused
canonical verifier, *built from* the recursion family — one frozen
composition in the vocabulary those primitives define. The number proves
it: the recursion spec gives the canonical Brakedown verifier as "~825
constraints (CCS jet + batch)"; the decider's conservative tier is
"89 + 736 = 825." Same circuit.

This proposal: keep `recursion` as the four reusable F_p compute primitives,
and move the `decider` out of nox's jet groups. The verifier circuit is a
protocol artifact owned by zheng; nox holds only its formula-hash anchor —
exactly as the `hash` jet anchors to hemera and `fri_fold` anchors to lens.

## 1. The problem

The registry's organizing principle is algebra. There is one group per
algebra, except F_p carries three. Two of those three — `recursion` and
`decider` — are not coordinate concerns at the same layer:

| | recursion jets | decider jet |
|---|---|---|
| nature | open vocabulary — composable primitives | one closed circuit — exact-match only |
| reuse | `poly_eval` = any Horner eval; `ntt` = any poly mult | one job: accept/reject the accumulator |
| recognition | each has its own formula hash, fires anywhere | single hand-fused CCS matrix, recognized once |
| layer | the algebra of field computation | a frozen composition in that algebra |

The recursion group itself already states the split internally
(`specs/jets/recursion.md`): `poly_eval` is verifier-critical (the
Brakedown opening), while `merkle_verify`, `fri_fold`, and `ntt` are
"retained for cross-system interoperability and domain-specific
acceleration" — explicitly **not used by the Brakedown verifier**. `ntt`
is consumed by the polynomial-ring/FHE jets; `fri_fold` serves external
FRI protocols.

So naming a merged group `decider` would be a category error: it names the
toolbox after one tool, and files general primitives (`ntt`, `fri_fold`)
that have nothing to do with verification under the name of the one
circuit that does. The decider is the apex, not the category.

## 2. Why the decider is a composite, not a primitive

The decider verifies the entire accumulated chain history — the HyperNova
accumulator (~200 bytes) folding every block from genesis — in one step.
Its 89-constraint breakdown (`specs/jets/decider.md`):

```
sumcheck replay:         20 constraints
CCS evaluation:          34 constraints
Brakedown (batched):     35 constraints
hemera:                   0 constraints (algebraic Fiat-Shamir)
────────────────         ──────────────
total:                   89 constraints
```

The 35-constraint "Brakedown (batched)" step is the same mathematical
operation `poly_eval` performs as a recursion jet — multilinear opening
verification — fused and specialized into the monolith. The decider
deliberately re-implements what `poly_eval` does, because fusing the whole
verifier into one exact-match circuit beats dispatching through the general
primitive. That is the definition of a composite: it is `poly_eval` +
sumcheck + CCS evaluation, frozen as one circuit and recognized by one
formula hash.

A primitive is reused across contexts. The decider has exactly one
context: verifying the accumulator. It is a protocol artifact.

## 3. The ownership principle

This session has applied one boundary rule consistently:

> the jet wrapper (noun I/O, TraceRow, budget) lives in nox; the primitive
> implementation lives in its domain repo.

- `hash` jet → hemera (Poseidon2)
- `merkle_verify` → hemera (tree hash)
- `fri_fold` → lens (polynomial commitment)
- `ntt`, `poly_eval` → strata-compute (Spectral) / honeycrisp (acpu)
- state jets → bbg (authenticated state)

The decider is the verifier of the proof system. zheng *is* the proof
system (SuperSpartan + Brakedown). By the same rule, the decider circuit
belongs to zheng. nox keeps the formula-hash anchor so the jet is
recognized at reduction time; zheng owns the CCS circuit, the three
optimizations, and the soundness proofs.

This is precisely the `hash` jet pattern: nox anchors, hemera implements.
The decider is `hash` for verification — nox anchors, zheng implements.

## 4. Proposed structure

```
recursion (F_p compute)        nox owns — the open vocabulary
  poly_eval, ntt, fri_fold, merkle_verify

decider                        zheng owns the circuit; nox holds the anchor hash
  = the fused canonical verifier, built from poly_eval + sumcheck + CCS
```

Concretely:

1. `recursion` stays as the four reusable F_p primitives. No change to
   `specs/jets/recursion.md` beyond removing any implication that the
   decider is a sibling.
2. The decider leaves the nox jet-group taxonomy. `specs/jets/decider.md`
   becomes an anchor spec: it states the formula, its hash, and the
   contract, and points to zheng for the circuit. The full constraint
   breakdown, the three optimizations, and the soundness questions migrate
   to `zheng/specs/`.
3. `rs/jets/decider.rs` keeps the wrapper (formula-hash dispatch, budget,
   delegation to the verifier). The verifier itself is a zheng dependency,
   not inline nox code.

This also cleans the three-way F_p split: `recursion` is F_p *compute*
(nox), `state` is F_p *state-machine* (bbg-owned — see
[[state-jets-redesign]]), and `decider` leaves the nox group taxonomy
entirely (zheng-owned). Each F_p sub-domain has one owner.

## 5. What does not change

- The decider remains a genesis jet, recognized by exact formula match.
  Ownership moves; the registry entry and the frozen anchor do not.
- `poly_eval` remains verifier-critical and remains a nox recursion jet.
  The decider depends on it; that dependency is the point.
- The 89/825 two-tier constraint story is unchanged — it just lives in
  zheng's verifier spec.

## 6. Migration

1. Write `zheng/specs/decider.md` — move the constraint breakdown, the
   three optimizations, and the open questions from `nox/specs/jets/decider.md`.
2. Reduce `nox/specs/jets/decider.md` to the anchor spec (formula, hash,
   contract, pointer to zheng).
3. Update `nox/specs/jets/README.md`: drop `decider` as a separate algebra
   group; note it as the zheng-owned verifier anchored in F_p.
4. Leave `rs/jets/decider.rs` as the wrapper. When zheng exposes the
   verifier, wire the wrapper to it (mirrors the hemera/lens wiring).

## 7. Open questions

1. Does the decider's formula hash live in nox's `compute_genesis_digests`
   (anchored here) or in zheng's genesis manifest? The hash must be
   identical either way; the question is which repo is the source of truth.
2. Recursion has four members but only `poly_eval` is verifier-critical.
   Should `merkle_verify` / `fri_fold` / `ntt` be reclassified as a
   general "F_p compute" group distinct from "recursion (verification
   support)", or is one group with an internal note sufficient? (Leaning:
   one group, internal note — the current split is already documented.)
