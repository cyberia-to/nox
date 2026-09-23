//! Static cost bound type — cached per DataEntry.
//!
//! `Cost::Exact(n)` is a tight upper bound. `Cost::Dynamic(n)` carries the
//! statically-analyzable prefix and signals that the formula contains a
//! compose (tag 2) or call (tag 16) at some depth — patterns where the
//! continuation cost depends on runtime values.
//!
//! Cached on [`crate::data::reduction::DataEntry`] at construction time so that
//! the parallel-scheduling decision `bound(child) ≤ budget?` is O(1) at
//! reduce time, independent of formula depth.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cost {
    /// Tight upper bound — `bound(t) ≥ actual_cost(t)` always holds.
    Exact(u64),
    /// Prefix cost is known; remainder is runtime-dependent.
    /// `n` is the bound for the statically-analyzable phase.
    Dynamic(u64),
}

impl Cost {
    /// Numeric prefix cost. For `Exact(n)` this is the full bound;
    /// for `Dynamic(n)` it covers only the static phase.
    #[inline]
    pub fn value(self) -> u64 {
        match self { Cost::Exact(n) | Cost::Dynamic(n) => n }
    }

    /// True if any sub-formula crossed the dynamic-cost frontier.
    #[inline]
    pub fn is_dynamic(self) -> bool {
        matches!(self, Cost::Dynamic(_))
    }

    /// Combine two child costs under a structural parent of cost `parent`.
    /// `Dynamic` is absorbing: any dynamic child makes the parent dynamic.
    #[inline]
    pub fn sum(parent: u64, a: Cost, b: Cost) -> Cost {
        let total = parent.saturating_add(a.value()).saturating_add(b.value());
        if a.is_dynamic() || b.is_dynamic() {
            Cost::Dynamic(total)
        } else {
            Cost::Exact(total)
        }
    }

    /// Combine a single child under a structural parent.
    #[inline]
    pub fn sum1(parent: u64, a: Cost) -> Cost {
        let total = parent.saturating_add(a.value());
        if a.is_dynamic() { Cost::Dynamic(total) } else { Cost::Exact(total) }
    }

    /// For branch (pattern 4): bound = parent + bound(test) + max(bound(yes), bound(no)).
    /// `Dynamic` in any sub propagates; the larger arm wins the max.
    #[inline]
    pub fn branch(parent: u64, test: Cost, yes: Cost, no: Cost) -> Cost {
        let arm = yes.value().max(no.value());
        let total = parent.saturating_add(test.value()).saturating_add(arm);
        let dyn_any = test.is_dynamic() || yes.is_dynamic() || no.is_dynamic();
        if dyn_any { Cost::Dynamic(total) } else { Cost::Exact(total) }
    }
}

/// Per-pattern static dispatch cost — must mirror `reduce::COSTS`.
/// Used by `Reduction::pair` to compute cached bounds at construction time.
pub(crate) const PATTERN_COSTS: [u64; 18] = [
    1,   // 0  axis
    1,   // 1  quote
    1,   // 2  compose (DYNAMIC continuation)
    1,   // 3  cons
    1,   // 4  branch (max-of-arms)
    1,   // 5  add
    1,   // 6  sub
    1,   // 7  mul
    64,  // 8  inv
    1,   // 9  eq
    64,  // 10 lt
    32,  // 11 xor
    32,  // 12 and
    32,  // 13 not
    32,  // 14 shl
    25,  // 15 hash
    1,   // 16 call (DYNAMIC continuation)
    1,   // 17 look
];

#[cfg(test)]
mod tests {
    use super::*;

    // The doc comment above promises PATTERN_COSTS mirrors reduce::COSTS;
    // nothing checked that until now, so the two tables could silently
    // drift and the cached O(1) bound on DataEntry would stop matching
    // what reduce() actually charges per pattern.
    #[test]
    fn pattern_costs_matches_reduce_costs() {
        assert_eq!(PATTERN_COSTS, crate::reduce::COSTS);
    }

    #[test]
    fn value_returns_inner_n_for_both_variants() {
        assert_eq!(Cost::Exact(5).value(), 5);
        assert_eq!(Cost::Dynamic(7).value(), 7);
    }

    #[test]
    fn is_dynamic_only_true_for_dynamic() {
        assert!(!Cost::Exact(0).is_dynamic());
        assert!(Cost::Dynamic(0).is_dynamic());
    }

    #[test]
    fn sum_of_two_exact_is_exact() {
        assert_eq!(Cost::sum(3, Cost::Exact(2), Cost::Exact(4)), Cost::Exact(9));
    }

    #[test]
    fn sum_dynamic_is_absorbing_from_either_side() {
        assert!(Cost::sum(1, Cost::Dynamic(2), Cost::Exact(3)).is_dynamic());
        assert!(Cost::sum(1, Cost::Exact(2), Cost::Dynamic(3)).is_dynamic());
    }

    #[test]
    fn sum_saturates_instead_of_overflowing() {
        let c = Cost::sum(u64::MAX, Cost::Exact(u64::MAX), Cost::Exact(1));
        assert_eq!(c, Cost::Exact(u64::MAX));
    }

    #[test]
    fn sum1_matches_sum_with_zero_second_child() {
        assert_eq!(Cost::sum1(3, Cost::Exact(2)), Cost::Exact(5));
        assert!(Cost::sum1(3, Cost::Dynamic(2)).is_dynamic());
    }

    #[test]
    fn sum1_saturates_instead_of_overflowing() {
        assert_eq!(Cost::sum1(u64::MAX, Cost::Exact(u64::MAX)), Cost::Exact(u64::MAX));
    }

    #[test]
    fn branch_takes_max_of_the_two_arms() {
        let c = Cost::branch(1, Cost::Exact(1), Cost::Exact(10), Cost::Exact(3));
        assert_eq!(c, Cost::Exact(12)); // 1 (parent) + 1 (test) + max(10, 3)

        let c2 = Cost::branch(1, Cost::Exact(1), Cost::Exact(3), Cost::Exact(10));
        assert_eq!(c2, Cost::Exact(12)); // symmetric: still picks the larger arm
    }

    #[test]
    fn branch_dynamic_propagates_from_test_or_either_arm() {
        assert!(Cost::branch(0, Cost::Dynamic(0), Cost::Exact(0), Cost::Exact(0)).is_dynamic());
        assert!(Cost::branch(0, Cost::Exact(0), Cost::Dynamic(0), Cost::Exact(0)).is_dynamic());
        assert!(Cost::branch(0, Cost::Exact(0), Cost::Exact(0), Cost::Dynamic(0)).is_dynamic());
    }

    #[test]
    fn branch_saturates_instead_of_overflowing() {
        let c = Cost::branch(u64::MAX, Cost::Exact(1), Cost::Exact(1), Cost::Exact(0));
        assert_eq!(c, Cost::Exact(u64::MAX));
    }
}
