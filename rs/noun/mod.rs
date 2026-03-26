//! noun — the universal data type
//!
//! everything in nox is a noun: atom(F) | cell(noun, noun)
//! stored in a flat order with hash-consing (DAG, not tree)
//!
//! hash nouns (4 field elements) = cell(cell(h0,h1), cell(h2,h3))

pub mod tag;
pub mod inner;
pub mod hash;
pub mod order;

pub use tag::Tag;
pub use inner::Noun;
pub use hash::Digest;
pub use order::{Order, NounEntry};

/// order index — all noun identifiers are u32 indices
pub type NounId = u32;

/// sentinel: no noun
pub const NIL: NounId = u32::MAX;

#[cfg(test)]
mod tests {
    use super::*;
    use nebu::Goldilocks;

    #[test]
    fn atom_allocation() {
        let mut order = Order::<1024>::new();
        let a = order.atom(Goldilocks::new(42), Tag::Field).unwrap();
        assert!(order.is_atom(a));
        let (val, tag) = order.atom_value(a).unwrap();
        assert_eq!(val, Goldilocks::new(42));
        assert_eq!(tag, Tag::Field);
    }

    #[test]
    fn cell_allocation() {
        let mut order = Order::<1024>::new();
        let a = order.atom(Goldilocks::new(1), Tag::Field).unwrap();
        let b = order.atom(Goldilocks::new(2), Tag::Field).unwrap();
        let c = order.cell(a, b).unwrap();
        assert!(order.is_cell(c));
        assert_eq!(order.head(c), Some(a));
        assert_eq!(order.tail(c), Some(b));
    }

    #[test]
    fn hash_consing_atoms() {
        let mut order = Order::<1024>::new();
        let a = order.atom(Goldilocks::new(42), Tag::Field).unwrap();
        let b = order.atom(Goldilocks::new(42), Tag::Field).unwrap();
        assert_eq!(a, b);
        assert_eq!(order.count(), 1);
    }

    #[test]
    fn hash_consing_cells() {
        let mut order = Order::<1024>::new();
        let x = order.atom(Goldilocks::new(1), Tag::Field).unwrap();
        let y = order.atom(Goldilocks::new(2), Tag::Field).unwrap();
        let c1 = order.cell(x, y).unwrap();
        let c2 = order.cell(x, y).unwrap();
        assert_eq!(c1, c2);
        assert_eq!(order.count(), 3);
    }

    #[test]
    fn different_tags_different_nouns() {
        let mut order = Order::<1024>::new();
        let a = order.atom(Goldilocks::new(42), Tag::Field).unwrap();
        let b = order.atom(Goldilocks::new(42), Tag::Word).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn hash_noun_roundtrip() {
        let mut order = Order::<1024>::new();
        let d = [Goldilocks::new(11), Goldilocks::new(22), Goldilocks::new(33), Goldilocks::new(44)];
        let h = order.hash_noun(&d).unwrap();
        assert!(order.is_cell(h));
        assert_eq!(order.read_hash_noun(h).unwrap(), d);
    }

    #[test]
    fn hash_noun_is_hash_consed() {
        let mut order = Order::<1024>::new();
        let d = [Goldilocks::new(1), Goldilocks::new(2), Goldilocks::new(3), Goldilocks::new(4)];
        assert_eq!(order.hash_noun(&d).unwrap(), order.hash_noun(&d).unwrap());
    }
}
