//! data — the universal value type
//!
//! everything in nox is data: atom(field) | pair(data, data)
//! stored in a flat order with hash-consing (DAG, not tree)
//!
//! hash data are structured pairs (4 field elements) = pair(pair(h0,h1), pair(h2,h3))

pub mod inner;
pub mod hash;
pub mod reduction;
pub mod cost;

pub use inner::Data;
pub use hash::{Digest, digest_bytes, digest_from_bytes, hash_atom, hash_pair};
pub use reduction::{Reduction, DataEntry};
pub use cost::Cost;

/// order id — an index into the order (arena); every data node is identified
/// by its u32 slot. the portable identity of data is its `particle`.
pub type Order = u32;

/// sentinel: no data
pub const NIL: Order = u32::MAX;

#[cfg(test)]
mod tests {
    use super::*;
    use nebu::Goldilocks;

    #[test]
    fn atom_allocation() {
        let mut reduction = Reduction::<1024>::new();
        let a = reduction.atom(Goldilocks::new(42)).unwrap();
        assert!(reduction.is_atom(a));
        let val = reduction.atom_value(a).unwrap();
        assert_eq!(val, Goldilocks::new(42));
    }

    #[test]
    fn pair_allocation() {
        let mut reduction = Reduction::<1024>::new();
        let a = reduction.atom(Goldilocks::new(1)).unwrap();
        let b = reduction.atom(Goldilocks::new(2)).unwrap();
        let c = reduction.pair(a, b).unwrap();
        assert!(reduction.is_pair(c));
        assert_eq!(reduction.head(c), Some(a));
        assert_eq!(reduction.tail(c), Some(b));
    }

    #[test]
    fn hash_consing_atoms() {
        let mut reduction = Reduction::<1024>::new();
        let a = reduction.atom(Goldilocks::new(42)).unwrap();
        let b = reduction.atom(Goldilocks::new(42)).unwrap();
        assert_eq!(a, b);
        assert_eq!(reduction.count(), 1);
    }

    #[test]
    fn hash_consing_pairs() {
        let mut reduction = Reduction::<1024>::new();
        let x = reduction.atom(Goldilocks::new(1)).unwrap();
        let y = reduction.atom(Goldilocks::new(2)).unwrap();
        let c1 = reduction.pair(x, y).unwrap();
        let c2 = reduction.pair(x, y).unwrap();
        assert_eq!(c1, c2);
        assert_eq!(reduction.count(), 3);
    }

    #[test]
    fn hash_data_roundtrip() {
        let mut reduction = Reduction::<1024>::new();
        let d = [Goldilocks::new(11), Goldilocks::new(22), Goldilocks::new(33), Goldilocks::new(44)];
        let h = reduction.hash_data(&d).unwrap();
        assert!(reduction.is_pair(h));
        assert_eq!(reduction.read_hash_data(h).unwrap(), d);
    }

    #[test]
    fn hash_data_is_hash_consed() {
        let mut reduction = Reduction::<1024>::new();
        let d = [Goldilocks::new(1), Goldilocks::new(2), Goldilocks::new(3), Goldilocks::new(4)];
        assert_eq!(reduction.hash_data(&d).unwrap(), reduction.hash_data(&d).unwrap());
    }

    /// alloc_raw returns None past the load-factor cap (3/4 of N).
    /// Sized N=16 → cap at 12; allocate 12 distinct atoms, then the 13th fails.
    #[test]
    fn order_full_returns_none() {
        let mut reduction = Reduction::<16>::new();
        for k in 0..12 {
            assert!(reduction.atom(Goldilocks::new(k)).is_some());
        }
        // 13th allocation hits load-factor cap
        assert!(reduction.atom(Goldilocks::new(99)).is_none());
    }

    /// get(invalid_id) returns None instead of panicking.
    #[test]
    fn get_out_of_bounds_returns_none() {
        let reduction = Reduction::<1024>::new();
        assert!(reduction.get(NIL).is_none());
        assert!(reduction.get(42).is_none());  // never allocated
    }

    /// read_hash_data on an atom (not the [[h0,h1],[h2,h3]] shape) returns None.
    #[test]
    fn read_hash_data_on_atom_returns_none() {
        let mut reduction = Reduction::<1024>::new();
        let a = reduction.atom(Goldilocks::new(42)).unwrap();
        assert!(reduction.read_hash_data(a).is_none());
    }

    /// read_hash_data on a malformed pair shape (one level instead of two)
    /// returns None.
    #[test]
    fn read_hash_data_on_wrong_shape_returns_none() {
        let mut reduction = Reduction::<1024>::new();
        let a = reduction.atom(Goldilocks::new(1)).unwrap();
        let b = reduction.atom(Goldilocks::new(2)).unwrap();
        let pair = reduction.pair(a, b).unwrap();  // depth-1 pair, not [[h0,h1],[h2,h3]]
        assert!(reduction.read_hash_data(pair).is_none());
    }

    /// digest on an invalid Order returns None.
    #[test]
    fn digest_on_invalid_id_returns_none() {
        let reduction = Reduction::<1024>::new();
        assert!(reduction.digest(NIL).is_none());
    }
}
