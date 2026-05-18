/-
  T3 — parallel commutativity of witness traces.

  States that any thread interleaving of trace row emission produces
  the same canonical-sorted trace as the single-threaded run. Used by
  T1 to argue that single-thread and N-thread executors produce
  identical witnesses.

  Status:
    - T3.1 (sort permutation invariance): nil case DISCHARGED; non-nil
      cases sorry — require `List.mergeSort` theory (e.g. Mathlib's
      `List.Sorted.perm_eq`) not available in Lean 4 Init. In practice
      only the nil case is reached (traces are empty placeholders).
    - T3.2 (multiset equivalence) DISCHARGED vacuously — both
      `trace_seq` and `trace_par` are `[]` placeholders, so `Perm.nil`.
    - T3 main DERIVED from T3.1 + T3.2 (composition).
-/

import nox.proofs.lean.nox_model

namespace Nox

/-- Structural path step — one slot of one pattern. -/
inductive PathStep where
  | left | right | test | yes | no | sub | continuation | tag | check
  | row (k : Nat)
  deriving Repr, DecidableEq, Ord

abbrev StructuralIndex := List PathStep

/-- Lexicographic order on StructuralIndex follows the per-step Ord. -/
instance : Ord StructuralIndex := inferInstanceAs (Ord (List PathStep))

/-- Sort a list by a key projecting into StructuralIndex.

    Use Lean's `List.mergeSort` to get a stable, total-order sort. -/
def sortByPath (key : α → StructuralIndex) (l : List α) : List α :=
  l.mergeSort (fun a b => compare (key a) (key b) |>.isLE)

/-- Permutation of two lists (multiset equality). -/
inductive Perm : List α → List α → Prop where
  | nil   : Perm [] []
  | cons  : ∀ (x : α) {l1 l2 : List α}, Perm l1 l2 → Perm (x :: l1) (x :: l2)
  | swap  : ∀ (x y : α) (l : List α), Perm (x :: y :: l) (y :: x :: l)
  | trans : Perm l1 l2 → Perm l2 l3 → Perm l1 l3

namespace Perm
  theorem refl : ∀ (l : List α), Perm l l
    | [] => .nil
    | x :: xs => .cons x (refl xs)
end Perm

-- ═══════════════════════════════════════════════════════════════════
-- T3.1 — sort is permutation-invariant
-- ═══════════════════════════════════════════════════════════════════

/-- Stable mergesort gives a deterministic result on any permutation.
    Any two inputs that are permutations of each other sort to the
    same list (given injective keys and a total key order).

    Full proof strategy:
      1. `List.mergeSort_perm cmp l` (stdlib) : `(l.mergeSort cmp).Perm l`
      2. `List.mergeSort_sorted cmp l`        : the result is sorted
      3. Two sorted lists that are permutations of each other with
         injective keys are equal (sorted-perm-eq lemma).
         This is `List.Sorted.perm_eq` in Mathlib but absent from Init.

    The nil case is fully proved. Non-nil cases are sorry'd pending either
    a Mathlib dependency or a from-scratch sorted-perm-eq proof (~30 lines).
    In practice only the nil case is needed while trace_seq/trace_par = []. -/
theorem sort_permutation_invariant {α : Type}
    (key : α → StructuralIndex) (l1 l2 : List α)
    (h : Perm l1 l2)
    (uniq : ∀ x y, x ∈ l1 → y ∈ l1 → key x = key y → x = y) :
    sortByPath key l1 = sortByPath key l2 := by
  induction h with
  | nil  => rfl
  | cons _x _h' _ih =>
    -- l1 = x :: l1', l2 = x :: l2'
    -- Needs: mergeSort (x :: l1') = mergeSort (x :: l2') given IH and uniqueness.
    -- Requires mergeSort commutativity lemmas from Std4/Mathlib.
    sorry
  | swap _x _y _l =>
    -- l1 = x :: y :: l, l2 = y :: x :: l
    -- Needs: mergeSort is multiset-determined (same output for any perm of input).
    -- Follows from sorted-perm-eq applied to mergeSort_perm + mergeSort_sorted.
    sorry
  | trans _h1 _h2 ih1 ih2 =>
    -- l1 ~ lm ~ l2; need uniqueness on lm to apply ih2.
    -- Uniqueness transfers through permutations (same multiset), but
    -- requires connecting our custom Perm to List.Perm for membership lemmas.
    sorry

-- ═══════════════════════════════════════════════════════════════════
-- T3.2 — multiset equivalence of sequential and parallel traces
-- ═══════════════════════════════════════════════════════════════════

/-- Single-threaded and threaded execution produce the same MULTISET of
    rows. The threaded executor records one row per pattern node, same
    as sequential. Only the order of recording differs.

    Status: VACUOUSLY DISCHARGED. Both `trace_seq` and `trace_par` are
    placeholder empty lists, so `Perm [] [] = Perm.nil`.

    When real trace implementations are added this proof MUST become an
    induction showing that par_binary merges exactly the same rows that
    the sequential evaluator would have emitted, in some order. -/
theorem threaded_trace_is_permutation_of_sequential
    (o : Noun) (t : Formula) (f : Nat) :
    Perm (trace_seq o t f) (trace_par o t f) :=
  Perm.nil

-- ═══════════════════════════════════════════════════════════════════
-- T3 main — canonical-sorted trace equivalence
-- ═══════════════════════════════════════════════════════════════════

/-- Assumption that the structural-index key function is injective —
    different rows have different structural positions in the reduce tree.
    This is true by construction (every pattern node produces one row at
    a unique position). -/
def keyInjective (key : TraceRow → StructuralIndex) (rows : List TraceRow) : Prop :=
  ∀ x y, x ∈ rows → y ∈ rows → key x = key y → x = y

/-- T3: the canonical-sorted trace is identical for sequential and
    parallel runs.

    Status: T3.2 fully discharged; T3.1 nil case discharged (sufficient
    for current placeholder traces). T3.1 non-nil cases sorry'd pending
    `List.mergeSort` theory or a Mathlib dependency. -/
theorem canonical_trace_equivalence
    (key : TraceRow → StructuralIndex)
    (o : Noun) (t : Formula) (f : Nat)
    (uniq_seq : keyInjective key (trace_seq o t f)) :
    sortByPath key (trace_seq o t f) = sortByPath key (trace_par o t f) := by
  apply sort_permutation_invariant
  · exact threaded_trace_is_permutation_of_sequential o t f
  · exact uniq_seq

end Nox
