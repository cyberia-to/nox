// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! Static cost bounds for formulas (O(1) lookup, cached on `DataEntry`).
//!
//! Every formula `t` has a structural upper bound `bound(t) ≥ actual_cost(t)`.
//! The bound is computed once at data-construction time (see
//! `Order::compute_pair_bound`) and cached on the `DataEntry`. Reduce-time
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

pub use crate::data::Cost;
use crate::data::{Order, OrderId};

/// Return the cached cost bound for a formula rooted at `root`.
///
/// O(1) — reads from the precomputed `bound` field on `DataEntry`. Atoms
/// always return `Cost::Exact(0)`. Out-of-range order ids return
/// `Cost::Exact(0)` (the reducer will charge `Malformed` later).
#[inline]
pub fn bound<const N: usize>(order: &Order<N>, root: OrderId) -> Cost {
    order.get(root).map_or(Cost::Exact(0), |e| e.bound)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{Order};
    use nebu::Goldilocks;

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    #[test]
    fn atom_has_zero_bound() {
        let mut ar = Order::<256>::new();
        let a = ar.atom(g(42)).unwrap();
        assert_eq!(bound(&ar, a), Cost::Exact(0));
    }

    #[test]
    fn quote_has_bound_one() {
        let mut ar = Order::<256>::new();
        let t1 = ar.atom(g(1)).unwrap();
        let v = ar.atom(g(42)).unwrap();
        let f = ar.pair(t1, v).unwrap();
        assert_eq!(bound(&ar, f), Cost::Exact(1));
    }

    #[test]
    fn axis_has_bound_one() {
        let mut ar = Order::<256>::new();
        let t0 = ar.atom(g(0)).unwrap();
        let addr = ar.atom(g(7)).unwrap();
        let f = ar.pair(t0, addr).unwrap();
        assert_eq!(bound(&ar, f), Cost::Exact(1));
    }

    #[test]
    fn add_sums_subbounds() {
        // [5 [[1 3] [1 5]]] → 1 + 1 + 1 = 3
        let mut ar = Order::<256>::new();
        let t5 = ar.atom(g(5)).unwrap();
        let t1 = ar.atom(g(1)).unwrap();
        let v3 = ar.atom(g(3)).unwrap();
        let v5 = ar.atom(g(5)).unwrap();
        let qa = ar.pair(t1, v3).unwrap();
        let qb = ar.pair(t1, v5).unwrap();
        let body = ar.pair(qa, qb).unwrap();
        let f = ar.pair(t5, body).unwrap();
        assert_eq!(bound(&ar, f), Cost::Exact(3));
    }

    #[test]
    fn inv_costs_64_plus_inner() {
        let mut ar = Order::<256>::new();
        let t8 = ar.atom(g(8)).unwrap();
        let t1 = ar.atom(g(1)).unwrap();
        let v = ar.atom(g(42)).unwrap();
        let body = ar.pair(t1, v).unwrap();
        let f = ar.pair(t8, body).unwrap();
        assert_eq!(bound(&ar, f), Cost::Exact(65));
    }

    #[test]
    fn hash_costs_25_plus_inner() {
        let mut ar = Order::<256>::new();
        let t15 = ar.atom(g(15)).unwrap();
        let t1 = ar.atom(g(1)).unwrap();
        let v = ar.atom(g(7)).unwrap();
        let body = ar.pair(t1, v).unwrap();
        let f = ar.pair(t15, body).unwrap();
        assert_eq!(bound(&ar, f), Cost::Exact(26));
    }

    #[test]
    fn lt_costs_64_plus_subbounds() {
        let mut ar = Order::<256>::new();
        let t10 = ar.atom(g(10)).unwrap();
        let t1 = ar.atom(g(1)).unwrap();
        let v3 = ar.atom(g(3)).unwrap();
        let v5 = ar.atom(g(5)).unwrap();
        let qa = ar.pair(t1, v3).unwrap();
        let qb = ar.pair(t1, v5).unwrap();
        let body = ar.pair(qa, qb).unwrap();
        let f = ar.pair(t10, body).unwrap();
        assert_eq!(bound(&ar, f), Cost::Exact(66));
    }

    #[test]
    fn branch_takes_max_of_arms() {
        let mut ar = Order::<256>::new();
        let t4 = ar.atom(g(4)).unwrap();
        let t1 = ar.atom(g(1)).unwrap();
        let t8 = ar.atom(g(8)).unwrap();
        let zero = ar.atom(g(0)).unwrap();
        let v99 = ar.atom(g(99)).unwrap();
        let v42 = ar.atom(g(42)).unwrap();
        let test = ar.pair(t1, zero).unwrap();
        let yes = ar.pair(t1, v99).unwrap();
        let no_inner = ar.pair(t1, v42).unwrap();
        let no = ar.pair(t8, no_inner).unwrap();
        let arms = ar.pair(yes, no).unwrap();
        let body = ar.pair(test, arms).unwrap();
        let f = ar.pair(t4, body).unwrap();
        // 1 + bound(test=1) + max(bound(yes=1), bound(no=65)) = 1 + 1 + 65 = 67
        assert_eq!(bound(&ar, f), Cost::Exact(67));
    }

    #[test]
    fn compose_is_dynamic() {
        let mut ar = Order::<256>::new();
        let t2 = ar.atom(g(2)).unwrap();
        let t1 = ar.atom(g(1)).unwrap();
        let a = ar.atom(g(10)).unwrap();
        let b = ar.atom(g(20)).unwrap();
        let qa = ar.pair(t1, a).unwrap();
        let qb = ar.pair(t1, b).unwrap();
        let body = ar.pair(qa, qb).unwrap();
        let f = ar.pair(t2, body).unwrap();
        let c = bound(&ar, f);
        assert!(c.is_dynamic());
        assert_eq!(c.value(), 3);
    }

    #[test]
    fn call_is_dynamic() {
        let mut ar = Order::<256>::new();
        let t16 = ar.atom(g(16)).unwrap();
        let t1 = ar.atom(g(1)).unwrap();
        let tag_v = ar.atom(g(0)).unwrap();
        let check_v = ar.atom(g(0)).unwrap();
        let qt = ar.pair(t1, tag_v).unwrap();
        let qc = ar.pair(t1, check_v).unwrap();
        let body = ar.pair(qt, qc).unwrap();
        let f = ar.pair(t16, body).unwrap();
        let c = bound(&ar, f);
        assert!(c.is_dynamic());
        assert_eq!(c.value(), 2);
    }

    #[test]
    fn dynamic_propagates_through_parents() {
        let mut ar = Order::<256>::new();
        let t5 = ar.atom(g(5)).unwrap();
        let t2 = ar.atom(g(2)).unwrap();
        let t1 = ar.atom(g(1)).unwrap();
        let v1 = ar.atom(g(1)).unwrap();
        let v2 = ar.atom(g(2)).unwrap();
        let v3 = ar.atom(g(3)).unwrap();
        let q1 = ar.pair(t1, v1).unwrap();
        let q2 = ar.pair(t1, v2).unwrap();
        let q3 = ar.pair(t1, v3).unwrap();
        let compose_body = ar.pair(q1, q2).unwrap();
        let compose_f = ar.pair(t2, compose_body).unwrap();
        let add_body = ar.pair(compose_f, q3).unwrap();
        let f = ar.pair(t5, add_body).unwrap();
        assert!(bound(&ar, f).is_dynamic());
    }

    #[test]
    fn bound_lookup_is_constant_time_under_deep_nesting() {
        // Construct a 1000-level-deep compose chain. With cached bound,
        // each pair construction is O(1) and the final bound lookup is O(1).
        // Without caching, this would be O(N²) at construction time and
        // O(N) per lookup.
        let mut ar = Order::<8192>::new();
        let t2 = ar.atom(g(2)).unwrap();
        let t1 = ar.atom(g(1)).unwrap();
        let zero = ar.atom(g(0)).unwrap();
        let quote_zero = ar.pair(t1, zero).unwrap();
        let mut chain = quote_zero;
        for _ in 0..1000 {
            let body = ar.pair(quote_zero, chain).unwrap();
            chain = ar.pair(t2, body).unwrap();
        }
        // Whatever the value, it must be Dynamic (chain contains compose).
        assert!(bound(&ar, chain).is_dynamic());
    }
}
