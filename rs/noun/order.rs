//! flat order for noun allocation with hash-consing
//!
//! one order per order() invocation. freed when order() returns.
//! hash-consing ensures identical sub-expressions share one slot.
//! DAG, not tree — immutable nouns, safe structural sharing.

use nebu::Goldilocks;
use super::tag::Tag;
use super::inner::Noun;
use super::hash::{Digest, hash_atom, hash_cell};
use super::cost::{Cost, PATTERN_COSTS};
use super::{NounId, NIL};

/// order entry — noun + cached identity hash + cached cost bound.
///
/// `bound` is computed at construction time so reduce-time partition decisions
/// (`bound(child) ≤ budget?`) are O(1). The bound represents the noun treated
/// as a formula; atoms always have `Cost::Exact(0)`.
#[derive(Debug, Clone, Copy)]
pub struct NounEntry {
    pub inner: Noun,
    pub hash: Digest,
    pub bound: Cost,
}

/// flat order with hash-consing
pub struct Order<const N: usize> {
    // SAFETY: entries[0..count] are initialized
    entries: [core::mem::MaybeUninit<NounEntry>; N],
    count: u32,
    index_keys: [Digest; N],
    index_vals: [NounId; N],
    index_mask: u32,
}

impl<const N: usize> Default for Order<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Order<N> {
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

    fn alloc_raw(&mut self, entry: NounEntry) -> Option<NounId> {
        // Cap load factor at 3/4 so linear probing stays O(1) expected and
        // bounded worst-case (~N/4 max probe). The hash-cons table lives in
        // the same Order; refusing past 3N/4 means index_insert always finds
        // an empty slot quickly.
        if (self.count as usize) >= (N / 4) * 3 { return None; }
        let idx = self.count;
        self.entries[idx as usize] = core::mem::MaybeUninit::new(entry);
        self.count += 1;
        Some(idx)
    }

    /// returns None for out-of-bounds NounId (never panics)
    pub fn get(&self, r: NounId) -> Option<&NounEntry> {
        if (r as usize) >= self.count as usize { return None; }
        // SAFETY: entries[0..count] are initialized, r < count
        Some(unsafe { self.entries[r as usize].assume_init_ref() })
    }

    pub fn atom(&mut self, value: Goldilocks, tag: Tag) -> Option<NounId> {
        let inner = Noun::Atom { value, tag };
        let hash = hash_atom(value, tag);
        if let Some(existing) = self.index_lookup(&hash) { return Some(existing); }
        // Atoms have no formula interpretation; bound is 0 by definition.
        let r = self.alloc_raw(NounEntry { inner, hash, bound: Cost::Exact(0) })?;
        self.index_insert(&hash, r);
        Some(r)
    }

    pub fn cell(&mut self, left: NounId, right: NounId) -> Option<NounId> {
        let lh = self.get(left)?.hash;
        let rh = self.get(right)?.hash;
        let hash = hash_cell(&lh, &rh);
        if let Some(existing) = self.index_lookup(&hash) { return Some(existing); }
        let inner = Noun::Cell { left, right };
        let bound = self.compute_cell_bound(left, right);
        let r = self.alloc_raw(NounEntry { inner, hash, bound })?;
        self.index_insert(&hash, r);
        Some(r)
    }

    /// Compute the cached cost bound for a cell at construction time.
    /// Treats the cell as a formula `[tag body]` when `left` is an atom in
    /// pattern range; otherwise bound is 0 (the cell is data, not code).
    /// Children's cached bounds are looked up in O(1).
    fn compute_cell_bound(&self, left: NounId, right: NounId) -> Cost {
        let left_entry = match self.get(left) { Some(e) => e, None => return Cost::Exact(0) };
        let tag = match left_entry.inner {
            Noun::Atom { value, .. } => value.as_u64(),
            _ => return Cost::Exact(0),
        };
        if (tag as usize) >= PATTERN_COSTS.len() { return Cost::Exact(0); }
        let base = PATTERN_COSTS[tag as usize];

        let child_bound = |id: NounId| -> Cost {
            self.get(id).map_or(Cost::Exact(0), |e| e.bound)
        };
        let pair = |id: NounId| -> Option<(NounId, NounId)> {
            match self.get(id)?.inner {
                Noun::Cell { left, right } => Some((left, right)),
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

    /// build hash noun: cell(cell(h0, h1), cell(h2, h3))
    pub fn hash_noun(&mut self, digest: &Digest) -> Option<NounId> {
        let h0 = self.atom(digest[0], Tag::Field)?;
        let h1 = self.atom(digest[1], Tag::Field)?;
        let h2 = self.atom(digest[2], Tag::Field)?;
        let h3 = self.atom(digest[3], Tag::Field)?;
        let left = self.cell(h0, h1)?;
        let right = self.cell(h2, h3)?;
        self.cell(left, right)
    }

    /// extract digest from hash noun
    pub fn read_hash_noun(&self, r: NounId) -> Option<Digest> {
        let (left, right) = match self.get(r)?.inner {
            Noun::Cell { left, right } => (left, right),
            _ => return None,
        };
        let (h0r, h1r) = match self.get(left)?.inner {
            Noun::Cell { left, right } => (left, right),
            _ => return None,
        };
        let (h2r, h3r) = match self.get(right)?.inner {
            Noun::Cell { left, right } => (left, right),
            _ => return None,
        };
        Some([
            self.atom_value(h0r)?.0,
            self.atom_value(h1r)?.0,
            self.atom_value(h2r)?.0,
            self.atom_value(h3r)?.0,
        ])
    }

    fn index_lookup(&self, hash: &Digest) -> Option<NounId> {
        let mut slot = (hash[0].as_u64() as u32) & self.index_mask;
        for _ in 0..N {
            let val = self.index_vals[slot as usize];
            if val == NIL { return None; }
            if self.index_keys[slot as usize] == *hash { return Some(val); }
            slot = (slot + 1) & self.index_mask;
        }
        None
    }

    fn index_insert(&mut self, hash: &Digest, r: NounId) {
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

    pub fn is_atom(&self, r: NounId) -> bool {
        self.get(r).is_some_and(|e| matches!(e.inner, Noun::Atom { .. }))
    }

    pub fn is_cell(&self, r: NounId) -> bool {
        self.get(r).is_some_and(|e| matches!(e.inner, Noun::Cell { .. }))
    }

    pub fn head(&self, r: NounId) -> Option<NounId> {
        match self.get(r)?.inner { Noun::Cell { left, .. } => Some(left), _ => None }
    }

    pub fn tail(&self, r: NounId) -> Option<NounId> {
        match self.get(r)?.inner { Noun::Cell { right, .. } => Some(right), _ => None }
    }

    pub fn atom_value(&self, r: NounId) -> Option<(Goldilocks, Tag)> {
        match self.get(r)?.inner { Noun::Atom { value, tag } => Some((value, tag)), _ => None }
    }

    pub fn digest(&self, r: NounId) -> Option<&Digest> {
        Some(&self.get(r)?.hash)
    }
}
