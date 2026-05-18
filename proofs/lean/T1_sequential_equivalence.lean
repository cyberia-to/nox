/-
  T1 — sequential-equivalence theorem.

  The consensus property. States that classical sequential threading
  and bound-partitioned semantics produce the same Outcome AND the
  same canonical-sorted trace on every input.

  Status:
    T1.1 (outcome equivalence) DISCHARGED vacuously — reduce_par is
      definitionally reduce_seq, so both sides are identical by rfl.
      Real proof needed once reduce_par has bound-partitioned semantics.
    T1.2 (trace equivalence) DERIVED from T3.
    T1 main DERIVED from T1.1 + T1.2.

  Depends on (all now discharged or partially discharged):
    T2.8 — vacuously discharged (reduce_seq placeholder)
    T3.2 — fully discharged (Perm.nil; both traces are [])
    T3.1 — nil case discharged; non-nil cases sorry pending mergeSort theory
-/

import nox.proofs.lean.nox_model
import nox.proofs.lean.T2_bound_monotonicity
import nox.proofs.lean.T3_parallel_commutativity

namespace Nox

-- ═══════════════════════════════════════════════════════════════════
-- T1.1 — Outcome equivalence
-- ═══════════════════════════════════════════════════════════════════

/-- reduce_seq and reduce_par produce the identical Outcome.

    Status: VACUOUSLY DISCHARGED. Both `reduce_seq` and `reduce_par` are
    placeholders: `reduce_par` is defined as `exact reduce_seq object formula
    budget`, so they are definitionally equal and `rfl` closes the goal.

    When `reduce_par` is given real bound-partitioned semantics this proof
    MUST become the genuine inductive argument whose sketch is below.

    Inductive sketch for the real proof:

    BINARY STRUCTURAL ([op [a b]]):
      Case (bound formula).value ≤ f (partitioned path):
        By T2.8 on a:  actual_cost(a) ≤ bound(a).value, so reduce_seq a
                       with budget bound(a) returns Ok without halting.
        By T2.8 on b: similarly.
        By IH on a, b: results identical.
        used_seq = c + actual_cost(a) + actual_cost(b)
        used_par = c + (bound(a) - r_a) + (bound(b) - r_b)
                 = c + actual_cost(a) + actual_cost(b)  (T2.8 + arithmetic)
        ∴ remaining is identical.
      Case (bound formula).value > f: both fall back to reduce_seq. Identical.
    BRANCH / COMPOSE / CALL: analogous case splits using T2.8 per arm. -/
theorem outcome_equivalence
    (o : Noun) (t : Formula) (f : Nat) :
    reduce_seq o t f = reduce_par o t f := rfl

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
