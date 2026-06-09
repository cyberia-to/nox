//! pattern 0: axis — navigate the data tree
//! axis(s, 0) = H(s) hash introspection
//! axis(s, 1) = s identity
//! axis(s, 2n) = head(axis(s, n))
//! axis(s, 2n+1) = tail(axis(s, n))

use crate::call::CallProvider;
use crate::data::{Reduction, Order, Data};
use crate::reduce::{Outcome, ErrorKind};
use crate::trace::TraceRow;

pub fn axis<const N: usize>(
    reduction: &mut Reduction<N>, object: Order, addr_ref: Order, budget: u64,
    hints: &dyn CallProvider<N>,
    row: &mut TraceRow,
) -> Outcome {
    let addr = match reduction.atom_value(addr_ref) {
        Some(v) => v.as_u64(),
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    // r4     = first field element of Lens commitment (0 when prover not active)
    // r5     = axis address (raw u64)
    // r6     = levels descended (depth of navigation)
    // r7     = result Order
    // r11-r14 = full 32-byte Lens commitment split into 4 u64s (little-endian)
    //
    // When hints.axis_commitment() is provided, r4 = r11 = commitment[0..8].
    // When no prover is attached, r4 = 0 and r11-r14 = 0 — correct: there is
    // no commitment to populate. The zheng circuit reads r4/r11-r14 for the
    // opening proof; without a proof context these registers are unused.
    row.r[5] = addr;
    if let Some(bytes) = hints.axis_commitment(object as u64) {
        let w0 = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
        row.r[4]  = w0;
        row.r[11] = w0;
        row.r[12] = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        row.r[13] = u64::from_le_bytes(bytes[16..24].try_into().unwrap());
        row.r[14] = u64::from_le_bytes(bytes[24..32].try_into().unwrap());
    }
    match addr {
        0 => {
            let digest = match reduction.digest(object) {
                Some(d) => *d,
                None => return Outcome::Error(ErrorKind::Unavailable),
            };
            match reduction.hash_data(&digest) {
                Some(r) => {
                    row.r[6] = 0;
                    row.r[7] = r as u64;
                    Outcome::Ok(r, budget)
                }
                None => Outcome::Error(ErrorKind::Unavailable),
            }
        }
        1 => {
            row.r[6] = 0;
            row.r[7] = object as u64;
            Outcome::Ok(object, budget)
        }
        _ => {
            let bits = 64 - addr.leading_zeros() - 1;
            let mut node = object;
            let mut levels = 0u64;
            for i in (0..bits).rev() {
                match reduction.get(node) {
                    Some(e) => match e.inner {
                        Data::Pair { left, right } => {
                            node = if (addr >> i) & 1 == 1 { right } else { left };
                            levels += 1;
                        }
                        _ => return Outcome::Error(ErrorKind::AxisError),
                    },
                    None => return Outcome::Error(ErrorKind::Malformed),
                }
            }
            row.r[6] = levels;
            row.r[7] = node as u64;
            Outcome::Ok(node, budget)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reduce::{reduce, Outcome, ErrorKind};
    use crate::call::NullCalls;
    use crate::trace::NoTrace;
    use crate::data::{Reduction};
    use nebu::Goldilocks;

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    fn make_axis<const N: usize>(reduction: &mut Reduction<N>, n: u64) -> Order {
        let tag = reduction.atom(g(0)).unwrap();
        let addr = reduction.atom(g(n)).unwrap();
        reduction.pair(tag, addr).unwrap()
    }

    #[test]
    fn axis_head() {
        let mut ar = Reduction::<1024>::new();
        let a = ar.atom(g(10)).unwrap();
        let b = ar.atom(g(20)).unwrap();
        let s = ar.pair(a, b).unwrap();
        let f = make_axis(&mut ar, 2);
        match reduce(&mut ar, s, f, 100, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap(), g(10)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn axis_tail() {
        let mut ar = Reduction::<1024>::new();
        let a = ar.atom(g(10)).unwrap();
        let b = ar.atom(g(20)).unwrap();
        let s = ar.pair(a, b).unwrap();
        let f = make_axis(&mut ar, 3);
        match reduce(&mut ar, s, f, 100, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap(), g(20)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn axis_identity() {
        let mut ar = Reduction::<1024>::new();
        let s = ar.atom(g(42)).unwrap();
        let f = make_axis(&mut ar, 1);
        match reduce(&mut ar, s, f, 100, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(r, s),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn axis_zero_hash_introspection() {
        let mut ar = Reduction::<1024>::new();
        let s = ar.atom(g(42)).unwrap();
        let f = make_axis(&mut ar, 0);
        match reduce(&mut ar, s, f, 100, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => {
                assert!(ar.is_pair(r));
                let d = ar.read_hash_data(r).unwrap();
                assert_eq!(d, *ar.digest(s).unwrap());
            }
            o => panic!("{:?}", o),
        }
    }

    /// axis(s, 0) on a pair object also returns the pair's digest hashed.
    #[test]
    fn axis_zero_hash_on_pair() {
        let mut ar = Reduction::<1024>::new();
        let a = ar.atom(g(10)).unwrap();
        let b = ar.atom(g(20)).unwrap();
        let s = ar.pair(a, b).unwrap();
        let f = make_axis(&mut ar, 0);
        let s_digest = *ar.digest(s).unwrap();
        match reduce(&mut ar, s, f, 100, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => {
                assert!(ar.is_pair(r));
                let d = ar.read_hash_data(r).unwrap();
                assert_eq!(d, s_digest, "axis(pair, 0) returns hash data of pair digest");
            }
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn axis_on_atom_errors() {
        let mut ar = Reduction::<1024>::new();
        let s = ar.atom(g(42)).unwrap();
        let f = make_axis(&mut ar, 2); // head of atom → error
        match reduce(&mut ar, s, f, 100, &NullCalls, &mut NoTrace) {
            Outcome::Error(ErrorKind::AxisError) => {}
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn axis_deep_navigation() {
        let mut ar = Reduction::<1024>::new();
        // s = [[1, 2], [3, 4]]
        let a = ar.atom(g(1)).unwrap();
        let b = ar.atom(g(2)).unwrap();
        let c = ar.atom(g(3)).unwrap();
        let d = ar.atom(g(4)).unwrap();
        let left = ar.pair(a, b).unwrap();
        let right = ar.pair(c, d).unwrap();
        let s = ar.pair(left, right).unwrap();
        // axis 7 = tail of tail = right of right = 4
        let f = make_axis(&mut ar, 7);
        match reduce(&mut ar, s, f, 100, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap(), g(4)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn budget_zero_halts_immediately() {
        let mut ar = Reduction::<1024>::new();
        let s = ar.atom(g(42)).unwrap();
        let f = make_axis(&mut ar, 1);
        match reduce(&mut ar, s, f, 0, &NullCalls, &mut NoTrace) {
            Outcome::Halt(0) => {}
            o => panic!("expected Halt(0), got {:?}", o),
        }
    }
}
