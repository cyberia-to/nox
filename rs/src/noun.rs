// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! noun — the universal data type
//!
//! everything in nox is a noun: atom(F) | cell(noun, noun)
//! stored in a flat arena with hash-consing (DAG, not tree)

use nebu::Goldilocks;

/// arena index — all noun references are u32 indices into the arena
pub type NounRef = u32;

/// sentinel: no noun
pub const NIL: NounRef = u32::MAX;

/// type tag for atoms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Tag {
    Field = 0x00,
    Word = 0x01,
    Hash = 0x02,
}

/// atom or cell — the two kinds of noun
#[derive(Debug, Clone, Copy)]
pub enum NounInner {
    Atom {
        value: Goldilocks,
        tag: Tag,
    },
    Cell {
        left: NounRef,
        right: NounRef,
    },
}

/// hash digest — 4 Goldilocks elements = 32 bytes
pub type Digest = [Goldilocks; 4];

/// arena entry — noun inner + cached hash
#[derive(Debug, Clone, Copy)]
pub struct NounEntry {
    pub inner: NounInner,
    pub hash: Digest,
}

/// flat arena for noun allocation with hash-consing
///
/// one arena per ask() invocation. freed when ask() returns.
/// hash-consing ensures identical sub-expressions share one slot.
pub struct Arena<const N: usize> {
    entries: [core::mem::MaybeUninit<NounEntry>; N],
    count: u32,
    // hash-cons index: maps truncated hash → NounRef
    // simple open-addressing hash table
    index_keys: [Digest; N],
    index_vals: [NounRef; N],
    index_mask: u32,
}

impl<const N: usize> Arena<N> {
    /// create a new empty arena
    pub fn new() -> Self {
        // N must be power of 2 for hash table
        debug_assert!(N.is_power_of_two());
        Self {
            entries: unsafe { core::mem::MaybeUninit::uninit().assume_init() },
            count: 0,
            index_keys: [[Goldilocks::ZERO; 4]; N],
            index_vals: [NIL; N],
            index_mask: (N as u32) - 1,
        }
    }

    /// allocate a raw noun (no hash-consing check)
    fn alloc_raw(&mut self, entry: NounEntry) -> Option<NounRef> {
        if (self.count as usize) >= N {
            return None; // arena full
        }
        let idx = self.count;
        self.entries[idx as usize] = core::mem::MaybeUninit::new(entry);
        self.count += 1;
        Some(idx)
    }

    /// get noun entry by reference
    pub fn get(&self, r: NounRef) -> &NounEntry {
        debug_assert!((r as usize) < self.count as usize);
        unsafe { self.entries[r as usize].assume_init_ref() }
    }

    /// allocate an atom
    pub fn atom(&mut self, value: Goldilocks, tag: Tag) -> Option<NounRef> {
        let inner = NounInner::Atom { value, tag };
        let hash = hash_atom(value, tag);

        // check hash-cons index
        if let Some(existing) = self.index_lookup(&hash) {
            return Some(existing);
        }

        let r = self.alloc_raw(NounEntry { inner, hash })?;
        self.index_insert(&hash, r);
        Some(r)
    }

    /// allocate a cell (hash-consed)
    pub fn cell(&mut self, left: NounRef, right: NounRef) -> Option<NounRef> {
        let lh = &self.get(left).hash;
        let rh = &self.get(right).hash;
        let hash = hash_cell(lh, rh);

        // check hash-cons index — identical cells reuse existing slot
        if let Some(existing) = self.index_lookup(&hash) {
            return Some(existing);
        }

        let inner = NounInner::Cell { left, right };
        let r = self.alloc_raw(NounEntry { inner, hash })?;
        self.index_insert(&hash, r);
        Some(r)
    }

    /// hash-cons index lookup
    fn index_lookup(&self, hash: &Digest) -> Option<NounRef> {
        let mut slot = (hash[0].as_u64() as u32) & self.index_mask;
        loop {
            let val = self.index_vals[slot as usize];
            if val == NIL {
                return None;
            }
            if self.index_keys[slot as usize] == *hash {
                return Some(val);
            }
            slot = (slot + 1) & self.index_mask;
        }
    }

    /// hash-cons index insert
    fn index_insert(&mut self, hash: &Digest, r: NounRef) {
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

    /// how many nouns allocated
    pub fn count(&self) -> u32 {
        self.count
    }

    /// is this noun an atom?
    pub fn is_atom(&self, r: NounRef) -> bool {
        matches!(self.get(r).inner, NounInner::Atom { .. })
    }

    /// is this noun a cell?
    pub fn is_cell(&self, r: NounRef) -> bool {
        matches!(self.get(r).inner, NounInner::Cell { .. })
    }

    /// get head (left child) of a cell
    pub fn head(&self, r: NounRef) -> Option<NounRef> {
        match self.get(r).inner {
            NounInner::Cell { left, .. } => Some(left),
            NounInner::Atom { .. } => None,
        }
    }

    /// get tail (right child) of a cell
    pub fn tail(&self, r: NounRef) -> Option<NounRef> {
        match self.get(r).inner {
            NounInner::Cell { right, .. } => Some(right),
            NounInner::Atom { .. } => None,
        }
    }

    /// get atom value
    pub fn atom_value(&self, r: NounRef) -> Option<(Goldilocks, Tag)> {
        match self.get(r).inner {
            NounInner::Atom { value, tag } => Some((value, tag)),
            NounInner::Cell { .. } => None,
        }
    }

    /// get the hash/identity of a noun
    pub fn hash(&self, r: NounRef) -> &Digest {
        &self.get(r).hash
    }
}

/// hash an atom via hemera with domain separation
fn hash_atom(value: Goldilocks, tag: Tag) -> Digest {
    // hemera capacity[14] = type_tag, capacity[9] = FLAG_CHUNK
    // simplified: hash the value with tag as domain separator
    let mut data = [0u8; 16];
    data[0..8].copy_from_slice(&value.as_u64().to_le_bytes());
    data[8] = tag as u8;
    let h = hemera::hash(&data);
    extract_digest(&h)
}

/// hash a cell via hemera (hash two child hashes together)
fn hash_cell(left: &Digest, right: &Digest) -> Digest {
    // hemera capacity[9] = FLAG_PARENT
    let mut data = [0u8; 64];
    for i in 0..4 {
        data[i * 8..(i + 1) * 8].copy_from_slice(&left[i].as_u64().to_le_bytes());
    }
    for i in 0..4 {
        data[32 + i * 8..32 + (i + 1) * 8].copy_from_slice(&right[i].as_u64().to_le_bytes());
    }
    let h = hemera::hash(&data);
    extract_digest(&h)
}

/// extract 4 Goldilocks elements from hemera output
fn extract_digest(h: &hemera::Hash) -> Digest {
    let bytes = h.as_bytes();
    let mut digest = [Goldilocks::ZERO; 4];
    for i in 0..4 {
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&bytes[i * 8..(i + 1) * 8]);
        let val = u64::from_le_bytes(buf);
        digest[i] = Goldilocks::new(val);
    }
    digest
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atom_allocation() {
        let mut arena = Arena::<1024>::new();
        let a = arena.atom(Goldilocks::new(42), Tag::Field).unwrap();
        assert!(arena.is_atom(a));
        assert!(!arena.is_cell(a));
        let (val, tag) = arena.atom_value(a).unwrap();
        assert_eq!(val, Goldilocks::new(42));
        assert_eq!(tag, Tag::Field);
    }

    #[test]
    fn test_cell_allocation() {
        let mut arena = Arena::<1024>::new();
        let a = arena.atom(Goldilocks::new(1), Tag::Field).unwrap();
        let b = arena.atom(Goldilocks::new(2), Tag::Field).unwrap();
        let c = arena.cell(a, b).unwrap();
        assert!(arena.is_cell(c));
        assert_eq!(arena.head(c), Some(a));
        assert_eq!(arena.tail(c), Some(b));
    }

    #[test]
    fn test_hash_consing() {
        let mut arena = Arena::<1024>::new();
        let a = arena.atom(Goldilocks::new(42), Tag::Field).unwrap();
        let b = arena.atom(Goldilocks::new(42), Tag::Field).unwrap();
        // same value, same tag → same NounRef (hash-consed)
        assert_eq!(a, b);
        assert_eq!(arena.count(), 1);
    }

    #[test]
    fn test_cell_hash_consing() {
        let mut arena = Arena::<1024>::new();
        let x = arena.atom(Goldilocks::new(1), Tag::Field).unwrap();
        let y = arena.atom(Goldilocks::new(2), Tag::Field).unwrap();
        let c1 = arena.cell(x, y).unwrap();
        let c2 = arena.cell(x, y).unwrap();
        assert_eq!(c1, c2);
        assert_eq!(arena.count(), 3); // 2 atoms + 1 cell
    }

    #[test]
    fn test_different_tags_different_nouns() {
        let mut arena = Arena::<1024>::new();
        let a = arena.atom(Goldilocks::new(42), Tag::Field).unwrap();
        let b = arena.atom(Goldilocks::new(42), Tag::Word).unwrap();
        assert_ne!(a, b);
        assert_eq!(arena.count(), 2);
    }
}
