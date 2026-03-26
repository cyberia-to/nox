//! pattern 0: axis — navigate the noun tree
//! axis(s, 0) = H(s) hash introspection
//! axis(s, 1) = s identity
//! axis(s, 2n) = head(axis(s, n))
//! axis(s, 2n+1) = tail(axis(s, n))

use crate::noun::{Arena, NounRef, NounInner};
use crate::reduce::{Outcome, ErrorKind};

pub fn axis<const N: usize>(arena: &mut Arena<N>, subject: NounRef, addr_ref: NounRef, budget: u64) -> Outcome {
    let addr = match arena.atom_value(addr_ref) {
        Some((v, _)) => v.as_u64(),
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    match addr {
        0 => {
            let digest = *arena.digest(subject);
            match arena.hash_noun(&digest) {
                Some(r) => Outcome::Ok(r, budget),
                None => Outcome::Error(ErrorKind::Unavailable),
            }
        }
        1 => Outcome::Ok(subject, budget),
        _ => {
            let bits = 64 - addr.leading_zeros() - 1;
            let mut node = subject;
            for i in (0..bits).rev() {
                match arena.get(node).inner {
                    NounInner::Cell { left, right } => {
                        node = if (addr >> i) & 1 == 1 { right } else { left };
                    }
                    _ => return Outcome::Error(ErrorKind::AxisError),
                }
            }
            Outcome::Ok(node, budget)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reduce::reduce;
    use crate::hint::NullHints;
    use crate::noun::{Arena, Tag};
    use nebu::Goldilocks;

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    fn make_axis<const N: usize>(arena: &mut Arena<N>, n: u64) -> NounRef {
        let tag = arena.atom(g(0), Tag::Field).unwrap();
        let addr = arena.atom(g(n), Tag::Field).unwrap();
        arena.cell(tag, addr).unwrap()
    }

    #[test]
    fn axis_head() {
        let mut ar = Arena::<1024>::new();
        let a = ar.atom(g(10), Tag::Field).unwrap();
        let b = ar.atom(g(20), Tag::Field).unwrap();
        let s = ar.cell(a, b).unwrap();
        let f = make_axis(&mut ar, 2);
        match reduce(&mut ar, s, f, 100, &NullHints) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(10)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn axis_tail() {
        let mut ar = Arena::<1024>::new();
        let a = ar.atom(g(10), Tag::Field).unwrap();
        let b = ar.atom(g(20), Tag::Field).unwrap();
        let s = ar.cell(a, b).unwrap();
        let f = make_axis(&mut ar, 3);
        match reduce(&mut ar, s, f, 100, &NullHints) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(20)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn axis_identity() {
        let mut ar = Arena::<1024>::new();
        let s = ar.atom(g(42), Tag::Field).unwrap();
        let f = make_axis(&mut ar, 1);
        match reduce(&mut ar, s, f, 100, &NullHints) {
            Outcome::Ok(r, _) => assert_eq!(r, s),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn axis_zero_hash_introspection() {
        let mut ar = Arena::<1024>::new();
        let s = ar.atom(g(42), Tag::Field).unwrap();
        let f = make_axis(&mut ar, 0);
        match reduce(&mut ar, s, f, 100, &NullHints) {
            Outcome::Ok(r, _) => {
                assert!(ar.is_cell(r));
                let d = ar.read_hash_noun(r).unwrap();
                assert_eq!(d, *ar.digest(s));
            }
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn axis_on_atom_errors() {
        let mut ar = Arena::<1024>::new();
        let s = ar.atom(g(42), Tag::Field).unwrap();
        let f = make_axis(&mut ar, 2); // head of atom → error
        match reduce(&mut ar, s, f, 100, &NullHints) {
            Outcome::Error(ErrorKind::AxisError) => {}
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn axis_deep_navigation() {
        let mut ar = Arena::<1024>::new();
        // s = [[1, 2], [3, 4]]
        let a = ar.atom(g(1), Tag::Field).unwrap();
        let b = ar.atom(g(2), Tag::Field).unwrap();
        let c = ar.atom(g(3), Tag::Field).unwrap();
        let d = ar.atom(g(4), Tag::Field).unwrap();
        let left = ar.cell(a, b).unwrap();
        let right = ar.cell(c, d).unwrap();
        let s = ar.cell(left, right).unwrap();
        // axis 7 = tail of tail = right of right = 4
        let f = make_axis(&mut ar, 7);
        match reduce(&mut ar, s, f, 100, &NullHints) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(4)),
            o => panic!("{:?}", o),
        }
    }
}
