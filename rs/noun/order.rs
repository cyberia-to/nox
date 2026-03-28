//! flat order for noun allocation with hash-consing
//!
//! one order per order() invocation. freed when order() returns.
//! hash-consing ensures identical sub-expressions share one slot.
//! DAG, not tree — immutable nouns, safe structural sharing.

use nebu::Goldilocks;
use super::tag::Tag;
use super::inner::Noun;
use super::hash::{Digest, hash_atom, hash_cell};
use super::{NounId, NIL};

/// order entry — noun + cached identity hash
#[derive(Debug, Clone, Copy)]
pub struct NounEntry {
    pub inner: Noun,
    pub hash: Digest,
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

impl<const N: usize> Order<N> {
    pub fn new() -> Self {
        assert!(N.is_power_of_two(), "order size must be power of 2");
        Self {
            // SAFETY: MaybeUninit<T> does not require initialization
            entries: unsafe { core::mem::MaybeUninit::uninit().assume_init() },
            count: 0,
            index_keys: [[Goldilocks::ZERO; 4]; N],
            index_vals: [NIL; N],
            index_mask: (N as u32) - 1,
        }
    }

    fn alloc_raw(&mut self, entry: NounEntry) -> Option<NounId> {
        if (self.count as usize) >= N - 1 { return None; }
        let idx = self.count;
        self.entries[idx as usize] = core::mem::MaybeUninit::new(entry);
        self.count += 1;
        Some(idx)
    }

    pub fn get(&self, r: NounId) -> &NounEntry {
        assert!((r as usize) < self.count as usize, "NounId out of bounds");
        // SAFETY: entries[0..count] are initialized, r < count
        unsafe { self.entries[r as usize].assume_init_ref() }
    }

    pub fn atom(&mut self, value: Goldilocks, tag: Tag) -> Option<NounId> {
        let inner = Noun::Atom { value, tag };
        let hash = hash_atom(value, tag);
        if let Some(existing) = self.index_lookup(&hash) { return Some(existing); }
        let r = self.alloc_raw(NounEntry { inner, hash })?;
        self.index_insert(&hash, r);
        Some(r)
    }

    pub fn cell(&mut self, left: NounId, right: NounId) -> Option<NounId> {
        let lh = self.get(left).hash;
        let rh = self.get(right).hash;
        let hash = hash_cell(&lh, &rh);
        if let Some(existing) = self.index_lookup(&hash) { return Some(existing); }
        let inner = Noun::Cell { left, right };
        let r = self.alloc_raw(NounEntry { inner, hash })?;
        self.index_insert(&hash, r);
        Some(r)
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
        let (left, right) = match self.get(r).inner {
            Noun::Cell { left, right } => (left, right),
            _ => return None,
        };
        let (h0r, h1r) = match self.get(left).inner {
            Noun::Cell { left, right } => (left, right),
            _ => return None,
        };
        let (h2r, h3r) = match self.get(right).inner {
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
        matches!(self.get(r).inner, Noun::Atom { .. })
    }

    pub fn is_cell(&self, r: NounId) -> bool {
        matches!(self.get(r).inner, Noun::Cell { .. })
    }

    pub fn head(&self, r: NounId) -> Option<NounId> {
        match self.get(r).inner { Noun::Cell { left, .. } => Some(left), _ => None }
    }

    pub fn tail(&self, r: NounId) -> Option<NounId> {
        match self.get(r).inner { Noun::Cell { right, .. } => Some(right), _ => None }
    }

    pub fn atom_value(&self, r: NounId) -> Option<(Goldilocks, Tag)> {
        match self.get(r).inner { Noun::Atom { value, tag } => Some((value, tag)), _ => None }
    }

    pub fn digest(&self, r: NounId) -> &Digest { &self.get(r).hash }
}
