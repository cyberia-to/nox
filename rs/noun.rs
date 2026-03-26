// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! noun — the universal data type
//!
//! everything in nox is a noun: atom(F) | cell(noun, noun)
//! stored in a flat arena with hash-consing (DAG, not tree)
//!
//! hash noun (4 field elements) represented as cell(cell(h0,h1), cell(h2,h3))
//! — preserves atom|cell axiom without extending NounInner

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
}

/// atom or cell — the two kinds of noun. nothing else.
#[derive(Debug, Clone, Copy)]
pub enum NounInner {
    Atom { value: Goldilocks, tag: Tag },
    Cell { left: NounRef, right: NounRef },
}

/// hash identity — 4 Goldilocks elements = 32 bytes (truncated from hemera's 64-byte output)
/// 128-bit collision security (birthday bound 2^64). intentional truncation per trace.md.
pub type Digest = [Goldilocks; 4];

/// arena entry — noun inner + cached identity hash
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
    // SAFETY: entries[0..count] are initialized. entries[count..N] are uninit.
    entries: [core::mem::MaybeUninit<NounEntry>; N],
    count: u32,
    // hash-cons index: open-addressing hash table, N slots
    // load factor approaches 1.0 near capacity — acceptable because
    // arena-full check in alloc_raw prevents insertion at 100%
    index_keys: [Digest; N],
    index_vals: [NounRef; N],
    index_mask: u32,
}

impl<const N: usize> Arena<N> {
    /// create a new empty arena
    pub fn new() -> Self {
        assert!(N.is_power_of_two(), "arena size must be power of 2");
        Self {
            // SAFETY: MaybeUninit<T> does not require initialization.
            // array of MaybeUninit is always valid to create uninit.
            entries: unsafe { core::mem::MaybeUninit::uninit().assume_init() },
            count: 0,
            index_keys: [[Goldilocks::ZERO; 4]; N],
            index_vals: [NIL; N],
            index_mask: (N as u32) - 1,
        }
    }

    /// allocate a raw noun (no hash-consing check)
    fn alloc_raw(&mut self, entry: NounEntry) -> Option<NounRef> {
        if (self.count as usize) >= N - 1 {
            return None; // leave one slot empty for hash table termination
        }
        let idx = self.count;
        self.entries[idx as usize] = core::mem::MaybeUninit::new(entry);
        self.count += 1;
        Some(idx)
    }

    /// get noun entry by reference
    pub fn get(&self, r: NounRef) -> &NounEntry {
        assert!((r as usize) < self.count as usize, "NounRef out of bounds");
        // SAFETY: entries[0..count] are initialized, and r < count
        unsafe { self.entries[r as usize].assume_init_ref() }
    }

    /// allocate a field-type atom
    pub fn atom(&mut self, value: Goldilocks, tag: Tag) -> Option<NounRef> {
        let inner = NounInner::Atom { value, tag };
        let hash = hash_atom(value, tag);
        if let Some(existing) = self.index_lookup(&hash) {
            return Some(existing);
        }
        let r = self.alloc_raw(NounEntry { inner, hash })?;
        self.index_insert(&hash, r);
        Some(r)
    }

    /// allocate a cell (hash-consed)
    pub fn cell(&mut self, left: NounRef, right: NounRef) -> Option<NounRef> {
        let lh = self.get(left).hash;
        let rh = self.get(right).hash;
        let hash = hash_cell(&lh, &rh);
        if let Some(existing) = self.index_lookup(&hash) {
            return Some(existing);
        }
        let inner = NounInner::Cell { left, right };
        let r = self.alloc_raw(NounEntry { inner, hash })?;
        self.index_insert(&hash, r);
        Some(r)
    }

    /// build a hash noun: cell(cell(h0, h1), cell(h2, h3))
    /// hash "atoms" are structured cells — preserves atom|cell axiom
    pub fn hash_noun(&mut self, digest: &Digest) -> Option<NounRef> {
        let h0 = self.atom(digest[0], Tag::Field)?;
        let h1 = self.atom(digest[1], Tag::Field)?;
        let h2 = self.atom(digest[2], Tag::Field)?;
        let h3 = self.atom(digest[3], Tag::Field)?;
        let left = self.cell(h0, h1)?;
        let right = self.cell(h2, h3)?;
        self.cell(left, right)
    }

    /// extract digest from a hash noun: cell(cell(h0, h1), cell(h2, h3)) → [h0, h1, h2, h3]
    pub fn read_hash_noun(&self, r: NounRef) -> Option<Digest> {
        let (left, right) = match self.get(r).inner {
            NounInner::Cell { left, right } => (left, right),
            _ => return None,
        };
        let (h0r, h1r) = match self.get(left).inner {
            NounInner::Cell { left, right } => (left, right),
            _ => return None,
        };
        let (h2r, h3r) = match self.get(right).inner {
            NounInner::Cell { left, right } => (left, right),
            _ => return None,
        };
        let h0 = self.atom_value(h0r)?.0;
        let h1 = self.atom_value(h1r)?.0;
        let h2 = self.atom_value(h2r)?.0;
        let h3 = self.atom_value(h3r)?.0;
        Some([h0, h1, h2, h3])
    }

    /// hash-cons index lookup
    fn index_lookup(&self, hash: &Digest) -> Option<NounRef> {
        let mut slot = (hash[0].as_u64() as u32) & self.index_mask;
        for _ in 0..N {
            let val = self.index_vals[slot as usize];
            if val == NIL { return None; }
            if self.index_keys[slot as usize] == *hash { return Some(val); }
            slot = (slot + 1) & self.index_mask;
        }
        None // table full — should not happen due to alloc_raw guard
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

    pub fn count(&self) -> u32 { self.count }

    pub fn is_atom(&self, r: NounRef) -> bool {
        matches!(self.get(r).inner, NounInner::Atom { .. })
    }

    pub fn is_cell(&self, r: NounRef) -> bool {
        matches!(self.get(r).inner, NounInner::Cell { .. })
    }

    pub fn head(&self, r: NounRef) -> Option<NounRef> {
        match self.get(r).inner {
            NounInner::Cell { left, .. } => Some(left),
            _ => None,
        }
    }

    pub fn tail(&self, r: NounRef) -> Option<NounRef> {
        match self.get(r).inner {
            NounInner::Cell { right, .. } => Some(right),
            _ => None,
        }
    }

    pub fn atom_value(&self, r: NounRef) -> Option<(Goldilocks, Tag)> {
        match self.get(r).inner {
            NounInner::Atom { value, tag } => Some((value, tag)),
            _ => None,
        }
    }

    pub fn digest(&self, r: NounRef) -> &Digest {
        &self.get(r).hash
    }
}

/// hash an atom using hemera tree leaf mode (capacity-based domain separation)
fn hash_atom(value: Goldilocks, tag: Tag) -> Digest {
    let mut data = [0u8; 9];
    data[0..8].copy_from_slice(&value.as_u64().to_le_bytes());
    data[8] = tag as u8;
    // hemera::tree::hash_leaf sets capacity[9] = FLAG_CHUNK
    // counter = tag as domain separator in the capacity region
    let h = hemera::tree::hash_leaf(&data, tag as u64, false);
    extract_digest(&h)
}

/// hash a cell using hemera tree node mode (capacity-based domain separation)
fn hash_cell(left: &Digest, right: &Digest) -> Digest {
    let lh = pack_digest(left);
    let rh = pack_digest(right);
    // hemera::tree::hash_node sets capacity[9] = FLAG_PARENT
    let h = hemera::tree::hash_node(&lh, &rh, false);
    extract_digest(&h)
}

/// pack digest into hemera Hash for node hashing
fn pack_digest(d: &Digest) -> hemera::Hash {
    let mut bytes = [0u8; 64];
    for i in 0..4 {
        bytes[i * 8..(i + 1) * 8].copy_from_slice(&d[i].as_u64().to_le_bytes());
    }
    // pad remaining 32 bytes with zeros
    hemera::Hash::from_bytes(bytes)
}

/// extract first 4 Goldilocks elements from hemera 64-byte output
/// intentional truncation: 128-bit collision security (per trace.md)
fn extract_digest(h: &hemera::Hash) -> Digest {
    let bytes = h.as_bytes();
    let mut digest = [Goldilocks::ZERO; 4];
    for i in 0..4 {
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&bytes[i * 8..(i + 1) * 8]);
        digest[i] = Goldilocks::new(u64::from_le_bytes(buf));
    }
    digest
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atom_allocation() {
        let mut arena = Arena::<1024>::new();
        let a = arena.atom(Goldilocks::new(42), Tag::Field).unwrap();
        assert!(arena.is_atom(a));
        assert!(!arena.is_cell(a));
        let (val, tag) = arena.atom_value(a).unwrap();
        assert_eq!(val, Goldilocks::new(42));
        assert_eq!(tag, Tag::Field);
    }

    #[test]
    fn cell_allocation() {
        let mut arena = Arena::<1024>::new();
        let a = arena.atom(Goldilocks::new(1), Tag::Field).unwrap();
        let b = arena.atom(Goldilocks::new(2), Tag::Field).unwrap();
        let c = arena.cell(a, b).unwrap();
        assert!(arena.is_cell(c));
        assert_eq!(arena.head(c), Some(a));
        assert_eq!(arena.tail(c), Some(b));
    }

    #[test]
    fn hash_consing_atoms() {
        let mut arena = Arena::<1024>::new();
        let a = arena.atom(Goldilocks::new(42), Tag::Field).unwrap();
        let b = arena.atom(Goldilocks::new(42), Tag::Field).unwrap();
        assert_eq!(a, b);
        assert_eq!(arena.count(), 1);
    }

    #[test]
    fn hash_consing_cells() {
        let mut arena = Arena::<1024>::new();
        let x = arena.atom(Goldilocks::new(1), Tag::Field).unwrap();
        let y = arena.atom(Goldilocks::new(2), Tag::Field).unwrap();
        let c1 = arena.cell(x, y).unwrap();
        let c2 = arena.cell(x, y).unwrap();
        assert_eq!(c1, c2);
        assert_eq!(arena.count(), 3);
    }

    #[test]
    fn different_tags_different_nouns() {
        let mut arena = Arena::<1024>::new();
        let a = arena.atom(Goldilocks::new(42), Tag::Field).unwrap();
        let b = arena.atom(Goldilocks::new(42), Tag::Word).unwrap();
        assert_ne!(a, b);
        assert_eq!(arena.count(), 2);
    }

    #[test]
    fn hash_noun_roundtrip() {
        let mut arena = Arena::<1024>::new();
        let d = [Goldilocks::new(11), Goldilocks::new(22), Goldilocks::new(33), Goldilocks::new(44)];
        let h = arena.hash_noun(&d).unwrap();
        assert!(arena.is_cell(h));
        let read = arena.read_hash_noun(h).unwrap();
        assert_eq!(read, d);
    }

    #[test]
    fn hash_noun_is_hash_consed() {
        let mut arena = Arena::<1024>::new();
        let d = [Goldilocks::new(1), Goldilocks::new(2), Goldilocks::new(3), Goldilocks::new(4)];
        let h1 = arena.hash_noun(&d).unwrap();
        let h2 = arena.hash_noun(&d).unwrap();
        assert_eq!(h1, h2); // same digest → same NounRef
    }
}
