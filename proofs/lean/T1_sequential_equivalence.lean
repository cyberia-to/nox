/-
  T1 — sequential-equivalence theorem.

  The consensus property. States that classical sequential threading
  and bound-partitioned semantics produce the same Outcome AND the
  same canonical-sorted trace on every input.

  Status: STATEMENT + COMPOSITION. T1.1 (outcome) and T1.2 (trace)
  are stubs; T1 main composes them. T1.1 is the load-bearing
  inductive theorem — its proof is the major remaining work.

  Depends on:
    T2.8 (bound upper bound)             — for partitioned cases
    T3.2 (multiset equivalence)          — for trace equivalence
    T3.1 (sort permutation invariance)   — for canonical ordering
-/

import nox.proofs.lean.nox_model
import nox.proofs.lean.T2_bound_monotonicity
import nox.proofs.lean.T3_parallel_commutativity

namespace Nox

-- ═══════════════════════════════════════════════════════════════════
-- T1.1 — Outcome equivalence
-- ═══════════════════════════════════════════════════════════════════

/-- reduce_seq and reduce_par produce the identical Outcome.

    Inductive cases per formula shape:

    BASE — atom or trivial formulas:
      reduce_seq atom = reduce_par atom = Outcome.error .malformed
      reduce_seq [1 c] = reduce_par [1 c] = Outcome.ok c (f-1)
      reduce_seq [0 a] = reduce_par [0 a] = ...   (both walk the same path)

    BINARY STRUCTURAL ([op [a b]]):
      Case (bound formula).value ≤ f (partitioned path):
        By T2.8 on a:  actual_cost(a) ≤ bound(a).value, so reduce_seq a
                       with budget ≥ bound(a) returns Ok.
        By T2.8 on b: similarly.
        By IH on a, b: results identical, remaining differs by `used`.
        used_seq = c + actual_cost(a) + actual_cost(b)
        used_par = c + (bound(a) - r_a) + (bound(b) - r_b)
                = c + actual_cost(a) + actual_cost(b)   (by T2.8 + arithmetic)
        ∴ remaining is identical.

      Case (bound formula).value > f (fallback path):
        Both call sequential_fallback (= reduce_seq). Identical.

    BRANCH ([4 [test [yes no]]]):
      reduce_seq evaluates test, then chosen arm.
      reduce_par evaluates test under partition, then chosen arm.
      By IH: test result identical → selector identical → chosen identical.
      By IH on chosen arm: result identical. Slack accounting matches.

    COMPOSE ([2 [x y]]):
      bound is Dynamic, so partition gates on bound(x) + bound(y) + 1 ≤ f.
      First phase: reduce x, reduce y. By IH, identical results.
      Continuation: reduce(rx, ry) — both use sequential, identical.

    CALL ([16 [tag check]]):
      Same as compose: partition on tag, sequential check.

    MULTI-ROW PATTERNS (8 inv, 10 lt, 11-14 bitwise, 15 hash):
      Row emission is in the pattern body, shared by both reducers.
      Identical row sequences by construction.

    Status: STUB. Discharge in a dedicated session. -/
theorem outcome_equivalence
    (o : Noun) (t : Formula) (f : Nat) :
    reduce_seq o t f = reduce_par o t f := by sorry

-- ═══════════════════════════════════════════════════════════════════
-- T1.2 — Trace equivalence (after canonical sort)
-- ═══════════════════════════════════════════════════════════════════

/-- The per-row trace produced by the two reducers, when sorted by
    structural index, are identical.

    DERIVED from T3 (canonical_trace_equivalence). -/
theorem trace_equivalence
    (key : TraceRow → StructuralIndex)
    (o : Noun) (t : Formula) (f : Nat)
    (uniq : keyInjective key (trace_seq o t f)) :
    sortByPath key (trace_seq o t f) = sortByPath key (trace_par o t f) :=
  canonical_trace_equivalence key o t f uniq

-- ═══════════════════════════════════════════════════════════════════
-- T1 (main) — observational equivalence
-- ═══════════════════════════════════════════════════════════════════

/-- T1: full observational equivalence on Outcome AND canonical trace.

    This is the consensus theorem. Any conforming implementation of
    the nox spec produces the identical Result and witness on every
    input, regardless of execution strategy. -/
theorem T1
    (key : TraceRow → StructuralIndex)
    (o : Noun) (t : Formula) (f : Nat)
    (uniq : keyInjective key (trace_seq o t f)) :
    reduce_seq o t f = reduce_par o t f ∧
    sortByPath key (trace_seq o t f) = sortByPath key (trace_par o t f) :=
  ⟨outcome_equivalence o t f, trace_equivalence key o t f uniq⟩

end Nox
