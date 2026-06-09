---
status: draft
date: 2026-06-09
author: jet taxonomy review session
---

# decider — the verification product

## Abstract

The genesis jet registry (`specs/jets/README.md`) labels each group with an
algebra, which makes algebra look like the organizing principle. It is not.
The registry organizes by **product**: each group delivers one tangible,
complete, powerful capability. Algebra is the substrate chosen to make that
product cheap — downstream, not the key.

Read through the product lens, `recursion` is not a product. It is an
ingredient shelf: `poly_eval`, `ntt`, `fri_fold`, `merkle_verify` are
primitives, not a capability you can name and hand to someone. The
`decider` is the product they compose into — "verify all chain history in
240 bytes, ~89 constraints, ~100 ns." That is the most tangible, complete,
powerful output in the whole registry.

This proposal: dissolve `recursion` as a top-level group and fold its
verification-critical ingredient (`poly_eval`) into a single product group,
**decider**. Name the group for the product, not the tools. Ownership is a
separate axis — the decider circuit is zheng's, exactly as the `hash`
product is hemera's; nox holds the formula-hash anchor.

This supersedes the earlier framing (the file once argued the F_p split
"broke an algebra rule"). There is no algebra rule. The earlier draft also
had the hierarchy inverted — it kept `recursion` as the group and treated
the decider as a misfiled sibling. Under the product principle the reverse
is true: the decider is the group, recursion is its parts.

## 1. The organizing principle is product, not algebra

The algebra label is a confound. Three falsifications:

| group | algebra | PCS | the product |
|---|---|---|---|
| hash | **all** | — | content identity (universal anchor) |
| recursion | **F_p** | **Brakedown** | — (no product; ingredient shelf) |
| state | **F_p** | **Brakedown** | the cybergraph state machine |
| decider | **F_p** | **Brakedown** | verify all history in 240 bytes |
| binary-tower | F₂ | Binius | quantized inference |
| polynomial-ring | R_q | Ikat | FHE bootstrapping |
| isogeny-curves | F_q | Porphyry | post-quantum privacy crypto |
| tropical-semiring | (min,+) | Assayer | combinatorial optimization |

1. **`recursion`, `state`, `decider` share `(F_p, Brakedown)` exactly** yet
   are three groups. If algebra organized the registry they would be one.
   They are separate because they are three different products (and one
   non-product).
2. **`hash` has algebra "all"** — not an algebra. It cannot sit in an
   algebra taxonomy. It sits in a product taxonomy fine: its product is
   identity.
3. **tropical is explicitly not a separate instantiation** — its spec says
   it "decompose[s] to existing patterns (branch + lt)." `(min,+)` is a
   semiring *view*, not a `nox<Algebra>` boundary. The product
   (optimization) is real; the algebra label is aspirational.

For the four exotic groups algebra and product *coincide* — because the
algebra was chosen to make the product cheap (F₂ for bits, R_q for FHE).
Causal direction is product → algebra. Algebra is distinctive for 4 of 8
rows, which is why it masquerades as the key.

A product, in this registry, is: a **tangible, clear output** you can name
and grasp; **powerful** (high leverage — large speedup or a capability that
does not otherwise exist); **complete** (a whole capability, not a
fragment). The group names are the tell — every one names a product
(inference, FHE, privacy, optimization, state, identity, verification) —
except `recursion`, which names a technique.

## 2. recursion is ingredients; the decider is the product

The recursion group's own spec splits its members by role
(`specs/jets/recursion.md`):

- `poly_eval` — verifier-critical (the Brakedown opening).
- `merkle_verify`, `fri_fold`, `ntt` — "retained for cross-system
  interoperability and domain-specific acceleration," explicitly **not used
  by the Brakedown verifier**.

None of the four is a product. You cannot hand someone "poly_eval" as a
capability — it is a tool. The capability they compose into is the decider:
verify the HyperNova accumulator (~200 bytes folding every block from
genesis) in one step.

The decider is a **composite of its ingredients, frozen as one circuit.**
The number proves it: the recursion spec gives the canonical Brakedown
verifier as "~825 constraints (CCS jet + batch)"; the decider's
conservative tier is "89 + 736 = 825." Same circuit. Its 89-constraint
breakdown (`specs/jets/decider.md`):

```
sumcheck replay:         20 constraints
CCS evaluation:          34 constraints
Brakedown (batched):     35 constraints   ← this IS poly_eval, fused
hemera:                   0 constraints   (algebraic Fiat-Shamir)
────────────────         ──────────────
total:                   89 constraints
```

The 35-constraint "Brakedown (batched)" step is `poly_eval`'s operation —
multilinear opening — fused and specialized into the monolith. `poly_eval`
is the decider's core ingredient. The decider is the product.

## 3. Where the other recursion jets go

The product lens surfaces a question the old grouping hid: if the decider
is the verification product, what product do `ntt`, `fri_fold`,
`merkle_verify` serve?

- **`ntt`** — a shared compute primitive. It is the core ingredient of the
  FHE product (`polynomial-ring`: `ntt_batch`, `blind_rotate`) and of ring
  multiplication generally. Like `hash`, it is cross-cutting. Attribute it
  to the product it primarily serves (FHE / ring), or keep it as a named
  shared primitive — not under "verification."
- **`fri_fold`, `merkle_verify`** — interop ingredients. Brakedown is
  Merkle-free and FRI-free, so **no genesis product uses them.** They are
  "retained for cross-system interoperability." Under the product
  principle that is a flag: freezing primitives at genesis (A3,
  append-only) that no genesis product consumes is cost with no shipped
  capability behind it. They may belong in a clearly-labeled interop
  primitives shelf, or be deferred out of genesis entirely. See open
  questions.

This does not weaken the decider; it clarifies that "recursion" was bundling
one verification ingredient (`poly_eval`) with three unrelated primitives
under a technique name.

## 4. Ownership is a separate axis

Grouping (how the spec organizes products) and ownership (which repo
implements) are orthogonal. The precedent is `hash`: its own product group,
owned by hemera, anchored in nox by formula hash. The boundary rule applied
across this session:

> the jet wrapper (data I/O, TraceRow, budget) lives in nox; the
> implementation lives in its domain repo.

- `hash` → hemera   ·   `merkle_verify` → hemera   ·   `fri_fold` → lens
- `ntt`, `poly_eval` → strata-compute / honeycrisp (acpu)
- state product → bbg ([[state-jets-redesign]])

The decider is the verifier of the proof system, and zheng *is* the proof
system (SuperSpartan + Brakedown). So the decider circuit is zheng's. nox
keeps the formula-hash anchor so the jet is recognized at reduction time;
zheng owns the CCS circuit, the three optimizations, and the soundness
proofs. The decider is `hash` for verification: nox anchors, zheng
implements.

## 5. Proposed structure

```
decider (verification product)     zheng owns the circuit; nox anchors
  ingredient:  poly_eval           strata-compute / honeycrisp implement; nox anchors
  product:     decider circuit     = poly_eval + sumcheck + CCS, fused

shared / interop primitives        (not a product group)
  ntt              → attribute to FHE product; strata/honeycrisp implement
  fri_fold         → interop; flag for genesis inclusion review
  merkle_verify    → interop; hemera implements; flag for review
```

Concretely:

1. Dissolve `recursion` as a top-level group in `specs/jets/README.md`.
2. The `decider` group becomes the verification product: `poly_eval` (core
   ingredient) + the decider circuit (the product). `specs/jets/decider.md`
   states the formula, its hash, and the contract, and points to zheng for
   the circuit; the constraint breakdown and the three optimizations migrate
   to `zheng/specs/`.
3. Reassign `ntt` to the FHE/ring product or a named shared-primitive shelf.
4. Park `fri_fold` and `merkle_verify` in an interop shelf with a genesis
   inclusion decision (open question 2).
5. `rs/jets/poly_eval.rs`, `rs/jets/decider.rs` keep their wrappers; wire
   the decider wrapper to zheng's verifier when exposed (mirrors the
   hemera/lens/acpu wiring already done for the other jets).

## 6. What does not change

- The decider stays a genesis jet, recognized by exact formula match.
- `poly_eval` stays a nox-anchored jet and stays verifier-critical.
- The 89/825 two-tier story is unchanged — it just lives in zheng's spec.

## 7. Open questions

1. Genesis-digest source of truth: does the decider's formula hash live in
   nox's `compute_genesis_digests` or in zheng's genesis manifest? The hash
   must be identical either way. (Shared with [[state-jets-redesign]] open
   question 4 — both proposals move circuit ownership out of nox while
   keeping anchors.)
2. `fri_fold` and `merkle_verify` serve no genesis product (Brakedown is
   Merkle-free, FRI-free). Freeze them at genesis as interop primitives, or
   defer them post-genesis? Freezing has a permanent A3 cost; deferring
   risks an interop gap. Needs a concrete interop requirement to decide.
3. Is `ntt` better filed under the FHE product (its primary consumer) or as
   a standalone shared primitive alongside `hash`? Both `hash` and `ntt`
   are cross-cutting; the registry currently has no "shared primitive"
   category.
