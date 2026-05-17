// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! Static cost bounds for formulas (O(1) lookup, cached on `NounEntry`).
//!
//! Every formula `t` has a structural upper bound `bound(t) ≥ actual_cost(t)`.
//! The bound is computed once at noun-construction time (see
//! `Order::compute_cell_bound`) and cached on the `NounEntry`. Reduce-time
//! parallel-scheduling decisions look it up in O(1) — there is no recursive
//! tree walk at reduce time.
//!
//! Two patterns cross the **dynamic-cost frontier** where the bound cannot be
//! determined structurally:
//!
//! - **compose (2)**: the third reduce `reduce(rx, ry)` operates on the
//!   runtime result of `reduce(o, y)`. Its cost depends on the formula
//!   produced at runtime, not on the static `[2 [x y]]` tree.
//! - **call (16)**: the witness is supplied by the prover; its formula
//!   shape is unknown at static-analysis time.
//!
//! For these, `bound()` returns [`Cost::Dynamic`] with the prefix bound.
//! The reducer treats `Dynamic` as the signal to use sequential threading
//! for the continuation phase (see `specs/reduction.md §parallel reduction`).

pub use crate::noun::Cost;
use crate::noun::{Order, NounId};

/// Return the cached cost bound for a formula rooted at `root`.
///
/// O(1) — reads from the precomputed `bound` field on `NounEntry`. Atoms
/// always return `Cost::Exact(0)`. Out-of-range NounIds return
/// `Cost::Exact(0)` (the reducer will charge `Malformed` later).
#[inline]
pub fn bound<const N: usize>(order: &Order<N>, root: NounId) -> Cost {
    order.get(root).map_or(Cost::Exact(0), |e| e.bound)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::noun::{Order, Tag};
    use nebu::Goldilocks;

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    #[test]
    fn atom_has_zero_bound() {
        let mut ar = Order::<256>::new();
        let a = ar.atom(g(42), Tag::Field).unwrap();
        assert_eq!(bound(&ar, a), Cost::Exact(0));
    }

    #[test]
    fn quote_has_bound_one() {
        let mut ar = Order::<256>::new();
        let t1 = ar.atom(g(1), Tag::Field).unwrap();
        let v = ar.atom(g(42), Tag::Field).unwrap();
        let f = ar.cell(t1, v).unwrap();
        assert_eq!(bound(&ar, f), Cost::Exact(1));
    }

    #[test]
    fn axis_has_bound_one() {
        let mut ar = Order::<256>::new();
        let t0 = ar.atom(g(0), Tag::Field).unwrap();
        let addr = ar.atom(g(7), Tag::Field).unwrap();
        let f = ar.cell(t0, addr).unwrap();
        assert_eq!(bound(&ar, f), Cost::Exact(1));
    }

    #[test]
    fn add_sums_subbounds() {
        // [5 [[1 3] [1 5]]] → 1 + 1 + 1 = 3
        let mut ar = Order::<256>::new();
        let t5 = ar.atom(g(5), Tag::Field).unwrap();
        let t1 = ar.atom(g(1), Tag::Field).unwrap();
        let v3 = ar.atom(g(3), Tag::Field).unwrap();
        let v5 = ar.atom(g(5), Tag::Field).unwrap();
        let qa = ar.cell(t1, v3).unwrap();
        let qb = ar.cell(t1, v5).unwrap();
        let body = ar.cell(qa, qb).unwrap();
        let f = ar.cell(t5, body).unwrap();
        assert_eq!(bound(&ar, f), Cost::Exact(3));
    }

    #[test]
    fn inv_costs_64_plus_inner() {
        let mut ar = Order::<256>::new();
        let t8 = ar.atom(g(8), Tag::Field).unwrap();
        let t1 = ar.atom(g(1), Tag::Field).unwrap();
        let v = ar.atom(g(42), Tag::Field).unwrap();
        let body = ar.cell(t1, v).unwrap();
        let f = ar.cell(t8, body).unwrap();
        assert_eq!(bound(&ar, f), Cost::Exact(65));
    }

    #[test]
    fn hash_costs_25_plus_inner() {
        let mut ar = Order::<256>::new();
        let t15 = ar.atom(g(15), Tag::Field).unwrap();
        let t1 = ar.atom(g(1), Tag::Field).unwrap();
        let v = ar.atom(g(7), Tag::Field).unwrap();
        let body = ar.cell(t1, v).unwrap();
        let f = ar.cell(t15, body).unwrap();
        assert_eq!(bound(&ar, f), Cost::Exact(26));
    }

    #[test]
    fn lt_costs_64_plus_subbounds() {
        let mut ar = Order::<256>::new();
        let t10 = ar.atom(g(10), Tag::Field).unwrap();
        let t1 = ar.atom(g(1), Tag::Field).unwrap();
        let v3 = ar.atom(g(3), Tag::Field).unwrap();
        let v5 = ar.atom(g(5), Tag::Field).unwrap();
        let qa = ar.cell(t1, v3).unwrap();
        let qb = ar.cell(t1, v5).unwrap();
        let body = ar.cell(qa, qb).unwrap();
        let f = ar.cell(t10, body).unwrap();
        assert_eq!(bound(&ar, f), Cost::Exact(66));
    }

    #[test]
    fn branch_takes_max_of_arms() {
        let mut ar = Order::<256>::new();
        let t4 = ar.atom(g(4), Tag::Field).unwrap();
        let t1 = ar.atom(g(1), Tag::Field).unwrap();
        let t8 = ar.atom(g(8), Tag::Field).unwrap();
        let zero = ar.atom(g(0), Tag::Field).unwrap();
        let v99 = ar.atom(g(99), Tag::Field).unwrap();
        let v42 = ar.atom(g(42), Tag::Field).unwrap();
        let test = ar.cell(t1, zero).unwrap();
        let yes = ar.cell(t1, v99).unwrap();
        let no_inner = ar.cell(t1, v42).unwrap();
        let no = ar.cell(t8, no_inner).unwrap();
        let arms = ar.cell(yes, no).unwrap();
        let body = ar.cell(test, arms).unwrap();
        let f = ar.cell(t4, body).unwrap();
        // 1 + bound(test=1) + max(bound(yes=1), bound(no=65)) = 1 + 1 + 65 = 67
        assert_eq!(bound(&ar, f), Cost::Exact(67));
    }

    #[test]
    fn compose_is_dynamic() {
        let mut ar = Order::<256>::new();
        let t2 = ar.atom(g(2), Tag::Field).unwrap();
        let t1 = ar.atom(g(1), Tag::Field).unwrap();
        let a = ar.atom(g(10), Tag::Field).unwrap();
        let b = ar.atom(g(20), Tag::Field).unwrap();
        let qa = ar.cell(t1, a).unwrap();
        let qb = ar.cell(t1, b).unwrap();
        let body = ar.cell(qa, qb).unwrap();
        let f = ar.cell(t2, body).unwrap();
        let c = bound(&ar, f);
        assert!(c.is_dynamic());
        assert_eq!(c.value(), 3);
    }

    #[test]
    fn call_is_dynamic() {
        let mut ar = Order::<256>::new();
        let t16 = ar.atom(g(16), Tag::Field).unwrap();
        let t1 = ar.atom(g(1), Tag::Field).unwrap();
        let tag_v = ar.atom(g(0), Tag::Field).unwrap();
        let check_v = ar.atom(g(0), Tag::Field).unwrap();
        let qt = ar.cell(t1, tag_v).unwrap();
        let qc = ar.cell(t1, check_v).unwrap();
        let body = ar.cell(qt, qc).unwrap();
        let f = ar.cell(t16, body).unwrap();
        let c = bound(&ar, f);
        assert!(c.is_dynamic());
        assert_eq!(c.value(), 2);
    }

    #[test]
    fn dynamic_propagates_through_parents() {
        let mut ar = Order::<256>::new();
        let t5 = ar.atom(g(5), Tag::Field).unwrap();
        let t2 = ar.atom(g(2), Tag::Field).unwrap();
        let t1 = ar.atom(g(1), Tag::Field).unwrap();
        let v1 = ar.atom(g(1), Tag::Field).unwrap();
        let v2 = ar.atom(g(2), Tag::Field).unwrap();
        let v3 = ar.atom(g(3), Tag::Field).unwrap();
        let q1 = ar.cell(t1, v1).unwrap();
        let q2 = ar.cell(t1, v2).unwrap();
        let q3 = ar.cell(t1, v3).unwrap();
        let compose_body = ar.cell(q1, q2).unwrap();
        let compose_f = ar.cell(t2, compose_body).unwrap();
        let add_body = ar.cell(compose_f, q3).unwrap();
        let f = ar.cell(t5, add_body).unwrap();
        assert!(bound(&ar, f).is_dynamic());
    }

    #[test]
    fn bound_lookup_is_constant_time_under_deep_nesting() {
        // Construct a 1000-level-deep compose chain. With cached bound,
        // each cell construction is O(1) and the final bound lookup is O(1).
        // Without caching, this would be O(N²) at construction time and
        // O(N) per lookup.
        let mut ar = Order::<8192>::new();
        let t2 = ar.atom(g(2), Tag::Field).unwrap();
        let t1 = ar.atom(g(1), Tag::Field).unwrap();
        let zero = ar.atom(g(0), Tag::Field).unwrap();
        let quote_zero = ar.cell(t1, zero).unwrap();
        let mut chain = quote_zero;
        for _ in 0..1000 {
            let body = ar.cell(quote_zero, chain).unwrap();
            chain = ar.cell(t2, body).unwrap();
        }
        // Whatever the value, it must be Dynamic (chain contains compose).
        assert!(bound(&ar, chain).is_dynamic());
    }
}
