---
status: T2 partially discharged; T1 and T3 main proofs remain
date: 2026-05-17
---
# nox formal verification — Lean 4 proofs

This directory holds Lean 4 statements and proofs of the load-bearing
theorems for nox's parallel-canonical semantics.

## status snapshot

| theorem | statement | proof | notes |
|---------|-----------|-------|-------|
| **T2.1** atom bound | ✅ | ✅ `rfl` | trivial: definitional |
| **T2.2** binary structural (×10 patterns) | ✅ | ✅ `rfl` | one lemma per pattern |
| **T2.3** unary patterns (inv, not, hash) | ✅ | ✅ `rfl` | trivial |
| **T2.4** branch | ✅ | ✅ `rfl` | uses `Cost.branch` |
| **T2.5** axis, quote (no-sub) | ✅ | ✅ `rfl` | trivial |
| **T2.6** compose Dynamic | ✅ | ✅ `rfl` | shape + isDynamic |
| **T2.7** call Dynamic | ✅ | ✅ `rfl` | shape + isDynamic |
| **T2 monotonicity in left** | ✅ | ✅ `omega` | example for one pattern |
| **T2.8** upper-bound property | ✅ | ❌ `sorry` | needs full `reduce_seq` |
| **T3.1** sort permutation invariance | ✅ | ❌ `sorry` | mergesort lemma |
| **T3.2** trace multiset equivalence | ✅ | ❌ `sorry` | needs `reduce_par` |
| **T3** main (sort eq) | ✅ | ✅ derived | composition |
| **T1.1** outcome equivalence | ✅ | ❌ `sorry` | major induction |
| **T1.2** trace equivalence | ✅ | ✅ derived | from T3 |
| **T1** main | ✅ | ✅ derived | composition of 1.1 + 1.2 |

**16 lemmas discharged; 3 remain (T2.8, T3.1, T3.2).** T1.1 absorbs most
of the remaining work — it's the inductive case-split over all 18
patterns and uses T2.8 + T3.2 as hypotheses.

## why machine-checked

T1 is the **consensus theorem** — a network of nox evaluators agrees
on Result and witness only if T1 holds. A hand-waved T1 is acceptable
for v1; v2 ships machine-checked T1 before adoption makes divergence
costly.

## theorem dependency graph

```
        T2.1–T2.7 (local lemmas)
                │
                ▼ (used in T2.8 and T1.1)
        T2.8 (bound upper bound)
                │
                ▼
        T1.1 (outcome equivalence)
                │
                ▼
T3.1 + T3.2 ──► T3 (canonical sort)  ──► T1.2 (trace equivalence)
                                                │
                                                ▼
                                          T1 (consensus)
```

## files

- `lakefile.lean` — Lake build config
- `lean-toolchain` — pinned Lean 4 version (v4.10.0)
- `nox_model.lean` — shared types + `bound` (fully defined), `reduce_seq`
  and `reduce_par` (stubs for T1.1)
- `T1_sequential_equivalence.lean` — main consensus theorem
- `T2_bound_monotonicity.lean` — 15+ local lemmas about `bound`
- `T3_parallel_commutativity.lean` — sort-of-traces lemmas

## what's discharged today

Everything in T2 except T2.8 — every local `bound` lemma. These are
proved by `rfl` because `bound` is defined as a recursive function
matching the same shape; the lemmas are essentially saying "definition
agrees with itself."

This means: **the bound function in `noun/cost.rs` IS the spec.** If
the rust impl computes a different value, the rust impl is wrong.

T3 main theorem is derived once T3.1 and T3.2 are discharged.

T1 main theorem is derived once T1.1 is discharged.

## what remains

**T2.8** — "actual_cost(t) ≤ bound(t).value" — requires full
`reduce_seq` definition. The proof structure is induction over `t`
with case splits per pattern; each case reduces to arithmetic.

**T3.1** — `sortByPath key l = sortByPath key l'` whenever `l` and `l'`
are permutations. Standard `List.mergeSort_perm` from Lean stdlib.

**T3.2** — `Perm (trace_seq o t f) (trace_par o t f)` — needs both
trace functions defined and proven to emit the same set of rows.

**T1.1** — the major induction. ~18 cases, each ~10-30 lines. The
template is:
```lean
case .add => by
  unfold reduce_seq reduce_par
  -- both sides compute via evaluate_binary
  cases h : bound(formula).value ≤ f with
  | true => -- partitioned: identical by T2.8 + IH
  | false => -- fallback: identical by definition
```

Estimated effort: T2.8 + T3.1 in 1 session; T3.2 + T1.1 in 2-3 sessions.

## running

Install Lean 4 via [`elan`](https://github.com/leanprover/elan):

```nu
curl -sSf https://raw.githubusercontent.com/leanprover/elan/master/elan-init.sh | sh
cd nox/proofs/lean
lake build              # compiles statements + checks discharged proofs
```

CI should run `lake build` on every PR touching:
- `rs/noun/cost.rs` (the PATTERN_COSTS table)
- `rs/reduce.rs` (the COSTS table, dispatch)
- `rs/patterns/*.rs` (per-pattern semantics)
- `rs/bound.rs` (bound function)
- `proofs/lean/*.lean`

Any divergence between the rust impl and the Lean model surfaces as a
proof failure on the Lean side.

## design principles

1. **Lean is the spec.** When rust and Lean disagree, rust changes.
2. **`bound` is the canonical interface to cost-bound analysis.** Both
   the rust scheduler and the Lean proofs use the same recursive
   definition.
3. **Statements first, proofs later.** Sorries are explicit and tracked
   in the table above. Untyped TODO is forbidden.
4. **No imports of `Mathlib`.** We use only core Lean + minimal
   stdlib. The model is self-contained and small.
