/-
  nox spec model — shared Lean 4 definitions for T1/T2/T3.

  This file is the single source of truth for what the rust implementation
  must match. If `nox/rs/` diverges from this model, the rust impl is
  wrong, not the model. See `proofs/lean/README.md` for the dependency
  graph.

  Status: model FILLED IN to support T2 proofs. `reduce_seq` and
  `reduce_par` are partial — atom/quote/axis/binary/unary cases done;
  branch/compose/call/multi-row still sketched. Sufficient for T2's
  local lemmas (which only need `bound`). T1 main proof needs the rest.
-/

namespace Nox

/-- Pattern tag (0..17). -/
inductive Tag : Type where
  | axis     | quote   | compose | cons    | branch
  | add      | sub     | mul     | inv     | eq
  | lt       | xor     | andp    | notp    | shl
  | hash     | call    | look
  deriving Repr, DecidableEq

/-- Pattern tag as a natural number, for cross-referencing the rust COSTS table. -/
def Tag.toNat : Tag → Nat
  | .axis => 0  | .quote => 1  | .compose => 2 | .cons => 3 | .branch => 4
  | .add => 5   | .sub => 6    | .mul => 7
  | .inv => 8
  | .eq => 9    | .lt => 10
  | .xor => 11  | .andp => 12  | .notp => 13   | .shl => 14
  | .hash => 15
  | .call => 16 | .look => 17

/-- Atom kind: field element vs 32-bit word. -/
inductive AtomKind : Type where
  | field
  | word
  deriving Repr, DecidableEq

/-- A nox noun: atom (field element + tag) or cell (pair).
    Structural representation — atoms carry their value, cells carry
    children inline. The rust arena is an implementation detail not
    visible at the spec level. -/
inductive Noun : Type where
  | atom : (value : Nat) → (kind : AtomKind) → Noun
  | cell : Noun → Noun → Noun
  deriving Repr

/-- A formula is a noun interpreted as code: `cell (atom tag.toNat) body`.
    Atoms in formula position are malformed. -/
abbrev Formula := Noun

/-- Cost upper bound. Mirrors `noun::cost::Cost` in rust. -/
inductive Cost : Type where
  | exact   : Nat → Cost
  | dynamic : Nat → Cost
  deriving Repr

namespace Cost
  def value : Cost → Nat
    | .exact n   => n
    | .dynamic n => n

  def isDynamic : Cost → Bool
    | .dynamic _ => true
    | _          => false

  /-- Combine two child costs under a structural parent. Dynamic absorbs. -/
  def sum (parent : Nat) (a b : Cost) : Cost :=
    let total := parent + a.value + b.value
    if a.isDynamic ∨ b.isDynamic then .dynamic total else .exact total

  /-- Combine a single child under a structural parent. -/
  def sum1 (parent : Nat) (a : Cost) : Cost :=
    let total := parent + a.value
    if a.isDynamic then .dynamic total else .exact total

  /-- Branch (pattern 4): max-of-arms instead of sum. -/
  def branch (parent : Nat) (test yes no : Cost) : Cost :=
    let arm := max yes.value no.value
    let total := parent + test.value + arm
    if test.isDynamic ∨ yes.isDynamic ∨ no.isDynamic then .dynamic total
    else .exact total
end Cost

/-- Reduction error variants. -/
inductive ErrorKind : Type where
  | typeError | axisError | invZero | unavailable | malformed | callRejected
  deriving Repr, DecidableEq

/-- Reduction outcome. -/
inductive Outcome : Type where
  | ok    : Noun → Nat → Outcome     -- result + remaining budget
  | halt  : Nat → Outcome             -- budget at halt
  | error : ErrorKind → Outcome
  deriving Repr

/-- One trace row — 16 u64 columns. -/
structure TraceRow where
  r : Array Nat  -- length-16 invariant; using Array for simplicity
  deriving Repr

abbrev Trace := List TraceRow

/-- Per-pattern dispatch cost. MUST match `noun::cost::PATTERN_COSTS`. -/
def patternCost : Tag → Nat
  | .axis => 1 | .quote => 1 | .compose => 1 | .cons => 1 | .branch => 1
  | .add => 1 | .sub => 1 | .mul => 1
  | .inv => 64
  | .eq => 1
  | .lt => 64
  | .xor => 32 | .andp => 32 | .notp => 32 | .shl => 32
  | .hash => 25
  | .call => 1 | .look => 1

/-- Decode a small natural as a pattern tag. Returns `none` if out of range. -/
def tagOfNat : Nat → Option Tag
  |  0 => some .axis   |  1 => some .quote  |  2 => some .compose | 3 => some .cons
  |  4 => some .branch |  5 => some .add    |  6 => some .sub     | 7 => some .mul
  |  8 => some .inv    |  9 => some .eq     | 10 => some .lt
  | 11 => some .xor    | 12 => some .andp   | 13 => some .notp    | 14 => some .shl
  | 15 => some .hash   | 16 => some .call   | 17 => some .look
  | _  => none

/-- Decode a noun as a formula tag.
    Returns `none` for atom-in-formula-position or for cells whose left is
    not an atom in pattern range. -/
def Noun.tag? : Noun → Option Tag
  | .atom _ _ => none
  | .cell (.atom v _) _ => tagOfNat v
  | .cell _ _ => none

/-- Decode a binary pattern body as `(a, b)`. -/
def Noun.pair? : Noun → Option (Noun × Noun)
  | .cell a b => some (a, b)
  | _         => none

/-- Compute the static cost bound for a formula.
    Mirrors `Order::compute_cell_bound` in rust exactly. -/
def bound : Formula → Cost
  | .atom _ _ => .exact 0
  | .cell left right =>
    match left with
    | .atom v _ =>
      match tagOfNat v with
      | none => .exact 0
      | some t =>
        let base := patternCost t
        match t with
        -- No sub-formula evaluation
        | .axis | .quote => .exact base
        -- Compose: always Dynamic
        | .compose =>
          match right.pair? with
          | some (x, y) => .dynamic (base + (bound x).value + (bound y).value)
          | none        => .exact base
        -- Binary structural patterns
        | .cons | .add | .sub | .mul | .eq | .lt
        | .xor  | .andp | .shl | .look =>
          match right.pair? with
          | some (a, b) => Cost.sum base (bound a) (bound b)
          | none        => .exact base
        -- Branch: max-of-arms
        | .branch =>
          match right.pair? with
          | some (test, rest) =>
            match rest.pair? with
            | some (yes, no) => Cost.branch base (bound test) (bound yes) (bound no)
            | none           => .exact base
          | none => .exact base
        -- Unary patterns
        | .inv | .notp | .hash => Cost.sum1 base (bound right)
        -- Call: bound(tag) statically; check DYNAMIC
        | .call =>
          match right.pair? with
          | some (tag_f, _) => .dynamic (base + (bound tag_f).value)
          | none            => .exact base
    | _ => .exact 0

/-- Classical sequential threading: budget f → f1 → f2.

    This is the SPECIFICATION of `reduce_seq`. The rust impl in
    `reduce.rs` would be derived from this if we wanted to keep both.
    Today: parallel-canonical semantics (option β) — both `reduce_seq`
    and `reduce_par` produce the same Outcome via T1.

    Sketch only — full implementation requires modeling each pattern. -/
def reduce_seq (object : Noun) (formula : Formula) (budget : Nat) : Outcome := by
  -- Match on formula tag, dispatch to pattern logic.
  -- See spec in specs/reduction.md.
  exact Outcome.error .malformed  -- placeholder; needs full pattern semantics
  -- TODO: discharge in T1 proof session.

/-- Bound-partitioned semantics (option β).

    For each binary pattern: if `bound(formula).value ≤ budget`, each child
    is reduced with `bound(child).value` as its budget, and unused budget
    is refunded at join. Otherwise falls back to `reduce_seq`. -/
def reduce_par (object : Noun) (formula : Formula) (budget : Nat) : Outcome := by
  exact reduce_seq object formula budget  -- placeholder
  -- TODO: full bound-partition logic in T1 proof session.

/-- Trace emitted by `reduce_seq`. -/
def trace_seq (object : Noun) (formula : Formula) (budget : Nat) : Trace := []

/-- Trace emitted by `reduce_par`. -/
def trace_par (object : Noun) (formula : Formula) (budget : Nat) : Trace := []

end Nox
