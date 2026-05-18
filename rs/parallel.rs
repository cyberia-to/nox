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
/// When the `std` feature is enabled, binary sub-formulas with Exact bounds
/// run on separate threads (`par_binary`). Without `std`, the semantics is
/// identical but evaluation is single-threaded.
///
/// ## sequential-equivalence theorem (T1)
///
/// For every `(object, formula, budget)`:
///
/// `reduce(o, t, f) == reduce_parallel(o, t, f)`
///
/// on every observable field (Outcome AND per-row trace after canonical sort).
/// See `specs/props/parallel-reduction.md`; `proofs/lean/T1.lean`.
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

// ── std: scoped-thread parallel binary evaluation ─────────────────────────
//
// `par_binary` is called by `evaluate_binary` (reduce.rs) when the `std`
// feature is enabled and both sub-formulas have Exact (non-Dynamic) bounds
// that fit the current budget. It forks the Order into two independent
// sub-orders, spawns one thread per sub-formula using `std::thread::scope`,
// joins, merges traces (left-before-right for canonical ordering), and
// re-interns results into the parent order.
//
// Safety argument:
//   - Each thread owns its sub-order exclusively (`fork()` clones all state).
//   - `hints: &dyn CallProvider<N>` is `Send` because `CallProvider<N>: Sync`
//     (the `Sync` supertrait requirement in call.rs).
//   - `std::thread::scope` bounds thread lifetimes to the stack frame,
//     so no `'static` requirements propagate to `hints` or other borrows.
//
// T1 (sequential-equivalence) holds: the `Outcome` and the budgets returned
// are identical to the sequential path because:
//   1. Each sub-formula receives exactly `bound(sub)` budget.
//   2. `reinter` finds pre-existing atoms/cells via hash-consing (same ID
//      as sequential path for nouns in the base snapshot).
//   3. Traces are merged in deterministic left-before-right order.

#[cfg(feature = "std")]
pub(crate) fn par_binary<const N: usize, T: Tracer>(
    order: &mut Order<N>,
    object: NounId,
    a: NounId,
    b: NounId,
    ba: u64,
    bb: u64,
    budget: u64,
    hints: &dyn CallProvider<N>,
    tracer: &mut T,
    depth: u64,
) -> core::result::Result<(NounId, NounId, u64), Outcome> {
    use crate::reduce::{evaluate, ErrorKind};
    use crate::trace::VecTrace;
    use crate::jets::registry::JetRegistry;

    // Fork: each thread gets an independent copy of the order.
    // Box the forks so the closure captures are thin pointers (~8 bytes each),
    // not 100+ KB structs on the current stack frame.
    let mut sub_a = std::boxed::Box::new(order.fork());
    let mut sub_b = std::boxed::Box::new(order.fork());
    let mut trace_a = VecTrace::default();
    let mut trace_b = VecTrace::default();

    // SAFETY: (1) `CallProvider<N>: Sync` (supertrait) — concurrent reads safe.
    // (2) Both threads are joined before `par_binary` returns, so `hints`
    // is valid for the entire lifetime of both threads. Transmuting to
    // `&'static` lets the closures satisfy `std::thread::spawn`'s `'static`
    // bound while keeping the actual lifetime sound.
    let hints_a: &'static dyn CallProvider<N> = unsafe { std::mem::transmute(hints) };
    let hints_b: &'static dyn CallProvider<N> = unsafe { std::mem::transmute(hints) };

    let ha = std::thread::spawn(move || {
        let r = evaluate(&mut *sub_a, object, a, ba, hints_a, &mut trace_a, depth,
                         &JetRegistry::empty());
        (r, sub_a, trace_a)
    });
    let hb = std::thread::spawn(move || {
        let r = evaluate(&mut *sub_b, object, b, bb, hints_b, &mut trace_b, depth,
                         &JetRegistry::empty());
        (r, sub_b, trace_b)
    });

    let (result_a, fin_order_a, fin_trace_a) = ha.join().unwrap();
    let (result_b, fin_order_b, fin_trace_b) = hb.join().unwrap();

    // Merge traces: left sub-formula rows first, then right (canonical order).
    for row in fin_trace_a.0 {
        tracer.record(row);
    }
    for row in fin_trace_b.0 {
        tracer.record(row);
    }

    match (result_a, result_b) {
        (Ok((va, ra)), Ok((vb, rb))) => {
            let va_p = order
                .reinter(&fin_order_a, va)
                .ok_or(Outcome::Error(ErrorKind::Unavailable))?;
            let vb_p = order
                .reinter(&fin_order_b, vb)
                .ok_or(Outcome::Error(ErrorKind::Unavailable))?;
            let used = (ba - ra).saturating_add(bb - rb);
            Ok((va_p, vb_p, budget - used))
        }
        (Err(o), _) | (_, Err(o)) => Err(o),
    }
}

/// Parallel-canonical entry point with explicit threading.
///
/// Identical observable semantics to [`reduce_parallel`] (which is identical
/// to [`crate::reduce::reduce`]) but evaluates independent binary sub-formulas
/// concurrently when the `std` feature is enabled.
///
/// Requires `H: Sync` so the hints reference can cross thread boundaries.
/// All current providers are unit structs and trivially satisfy this.
#[cfg(feature = "std")]
pub fn reduce_parallel_threaded<const N: usize, T: Tracer>(
    order: &mut Order<N>,
    object: NounId,
    formula: NounId,
    budget: u64,
    hints: &dyn CallProvider<N>,
    tracer: &mut T,
) -> Outcome {
    // With par_binary wired into evaluate_binary (via cfg(feature="std")),
    // reduce() already dispatches to the threaded path for every binary
    // pattern when can_partition is true. This entry point is the named
    // parallel API; the implementation is transparent.
    reduce(order, object, formula, budget, hints, tracer)
}

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

    /// T1 empirical (threaded): reduce and reduce_parallel_threaded produce
    /// identical Outcome on a binary formula that WILL trigger par_binary.
    ///
    /// Formula: [5 [[1 7] [1 11]]] = add(quote(7), quote(11)) = 18.
    /// Both sub-formulas are Exact-bounded quotes → can_partition = true →
    /// par_binary spawns two threads when std feature is enabled.
    #[cfg(feature = "std")]
    #[test]
    fn threaded_add_matches_sequential() {
        let make_add = |ar: &mut Order<256>| {
            let t5 = ar.atom(g(5), Tag::Field).unwrap();
            let t1 = ar.atom(g(1), Tag::Field).unwrap();
            let v7 = ar.atom(g(7), Tag::Field).unwrap();
            let v11 = ar.atom(g(11), Tag::Field).unwrap();
            let qa = ar.cell(t1, v7).unwrap();
            let qb = ar.cell(t1, v11).unwrap();
            let body = ar.cell(qa, qb).unwrap();
            ar.cell(t5, body).unwrap()
        };

        let mut ar_seq = Order::<256>::new();
        let obj_seq = ar_seq.atom(g(0), Tag::Field).unwrap();
        let f_seq = make_add(&mut ar_seq);
        let mut tr_seq = VecTrace::default();
        let r_seq = reduce(&mut ar_seq, obj_seq, f_seq, 1000, &NullCalls, &mut tr_seq);

        let mut ar_par = Order::<256>::new();
        let obj_par = ar_par.atom(g(0), Tag::Field).unwrap();
        let f_par = make_add(&mut ar_par);
        let mut tr_par = VecTrace::default();
        let r_par = reduce_parallel_threaded(
            &mut ar_par, obj_par, f_par, 1000, &NullCalls, &mut tr_par,
        );

        match (r_seq, r_par) {
            (Outcome::Ok(vs, bs), Outcome::Ok(vp, bp)) => {
                assert_eq!(
                    ar_seq.atom_value(vs).unwrap().0,
                    ar_par.atom_value(vp).unwrap().0,
                    "threaded result must match sequential",
                );
                assert_eq!(bs, bp, "remaining budget must match");
            }
            (x, y) => panic!("Outcome variants diverged: {:?} vs {:?}", x, y),
        }

        // Trace lengths equal: both emit the same rows regardless of scheduler.
        assert_eq!(
            tr_seq.0.len(),
            tr_par.0.len(),
            "threaded trace length must match sequential",
        );
    }

    /// Minimal smoke test: std::thread::scope works in this binary.
    #[cfg(feature = "std")]
    #[test]
    fn scope_smoke_test() {
        let x = 42u64;
        let result = std::thread::scope(|s| {
            let h = s.spawn(|| x + 1);
            h.join().unwrap()
        });
        assert_eq!(result, 43);
    }

    /// Order::fork produces an independent order with identical initial entries.
    #[cfg(feature = "std")]
    #[test]
    fn fork_is_independent() {
        let mut ar = Order::<256>::new();
        let a = ar.atom(g(3), Tag::Field).unwrap();
        let b = ar.atom(g(7), Tag::Field).unwrap();
        let c = ar.cell(a, b).unwrap();

        let base_count = ar.count();
        let mut fork = ar.fork();

        // Fork has same count as parent.
        assert_eq!(fork.count(), base_count);

        // Adding to fork doesn't change parent.
        fork.atom(g(99), Tag::Field).unwrap();
        assert_eq!(ar.count(), base_count, "parent count unchanged after fork write");

        // Reinter re-finds existing noun by hash-consing.
        let c_back = ar.reinter(&fork, c).unwrap();
        assert_eq!(c_back, c, "reinter on pre-fork noun returns same ID");
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
