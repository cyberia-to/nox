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
