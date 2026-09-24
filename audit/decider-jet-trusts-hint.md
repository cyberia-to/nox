---
tags: nox, audit, cyber
crystal-type: report
crystal-domain: comp
---
# decider jet trusts its hint, verifies nothing

date: 2026-09-24 · revision: origin/master 2612673 · property: launch #40 (a
verifier checks every argument it takes); relevant also to #5 (fold is a
commutative monoid, decide is O(1))

## finding

`specs/jets/decider.md` describes the decider jet as a working verifier: "the
system ships with 825 [constraints, the conservative Fiat-Shamir tier] and
transitions to 89 when algebraic FS soundness is formally verified." Its own
open-questions list (item 3) frames the remaining gap as "one-time formal
verification that 89-constraint CCS = full decider" — implying the 89/825
constraint check already runs and only its *equivalence* to a from-scratch
verifier is unconfirmed.

`rs/jets/decider.rs` does not run any constraint check at all. Reading the
full function (38 lines, reproduced in this audit's companion PR): it splits
`object` into `(proof_id, instance_id)`, charges `DECIDER_COST` (825, matching
the spec's conservative tier), then does exactly one thing —
`hints.provide(reduction, tag, object)` — and returns whatever the hint
returns, or the atom `0` if the hint is absent. `proof_id` and `instance_id`
are read out of `object` only to populate trace registers (`row.r[4]`,
`row.r[5]`); neither is used in any computation that could reject a hint's
answer. A `CallProvider` that returns `Some(1)` for every call makes the
decider accept every proof against every instance; the two existing tests
(`decider_with_null_calls_returns_zero`, `decider_exhausted_budget_halts`)
only exercise the no-hint and budget-exhausted paths and would not have
caught this — there is no test with a hint provider that returns a fixed
answer independent of its input.

The doc-comment already says this plainly: "Implementation note: For 0.1.0,
verification logic is delegated to the call provider (hints). A full CPU
verifier will be added in 0.2.0." This audit's contribution is not
discovering an undocumented bug — it is: (1) confirming empirically (build +
existing test run, this PR) that the code matches that comment exactly, with
zero independent verification logic anywhere in the jet; (2) naming the gap
between that comment and `specs/jets/decider.md`, which describes the
constraint check as already running; (3) connecting it to the property
registry, where neither row 5 nor row 40 currently mentions that the actual
on-chain decider — the gate core #2 (fold) calls "without this, settlement
verification is a DoS surface" — is a hint pass-through today, independent of
whatever zheng's own `decide()`/`SpartanVerifier` correctness looks like
(zheng#20, zheng#34 this session).

## why this matters beyond 0.1.0→0.2.0 bookkeeping

Per `specs/jets/README.md`, every genesis jet must have "an equivalent pure
Layer 1 program... committed in genesis BBG state" such that removing the jet
gives identical (if slower) results. For the decider specifically, no such
pure-nox reference program exists yet in `rs/patterns` (checked: no file
matches). So there is currently nothing for `decider_jet`'s hint to be
*equivalent to* even in principle — the jet-program equivalence open question
in `specs/jets/decider.md` (item 3) presupposes a working fast path whose
equivalence to a slow path needs proving; today there is neither a working
fast-path check nor a slow-path reference.

## remains

- `specs/jets/decider.md` gets a status line naming the 0.1.0 hint-based
  implementation explicitly, so a reader of the spec alone (not the code
  comment) knows the constraint check does not run yet — this PR adds it.
- The 0.2.0 work itself (a full CPU verifier) is the real fix and is out of
  this slice's scope: per `roadmap/decider-product.md` §4 ("circuit ownership
  follows the domain — the verifier is zheng's; nox holds the formula-hash
  anchor"), it is a call from `decider_jet` into zheng's `SpartanVerifier`
  (now degree-checked, `zheng#34` this session) over the actual HyperNova
  accumulator, not a nox-internal circuit.
- A pure-nox reference program for the decider (needed for the
  `specs/jets/README.md` jet-removal equivalence property) does not exist and
  is not scoped here either.
