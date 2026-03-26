//! noun — the universal data type
//!
//! everything in nox is a noun: atom(F) | cell(noun, noun)
//! stored in a flat arena with hash-consing (DAG, not tree)
//!
//! hash nouns (4 field elements) = cell(cell(h0,h1), cell(h2,h3))

pub mod tag;
pub mod inner;
pub mod hash;
pub mod arena;

pub use tag::Tag;
pub use inner::NounInner;
pub use hash::Digest;
pub use arena::{Arena, NounEntry};

/// arena index — all noun references are u32 indices
pub type NounRef = u32;

/// sentinel: no noun
pub const NIL: NounRef = u32::MAX;

#[cfg(test)]
mod tests {
    use super::*;
    use nebu::Goldilocks;

    #[test]
    fn atom_allocation() {
        let mut arena = Arena::<1024>::new();
        let a = arena.atom(Goldilocks::new(42), Tag::Field).unwrap();
        assert!(arena.is_atom(a));
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
    }

    #[test]
    fn hash_noun_roundtrip() {
        let mut arena = Arena::<1024>::new();
        let d = [Goldilocks::new(11), Goldilocks::new(22), Goldilocks::new(33), Goldilocks::new(44)];
        let h = arena.hash_noun(&d).unwrap();
        assert!(arena.is_cell(h));
        assert_eq!(arena.read_hash_noun(h).unwrap(), d);
    }

    #[test]
    fn hash_noun_is_hash_consed() {
        let mut arena = Arena::<1024>::new();
        let d = [Goldilocks::new(1), Goldilocks::new(2), Goldilocks::new(3), Goldilocks::new(4)];
        assert_eq!(arena.hash_noun(&d).unwrap(), arena.hash_noun(&d).unwrap());
    }
}
