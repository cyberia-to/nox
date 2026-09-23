//! flat order for data allocation with hash-consing
//!
//! one order per order() invocation. freed when order() returns.
//! hash-consing ensures identical sub-expressions share one slot.
//! DAG, not tree — immutable data nodes, safe structural sharing.

use nebu::Goldilocks;
use super::inner::Data;
use super::hash::{Digest, hash_atom, hash_pair};
use super::cost::{Cost, PATTERN_COSTS};
use super::{Order, NIL};

/// order entry — data node + cached identity hash + cached cost bound.
///
/// `bound` is computed at construction time so reduce-time partition decisions
/// (`bound(child) ≤ budget?`) are O(1). The bound represents the data node treated
/// as a formula; atoms always have `Cost::Exact(0)`.
#[derive(Debug, Clone, Copy)]
pub struct DataEntry {
    pub inner: Data,
    pub hash: Digest,
    pub bound: Cost,
}

/// flat order with hash-consing
pub struct Reduction<const N: usize> {
    // SAFETY: entries[0..count] are initialized
    entries: [core::mem::MaybeUninit<DataEntry>; N],
    count: u32,
    index_keys: [Digest; N],
    index_vals: [Order; N],
    index_mask: u32,
}

impl<const N: usize> Default for Reduction<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Reduction<N> {
    pub fn new() -> Self {
        assert!(N.is_power_of_two(), "order size must be power of 2");
        Self {
            // `[const { … }; N]` initializes each slot uninitialized at compile
            // time. Replaces the deprecated `MaybeUninit::uninit().assume_init()`
            // pattern which was UB on older toolchains and Miri-flagged.
            entries: [const { core::mem::MaybeUninit::uninit() }; N],
            count: 0,
            index_keys: [[Goldilocks::ZERO; 4]; N],
            index_vals: [NIL; N],
            index_mask: (N as u32) - 1,
        }
    }

    pub fn alloc_raw(&mut self, entry: DataEntry) -> Option<Order> {
        // Cap load factor at 3/4 so linear probing stays O(1) expected and
        // bounded worst-case (~N/4 max probe). The hash-cons table lives in
        // the same Reduction; refusing past 3N/4 means index_insert always finds
        // an empty slot quickly.
        if (self.count as usize) >= (N / 4) * 3 { return None; }
        let idx = self.count;
        self.entries[idx as usize] = core::mem::MaybeUninit::new(entry);
        self.count += 1;
        Some(idx)
    }

    /// returns None for out-of-bounds Order (never panics)
    pub fn get(&self, r: Order) -> Option<&DataEntry> {
        if (r as usize) >= self.count as usize { return None; }
        // SAFETY: entries[0..count] are initialized, r < count
        Some(unsafe { self.entries[r as usize].assume_init_ref() })
    }

    pub fn atom(&mut self, value: Goldilocks) -> Option<Order> {
        let inner = Data::Atom { value };
        let hash = hash_atom(value);
        if let Some(existing) = self.index_lookup(&hash) { return Some(existing); }
        // Atoms have no formula interpretation; bound is 0 by definition.
        let r = self.alloc_raw(DataEntry { inner, hash, bound: Cost::Exact(0) })?;
        self.index_insert(&hash, r);
        Some(r)
    }

    pub fn pair(&mut self, left: Order, right: Order) -> Option<Order> {
        let lh = self.get(left)?.hash;
        let rh = self.get(right)?.hash;
        let hash = hash_pair(&lh, &rh);
        if let Some(existing) = self.index_lookup(&hash) { return Some(existing); }
        let inner = Data::Pair { left, right };
        let bound = self.compute_pair_bound(left, right);
        let r = self.alloc_raw(DataEntry { inner, hash, bound })?;
        self.index_insert(&hash, r);
        Some(r)
    }

    /// Compute the cached cost bound for a pair at construction time.
    /// Treats the pair as a formula `[tag body]` when `left` is an atom in
    /// pattern range; otherwise bound is 0 (the pair is data, not code).
    /// Children's cached bounds are looked up in O(1).
    fn compute_pair_bound(&self, left: Order, right: Order) -> Cost {
        let left_entry = match self.get(left) { Some(e) => e, None => return Cost::Exact(0) };
        let tag = match left_entry.inner {
            Data::Atom { value } => value.as_u64(),
            _ => return Cost::Exact(0),
        };
        if (tag as usize) >= PATTERN_COSTS.len() { return Cost::Exact(0); }
        let base = PATTERN_COSTS[tag as usize];

        let child_bound = |id: Order| -> Cost {
            self.get(id).map_or(Cost::Exact(0), |e| e.bound)
        };
        let pair = |id: Order| -> Option<(Order, Order)> {
            match self.get(id)?.inner {
                Data::Pair { left, right } => Some((left, right)),
                _ => None,
            }
        };

        match tag {
            // No sub-formula evaluation
            0 | 1 => Cost::Exact(base),
            // Compose: bound(x) + bound(y) statically; continuation DYNAMIC
            2 => match pair(right) {
                Some((x, y)) => {
                    let total = base
                        .saturating_add(child_bound(x).value())
                        .saturating_add(child_bound(y).value());
                    Cost::Dynamic(total)
                }
                None => Cost::Exact(base),
            },
            // Binary structural / look
            3 | 5 | 6 | 7 | 9 | 10 | 11 | 12 | 14 | 17 => match pair(right) {
                Some((a, b)) => Cost::sum(base, child_bound(a), child_bound(b)),
                None => Cost::Exact(base),
            },
            // Branch: max of arms
            4 => match pair(right) {
                Some((test, rest)) => match pair(rest) {
                    Some((yes, no)) => Cost::branch(
                        base, child_bound(test), child_bound(yes), child_bound(no),
                    ),
                    None => Cost::Exact(base),
                },
                None => Cost::Exact(base),
            },
            // Unary patterns
            8 | 13 | 15 => Cost::sum1(base, child_bound(right)),
            // Call: bound(tag) statically; check is DYNAMIC
            16 => match pair(right) {
                Some((tag_f, _check_f)) => {
                    let total = base.saturating_add(child_bound(tag_f).value());
                    Cost::Dynamic(total)
                }
                None => Cost::Exact(base),
            },
            _ => Cost::Exact(base),
        }
    }

    /// build hash data: pair(pair(h0, h1), pair(h2, h3))
    pub fn hash_data(&mut self, digest: &Digest) -> Option<Order> {
        let h0 = self.atom(digest[0])?;
        let h1 = self.atom(digest[1])?;
        let h2 = self.atom(digest[2])?;
        let h3 = self.atom(digest[3])?;
        let left = self.pair(h0, h1)?;
        let right = self.pair(h2, h3)?;
        self.pair(left, right)
    }

    /// extract digest from hash data
    pub fn read_hash_data(&self, r: Order) -> Option<Digest> {
        let (left, right) = match self.get(r)?.inner {
            Data::Pair { left, right } => (left, right),
            _ => return None,
        };
        let (h0r, h1r) = match self.get(left)?.inner {
            Data::Pair { left, right } => (left, right),
            _ => return None,
        };
        let (h2r, h3r) = match self.get(right)?.inner {
            Data::Pair { left, right } => (left, right),
            _ => return None,
        };
        Some([
            self.atom_value(h0r)?,
            self.atom_value(h1r)?,
            self.atom_value(h2r)?,
            self.atom_value(h3r)?,
        ])
    }

    fn index_lookup(&self, hash: &Digest) -> Option<Order> {
        let mut slot = (hash[0].as_u64() as u32) & self.index_mask;
        for _ in 0..N {
            let val = self.index_vals[slot as usize];
            if val == NIL { return None; }
            if self.index_keys[slot as usize] == *hash { return Some(val); }
            slot = (slot + 1) & self.index_mask;
        }
        None
    }

    fn index_insert(&mut self, hash: &Digest, r: Order) {
        let mut slot = (hash[0].as_u64() as u32) & self.index_mask;
        loop {
            if self.index_vals[slot as usize] == NIL {
                self.index_keys[slot as usize] = *hash;
                self.index_vals[slot as usize] = r;
                return;
            }
            slot = (slot + 1) & self.index_mask;
        }
    }

    pub fn count(&self) -> u32 { self.count }

    // ── std-only: fork / reinter ───────────────────────────────────
    // Used by the parallel executor (parallel.rs) to give each thread
    // a private mutable Reduction derived from the shared parent state.
    //
    // Invariant (fork): for all i < self.count, fork()[i] == self[i].
    // Invariant (reinter): reinter(src, id) == id if id < self.count at fork time.
    // These two invariants together mean re-interning an atom or pair
    // from a sub-order into the parent finds the pre-existing entry
    // (via hash-consing) and returns the same Order. Only entries
    // created DURING evaluation (id >= base_count) require new allocations.

    /// Clone this order into a fresh independent order.
    ///
    /// The fork shares the same initial data set (entries 0..self.count).
    /// New data nodes allocated in the fork do NOT appear in self, and vice
    /// versa. Used by the parallel executor: one fork per thread.
    #[cfg(feature = "std")]
    pub fn fork(&self) -> Self {
        let mut new = Reduction::<N>::new();
        new.count = self.count;
        new.index_mask = self.index_mask;
        // Copy initialized entries (SAFETY: entries[0..count] are initialized).
        for i in 0..self.count as usize {
            let entry = unsafe { self.entries[i].assume_init() };
            new.entries[i] = core::mem::MaybeUninit::new(entry);
        }
        // Copy the full hash-cons table (probe chains may span any slot in 0..N).
        new.index_keys.copy_from_slice(&self.index_keys);
        new.index_vals.copy_from_slice(&self.index_vals);
        new
    }

    /// Re-intern a data node from `src` into `self` and return its Order in self.
    ///
    /// Atoms are found (or created) by value+tag. Pairs are recursively
    /// re-interned then joined. Hash-consing guarantees idempotency: if the
    /// data node was in self before the fork, `reinter` returns the original Order.
    ///
    /// Returns `None` if the order is full (same failure mode as `atom`/`pair`).
    #[cfg(feature = "std")]
    pub fn reinter(&mut self, src: &Self, id: Order) -> Option<Order> {
        let entry = src.get(id)?;
        match entry.inner {
            Data::Atom { value } => self.atom(value),
            Data::Pair { left, right } => {
                let l = self.reinter(src, left)?;
                let r = self.reinter(src, right)?;
                self.pair(l, r)
            }
        }
    }

    pub fn is_atom(&self, r: Order) -> bool {
        self.get(r).is_some_and(|e| matches!(e.inner, Data::Atom { .. }))
    }

    pub fn is_pair(&self, r: Order) -> bool {
        self.get(r).is_some_and(|e| matches!(e.inner, Data::Pair { .. }))
    }

    pub fn head(&self, r: Order) -> Option<Order> {
        match self.get(r)?.inner { Data::Pair { left, .. } => Some(left), _ => None }
    }

    pub fn tail(&self, r: Order) -> Option<Order> {
        match self.get(r)?.inner { Data::Pair { right, .. } => Some(right), _ => None }
    }

    pub fn atom_value(&self, r: Order) -> Option<Goldilocks> {
        match self.get(r)?.inner { Data::Atom { value } => Some(value), _ => None }
    }

    pub fn digest(&self, r: Order) -> Option<&Digest> {
        Some(&self.get(r)?.hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g(v: u64) -> Goldilocks {
        Goldilocks::new(v)
    }

    #[test]
    fn atom_hash_conses_identical_values() {
        let mut r = Reduction::<16>::new();
        let a = r.atom(g(7)).unwrap();
        let b = r.atom(g(7)).unwrap();
        assert_eq!(a, b);
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn atom_distinguishes_values() {
        let mut r = Reduction::<16>::new();
        let a = r.atom(g(7)).unwrap();
        let b = r.atom(g(8)).unwrap();
        assert_ne!(a, b);
        assert_eq!(r.count(), 2);
    }

    #[test]
    fn pair_hash_conses_identical_children() {
        let mut r = Reduction::<16>::new();
        let a = r.atom(g(1)).unwrap();
        let b = r.atom(g(2)).unwrap();
        let p1 = r.pair(a, b).unwrap();
        let p2 = r.pair(a, b).unwrap();
        assert_eq!(p1, p2);
        assert_eq!(r.count(), 3); // two atoms + one pair, not two pairs
    }

    #[test]
    fn get_returns_none_out_of_bounds() {
        let mut r = Reduction::<16>::new();
        let a = r.atom(g(1)).unwrap();
        assert!(r.get(a).is_some());
        assert!(r.get(a + 1).is_none());
        assert!(r.get(Order::MAX).is_none());
    }

    #[test]
    fn alloc_raw_refuses_past_three_quarter_load_factor() {
        // N=4: cap is (4/4)*3 = 3 entries. Distinct atoms never hash-cons,
        // so the 4th allocation must be refused instead of overflowing entries[].
        let mut r = Reduction::<4>::new();
        assert!(r.atom(g(1)).is_some());
        assert!(r.atom(g(2)).is_some());
        assert!(r.atom(g(3)).is_some());
        assert_eq!(r.count(), 3);
        assert!(r.atom(g(4)).is_none());
        assert_eq!(r.count(), 3); // refused allocation must not bump count
    }

    #[test]
    fn hash_data_round_trips_through_read_hash_data() {
        let mut r = Reduction::<16>::new();
        let d = hash_atom(g(0xC0FFEE));
        let node = r.hash_data(&d).unwrap();
        assert_eq!(r.read_hash_data(node).unwrap(), d);
    }

    #[test]
    fn read_hash_data_rejects_a_plain_atom() {
        let mut r = Reduction::<16>::new();
        let a = r.atom(g(1)).unwrap();
        assert!(r.read_hash_data(a).is_none());
    }

    #[test]
    fn is_atom_is_pair_head_tail_atom_value() {
        let mut r = Reduction::<16>::new();
        let a = r.atom(g(9)).unwrap();
        let b = r.atom(g(10)).unwrap();
        let p = r.pair(a, b).unwrap();

        assert!(r.is_atom(a) && !r.is_pair(a));
        assert!(r.is_pair(p) && !r.is_atom(p));
        assert_eq!(r.head(p), Some(a));
        assert_eq!(r.tail(p), Some(b));
        assert_eq!(r.atom_value(a), Some(g(9)));
        assert_eq!(r.atom_value(p), None);
        assert_eq!(r.head(a), None);
    }

    // compute_pair_bound dispatch — each arm below matches the same tag
    // that data::cost::PATTERN_COSTS (and reduce::COSTS, locked together
    // in row 211) assigns that pattern.
    mod compute_pair_bound {
        use super::*;

        fn tagged_pair<const N: usize>(r: &mut Reduction<N>, tag: u64, body: Order) -> Order {
            let t = r.atom(g(tag)).unwrap();
            r.pair(t, body).unwrap()
        }

        #[test]
        fn no_sub_formula_tag_is_exact_base() {
            let mut r = Reduction::<64>::new();
            let body = r.atom(g(0)).unwrap();
            let p = tagged_pair(&mut r, 1, body); // 1 = quote
            assert_eq!(r.get(p).unwrap().bound, Cost::Exact(PATTERN_COSTS[1]));
        }

        #[test]
        fn compose_tag_is_dynamic() {
            let mut r = Reduction::<64>::new();
            let x = r.atom(g(0)).unwrap();
            let y = r.atom(g(0)).unwrap();
            let xy = r.pair(x, y).unwrap();
            let p = tagged_pair(&mut r, 2, xy); // 2 = compose
            assert!(r.get(p).unwrap().bound.is_dynamic());
        }

        #[test]
        fn binary_structural_tag_sums_children() {
            let mut r = Reduction::<64>::new();
            let a = r.atom(g(0)).unwrap();
            let b = r.atom(g(0)).unwrap();
            let ab = r.pair(a, b).unwrap();
            let p = tagged_pair(&mut r, 5, ab); // 5 = add
            assert_eq!(r.get(p).unwrap().bound, Cost::Exact(PATTERN_COSTS[5]));
        }

        #[test]
        fn branch_tag_takes_max_of_arms() {
            let mut r = Reduction::<64>::new();
            // Build a deep `yes` branch (tag 5, add) so its bound exceeds `no`'s.
            let x = r.atom(g(0)).unwrap();
            let y = r.atom(g(0)).unwrap();
            let xy = r.pair(x, y).unwrap();
            let yes = tagged_pair(&mut r, 5, xy);
            let no = r.atom(g(0)).unwrap();
            let test = r.atom(g(0)).unwrap();
            let rest = r.pair(yes, no).unwrap();
            let body = r.pair(test, rest).unwrap();
            let p = tagged_pair(&mut r, 4, body); // 4 = branch

            let expected = Cost::branch(
                PATTERN_COSTS[4],
                Cost::Exact(0), // test is a bare atom, bound 0
                r.get(yes).unwrap().bound,
                Cost::Exact(0), // no is a bare atom, bound 0
            );
            assert_eq!(r.get(p).unwrap().bound, expected);
        }

        #[test]
        fn unary_tag_uses_sum1() {
            let mut r = Reduction::<64>::new();
            let inner = r.atom(g(0)).unwrap();
            let p = tagged_pair(&mut r, 8, inner); // 8 = inv
            assert_eq!(r.get(p).unwrap().bound, Cost::Exact(PATTERN_COSTS[8]));
        }

        #[test]
        fn call_tag_is_dynamic() {
            let mut r = Reduction::<64>::new();
            let tag_f = r.atom(g(0)).unwrap();
            let check_f = r.atom(g(0)).unwrap();
            let body = r.pair(tag_f, check_f).unwrap();
            let p = tagged_pair(&mut r, 16, body); // 16 = call
            assert!(r.get(p).unwrap().bound.is_dynamic());
        }

        #[test]
        fn unknown_tag_is_exact_zero() {
            let mut r = Reduction::<64>::new();
            let body = r.atom(g(0)).unwrap();
            let p = tagged_pair(&mut r, 99, body); // no pattern uses tag 99
            assert_eq!(r.get(p).unwrap().bound, Cost::Exact(0));
        }

        #[test]
        fn left_child_not_an_atom_is_exact_zero() {
            // compute_pair_bound treats a pair as data, not a formula, when
            // its own left child isn't an atom (no tag to dispatch on).
            let mut r = Reduction::<64>::new();
            let a = r.atom(g(0)).unwrap();
            let b = r.atom(g(0)).unwrap();
            let left = r.pair(a, b).unwrap(); // pair, not an atom tag
            let right = r.atom(g(0)).unwrap();
            let p = r.pair(left, right).unwrap();
            assert_eq!(r.get(p).unwrap().bound, Cost::Exact(0));
        }
    }
}
