// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! Parallel reduction API and structural-index tracing.
//!
//! ## option β: one canonical semantics
//!
//! Nox commits to a single canonical reduction semantics — bound-partitioned
//! evaluation (see `specs/reduction.md §parallel reduction`). The
//! single-threaded [`reduce`](crate::reduce::reduce) function already
//! produces the parallel-canonical witness because every binary / unary
//! pattern uses [`crate::reduce::evaluate_binary`] /
//! [`crate::reduce::evaluate_unary`] which thread budget via the partition
//! rule, not classical f → f1 → f2.
//!
//! [`reduce_parallel`] is the explicit name for the parallel-canonical
//! entry point. It is currently implemented as a single-threaded driver
//! identical to `reduce()`. The semantics is already parallel-safe; only
//! the threaded executor remains as a future implementation choice.
//!
//! ## what changes when actual threading lands
//!
//! - Two changes only:
//!   1. `evaluate_binary` spawns its two `evaluate` calls on separate
//!      threads when `can_partition` is true.
//!   2. `Order::cell` / `Order::atom` become thread-safe (atomic hash-cons
//!      table or per-thread scratch with merge-at-join).
//!
//! - **Zero changes to observable Result or per-row trace.** This is T1
//!   (sequential-equivalence): the parallel-canonical witness is identical
//!   regardless of scheduling. Single-threaded callers and N-threaded
//!   callers produce the same trace and the same Outcome on every input.
//!
//! ## structural index for canonical row sort
//!
//! When threads run concurrently their `TraceRow::record()` calls interleave.
//! For consensus, the final witness must be in canonical order. Every row
//! carries a [`StructuralIndex`] — its position in the reduce tree —
//! which gives a total order independent of execution thread.
//!
//! - root reduce: index `()`.
//! - left sub-formula of a binary pattern at index `p`: `p.l`.
//! - right sub-formula: `p.r`.
//! - test of a branch: `p.t`. yes-arm: `p.y`. no-arm: `p.n`.
//! - single sub of a unary pattern: `p.0`.
//! - tag of call: `p.tag`. check of call: `p.check`.
//! - bound-multi-row rows (inv/lt/xor/and/not/shl/hash): suffix `.row[k]`
//!   for the k-th row of that pattern's block.
//!
//! Sorting rows lexicographically by structural index produces the
//! canonical witness ordering. This is the foundation for T3 (parallel
//! commutativity).

use crate::call::CallProvider;
use crate::noun::{NounId, Order};
use crate::reduce::{reduce, Outcome};
use crate::trace::Tracer;

/// Parallel-canonical reduction.
///
/// Observationally identical to [`crate::reduce`] on every `(o, t, f)` input.
/// The current implementation is single-threaded — the semantic infrastructure
/// (bound-partitioned `evaluate_binary` / `evaluate_unary`) is already in
/// place. A future threaded executor would change scheduling without
/// changing the Result or the trace.
///
/// ## sequential-equivalence theorem (T1)
///
/// For every `(object, formula, budget)`:
///
/// `reduce(o, t, f) == reduce_parallel(o, t, f)`
///
/// on every observable field. See `specs/props/parallel-reduction.md` for
/// the formal statement; `proofs/lean/T1.lean` for the machine-check
/// target.
pub fn reduce_parallel<const N: usize, T: Tracer>(
    order: &mut Order<N>,
    object: NounId,
    formula: NounId,
    budget: u64,
    hints: &dyn CallProvider<N>,
    tracer: &mut T,
) -> Outcome {
    reduce(order, object, formula, budget, hints, tracer)
}

/// Structural position of a TraceRow within the reduce tree.
///
/// Used to sort rows into canonical order regardless of which thread
/// recorded them. The encoding is a path from the root reduce call;
/// each step picks a child slot of the current pattern.
///
/// `Branch` is variable-arity to keep the path representation small
/// without heap allocation in the common case. Path depth is bounded by
/// `crate::reduce::MAX_DEPTH = 1000`; the inline buffer covers
/// shallow paths and the spill array handles deeper paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PathStep {
    /// Left sub-formula of a binary pattern (3, 5–7, 9–12, 14, 17).
    Left,
    /// Right sub-formula of a binary pattern.
    Right,
    /// Branch (4) test sub-formula.
    Test,
    /// Branch (4) yes-arm sub-formula.
    Yes,
    /// Branch (4) no-arm sub-formula.
    No,
    /// Unary pattern (8, 13, 15) sole sub-formula.
    Sub,
    /// Compose (2) third reduce — sequential continuation.
    Continue,
    /// Call (16) tag-formula evaluation.
    Tag,
    /// Call (16) check-formula evaluation.
    Check,
    /// k-th internal row of a bound-multi-row pattern (8, 10, 11, 12, 13, 14, 15).
    /// `k` is the row counter (0..rounds_total).
    Row(u16),
}

/// Canonical structural index for a row. Sort by this to canonicalize.
///
/// Reserved for future use — Phase 3 (next session) wires this through
/// `Tracer::record` and adds `canonicalize(&mut Vec<TraceRow>)`. Until
/// then, the single-threaded executor emits rows in canonical order by
/// construction (no shuffling), so explicit sorting is not required.
#[derive(Debug, Clone, Default)]
pub struct StructuralIndex {
    /// Path from root. Index 0 is the immediate child of root; deeper
    /// indices follow the path.
    path: alloc::vec::Vec<PathStep>,
}

impl StructuralIndex {
    /// Empty path = root reduce call.
    pub fn root() -> Self { Self::default() }

    /// Push a step onto the path.
    pub fn push(&mut self, step: PathStep) { self.path.push(step); }

    /// Pop the last step (for unwinding).
    pub fn pop(&mut self) -> Option<PathStep> { self.path.pop() }

    /// Read-only view of the path.
    pub fn path(&self) -> &[PathStep] { &self.path }
}

extern crate alloc;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::call::NullCalls;
    use crate::noun::{Order, Tag};
    use crate::trace::{NoTrace, VecTrace};
    use nebu::Goldilocks;

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    /// T1 (empirical): reduce and reduce_parallel produce identical Outcome
    /// on a deterministic Layer 1 program.
    #[test]
    fn reduce_parallel_matches_reduce_on_add() {
        let formula = |ar: &mut Order<256>| {
            let t5 = ar.atom(g(5), Tag::Field).unwrap();
            let t1 = ar.atom(g(1), Tag::Field).unwrap();
            let v3 = ar.atom(g(3), Tag::Field).unwrap();
            let v5 = ar.atom(g(5), Tag::Field).unwrap();
            let qa = ar.cell(t1, v3).unwrap();
            let qb = ar.cell(t1, v5).unwrap();
            let body = ar.cell(qa, qb).unwrap();
            ar.cell(t5, body).unwrap()
        };

        let mut ar1 = Order::<256>::new();
        let obj1 = ar1.atom(g(0), Tag::Field).unwrap();
        let f1 = formula(&mut ar1);
        let r1 = reduce(&mut ar1, obj1, f1, 1000, &NullCalls, &mut NoTrace);

        let mut ar2 = Order::<256>::new();
        let obj2 = ar2.atom(g(0), Tag::Field).unwrap();
        let f2 = formula(&mut ar2);
        let r2 = reduce_parallel(&mut ar2, obj2, f2, 1000, &NullCalls, &mut NoTrace);

        match (r1, r2) {
            (Outcome::Ok(a, ba), Outcome::Ok(b, bb)) => {
                assert_eq!(ar1.atom_value(a).unwrap().0, ar2.atom_value(b).unwrap().0);
                assert_eq!(ba, bb, "remaining budget must match");
            }
            (x, y) => panic!("Outcome variants diverged: {:?} vs {:?}", x, y),
        }
    }

    /// T1 (trace equivalence): the per-row trace produced by reduce and
    /// reduce_parallel are identical (single-threaded scheduling already
    /// matches the canonical order).
    #[test]
    fn reduce_parallel_trace_matches_reduce_on_hash() {
        let formula = |ar: &mut Order<256>| {
            let t15 = ar.atom(g(15), Tag::Field).unwrap();
            let t1 = ar.atom(g(1), Tag::Field).unwrap();
            let v = ar.atom(g(7), Tag::Field).unwrap();
            let body = ar.cell(t1, v).unwrap();
            ar.cell(t15, body).unwrap()
        };

        let mut ar1 = Order::<256>::new();
        let obj1 = ar1.atom(g(0), Tag::Field).unwrap();
        let f1 = formula(&mut ar1);
        let mut tr1 = VecTrace::default();
        reduce(&mut ar1, obj1, f1, 10_000, &NullCalls, &mut tr1);

        let mut ar2 = Order::<256>::new();
        let obj2 = ar2.atom(g(0), Tag::Field).unwrap();
        let f2 = formula(&mut ar2);
        let mut tr2 = VecTrace::default();
        reduce_parallel(&mut ar2, obj2, f2, 10_000, &NullCalls, &mut tr2);

        assert_eq!(tr1.0.len(), tr2.0.len(), "trace lengths must match");
        for (i, (a, b)) in tr1.0.iter().zip(tr2.0.iter()).enumerate() {
            assert_eq!(a.r(), b.r(), "row {} differs", i);
        }
    }

    #[test]
    fn structural_index_push_pop() {
        let mut idx = StructuralIndex::root();
        idx.push(PathStep::Left);
        idx.push(PathStep::Row(7));
        assert_eq!(idx.path(), &[PathStep::Left, PathStep::Row(7)]);
        assert_eq!(idx.pop(), Some(PathStep::Row(7)));
        assert_eq!(idx.path(), &[PathStep::Left]);
    }

    #[test]
    fn path_step_order_is_total() {
        // Reserved variants must sort: Left < Right < Test < Yes < No < Sub < Continue < Tag < Check < Row(_).
        let mut path = [
            PathStep::Row(0),
            PathStep::Check,
            PathStep::Tag,
            PathStep::Continue,
            PathStep::Sub,
            PathStep::No,
            PathStep::Yes,
            PathStep::Test,
            PathStep::Right,
            PathStep::Left,
        ];
        path.sort();
        assert_eq!(
            path,
            [
                PathStep::Left, PathStep::Right, PathStep::Test, PathStep::Yes,
                PathStep::No, PathStep::Sub, PathStep::Continue, PathStep::Tag,
                PathStep::Check, PathStep::Row(0),
            ]
        );
    }
}
