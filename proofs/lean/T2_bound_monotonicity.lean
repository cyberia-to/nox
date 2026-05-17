/-
  T2 — bound monotonicity (local lemmas about how `bound` composes).

  Used by T1's inductive proof. Each lemma establishes the bound for
  one pattern shape; T1 case-splits on the pattern and applies the
  corresponding T2 lemma.

  Status: T2.1–T2.7 DISCHARGED (by `rfl` — definitional equality on
  `bound`). T2.8 (upper-bound property) requires full `reduce_seq`
  implementation; deferred to T1 session.
-/

import nox.proofs.lean.nox_model

namespace Nox

-- ═══════════════════════════════════════════════════════════════════
-- T2.1 — atom bound is Exact 0
-- ═══════════════════════════════════════════════════════════════════

theorem bound_atom (n : Nat) (k : AtomKind) :
    bound (.atom n k) = .exact 0 := by
  rfl

-- ═══════════════════════════════════════════════════════════════════
-- T2.2 — binary structural patterns: bound = sum of children + base
-- ═══════════════════════════════════════════════════════════════════

theorem bound_cons (a b : Formula) :
    bound (.cell (.atom 3 .field) (.cell a b))
      = Cost.sum 1 (bound a) (bound b) := by
  rfl

theorem bound_add (a b : Formula) :
    bound (.cell (.atom 5 .field) (.cell a b))
      = Cost.sum 1 (bound a) (bound b) := by
  rfl

theorem bound_sub (a b : Formula) :
    bound (.cell (.atom 6 .field) (.cell a b))
      = Cost.sum 1 (bound a) (bound b) := by
  rfl

theorem bound_mul (a b : Formula) :
    bound (.cell (.atom 7 .field) (.cell a b))
      = Cost.sum 1 (bound a) (bound b) := by
  rfl

theorem bound_eq (a b : Formula) :
    bound (.cell (.atom 9 .field) (.cell a b))
      = Cost.sum 1 (bound a) (bound b) := by
  rfl

theorem bound_lt (a b : Formula) :
    bound (.cell (.atom 10 .field) (.cell a b))
      = Cost.sum 64 (bound a) (bound b) := by
  rfl

theorem bound_xor (a b : Formula) :
    bound (.cell (.atom 11 .field) (.cell a b))
      = Cost.sum 32 (bound a) (bound b) := by
  rfl

theorem bound_and (a b : Formula) :
    bound (.cell (.atom 12 .field) (.cell a b))
      = Cost.sum 32 (bound a) (bound b) := by
  rfl

theorem bound_shl (a n : Formula) :
    bound (.cell (.atom 14 .field) (.cell a n))
      = Cost.sum 32 (bound a) (bound n) := by
  rfl

theorem bound_look (ns key : Formula) :
    bound (.cell (.atom 17 .field) (.cell ns key))
      = Cost.sum 1 (bound ns) (bound key) := by
  rfl

-- ═══════════════════════════════════════════════════════════════════
-- T2.3 — unary patterns: bound = sum1 base (bound child)
-- ═══════════════════════════════════════════════════════════════════

theorem bound_inv (a : Formula) :
    bound (.cell (.atom 8 .field) a)
      = Cost.sum1 64 (bound a) := by
  rfl

theorem bound_not (a : Formula) :
    bound (.cell (.atom 13 .field) a)
      = Cost.sum1 32 (bound a) := by
  rfl

theorem bound_hash (a : Formula) :
    bound (.cell (.atom 15 .field) a)
      = Cost.sum1 25 (bound a) := by
  rfl

-- ═══════════════════════════════════════════════════════════════════
-- T2.4 — branch: max-of-arms
-- ═══════════════════════════════════════════════════════════════════

theorem bound_branch (test yes no : Formula) :
    bound (.cell (.atom 4 .field) (.cell test (.cell yes no)))
      = Cost.branch 1 (bound test) (bound yes) (bound no) := by
  rfl

-- ═══════════════════════════════════════════════════════════════════
-- T2.5 — no-sub patterns (axis, quote): bound is exact base
-- ═══════════════════════════════════════════════════════════════════

theorem bound_axis (addr : Formula) :
    bound (.cell (.atom 0 .field) addr) = .exact 1 := by
  rfl

theorem bound_quote (c : Formula) :
    bound (.cell (.atom 1 .field) c) = .exact 1 := by
  rfl

-- ═══════════════════════════════════════════════════════════════════
-- T2.6 — compose: always Dynamic
-- ═══════════════════════════════════════════════════════════════════

theorem bound_compose_dynamic (x y : Formula) :
    (bound (.cell (.atom 2 .field) (.cell x y))).isDynamic = true := by
  -- bound expands to `.dynamic (1 + ...)` per the .compose case in the
  -- definition; isDynamic returns true.
  rfl

theorem bound_compose_value (x y : Formula) :
    (bound (.cell (.atom 2 .field) (.cell x y))).value
      = 1 + (bound x).value + (bound y).value := by
  rfl

-- ═══════════════════════════════════════════════════════════════════
-- T2.7 — call: always Dynamic
-- ═══════════════════════════════════════════════════════════════════

theorem bound_call_dynamic (tag check : Formula) :
    (bound (.cell (.atom 16 .field) (.cell tag check))).isDynamic = true := by
  rfl

theorem bound_call_value (tag check : Formula) :
    (bound (.cell (.atom 16 .field) (.cell tag check))).value
      = 1 + (bound tag).value := by
  rfl

-- ═══════════════════════════════════════════════════════════════════
-- Derived: monotonicity under sub-formula substitution
-- ═══════════════════════════════════════════════════════════════════

/-- If we substitute a sub-formula `a` with a heavier one `a'`
    (`bound a'.value ≥ bound a.value`), the parent's bound only grows.

    Used in T1's compositional reasoning. -/
theorem bound_monotone_in_left (a a' b : Formula)
    (hge : (bound a').value ≥ (bound a).value) :
    (bound (.cell (.atom 5 .field) (.cell a' b))).value
      ≥ (bound (.cell (.atom 5 .field) (.cell a  b))).value := by
  -- both reduce to `Cost.sum 1 (bound _) (bound b)` via T2.bound_add.
  -- Cost.sum is monotone in its arguments (their value field).
  rw [bound_add, bound_add]
  simp [Cost.sum, Cost.value]
  -- The only thing that changes is `bound a` vs `bound a'`; goal becomes
  -- `1 + bound(a').value + bound(b).value ≥ 1 + bound(a).value + bound(b).value`
  -- which follows from hge by linear arithmetic.
  omega

-- ═══════════════════════════════════════════════════════════════════
-- T2.8 — upper bound on actual cost
-- ═══════════════════════════════════════════════════════════════════

/-- For any program where `reduce_seq` returns Ok(_, remaining),
    the consumed budget (f - remaining) is at most `bound(formula).value`.

    This is the load-bearing property for partitioned scheduling:
    if each child is given `bound(child)` budget, no child halts internally
    (since actual_cost ≤ bound by this theorem applied to each child).

    Status: STUB. Proof requires full `reduce_seq` definition. Discharge
    in T1 session along with the rest of `reduce_seq`. -/
theorem bound_upper_bounds_actual_cost
    (o : Noun) (t : Formula) (f : Nat)
    (n : Noun) (r : Nat)
    (h : reduce_seq o t f = Outcome.ok n r) :
    f - r ≤ (bound t).value := by sorry

end Nox
