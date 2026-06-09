//! pattern 9: eq — equality by data identity (hash comparison)
//!
//! works for ALL data types: atoms, pairs, hash data. returns 0 if equal,
//! 1 if not equal. r4 = left operand field value (atom: canonical value,
//! pair: digest[0]), r5 = right operand field value (same rule), r6 = result
//! (0=equal, 1=unequal), r7 = inverse hint: (r4-r5)^-1 when unequal, 0 when
//! equal. zheng constrains the gadget (r4 - r5) * r7 = r6 with r6 ∈ {0, 1}.

use nebu::Goldilocks;
use crate::data::{Reduction, Order};
use crate::reduce::{Outcome, ErrorKind, pair_children, evaluate_binary, make_field};
use crate::call::CallProvider;
use crate::trace::{Tracer, TraceRow};
use crate::jets::registry::JetRegistry;

pub fn eq<const N: usize, T: Tracer>(
    reduction: &mut Reduction<N>, object: Order, body: Order, budget: u64,
    hints: &dyn CallProvider<N>, tracer: &mut T, depth: u64,
    row: &mut TraceRow, registry: &JetRegistry<N>,
) -> Outcome {
    let (a, b) = match pair_children(reduction, body) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    let (ra, rb, budget) = match evaluate_binary(reduction, object, a, b, budget, hints, tracer, depth, registry) {
        Ok(v) => v, Err(o) => return o,
    };
    let da = match reduction.digest(ra) {
        Some(d) => *d,
        None => return Outcome::Error(ErrorKind::Unavailable),
    };
    let db = match reduction.digest(rb) {
        Some(d) => *d,
        None => return Outcome::Error(ErrorKind::Unavailable),
    };
    let equal = da == db;
    let r6 = if equal { 0u64 } else { 1u64 };
    // when both operands are atoms, expose canonical values and inverse hint.
    // otherwise fall back to particle identity (digest equality is the binding constraint).
    let va = reduction.atom_value(ra);
    let vb = reduction.atom_value(rb);
    match (va, vb) {
        (Some(va), Some(vb)) => {
            row.r[4] = va.as_u64();
            row.r[5] = vb.as_u64();
            row.r[6] = r6;
            // inverse hint: (va - vb)^-1 when unequal, 0 when equal
            row.r[7] = if equal { 0 } else { (va - vb).inv().as_u64() };
        }
        _ => {
            // pairs / hash data — use digest[0] as the representative field value.
            // r4 = left digest[0], r5 = right digest[0], r7 = inverse hint.
            let left_val = da[0].as_u64();
            let right_val = db[0].as_u64();
            row.r[4] = left_val;
            row.r[5] = right_val;
            row.r[6] = r6;
            row.r[7] = if equal {
                0
            } else {
                (nebu::Goldilocks::new(left_val) - nebu::Goldilocks::new(right_val)).inv().as_u64()
            };
        }
    }
    let result = if equal { Goldilocks::ZERO } else { Goldilocks::ONE };
    make_field(reduction, result, budget)
}

#[cfg(test)]
mod tests {
    use crate::reduce::{reduce, Outcome};
    use crate::call::NullCalls;
    use crate::trace::NoTrace;
    use crate::data::{Reduction};
    use nebu::Goldilocks;

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    /// formula = [9 [[1 a] [1 b]]]
    fn make_eq<const N: usize>(ar: &mut Reduction<N>, a: u64, b: u64) -> crate::data::Order {
        let t9 = ar.atom(g(9)).unwrap();
        let t1 = ar.atom(g(1)).unwrap();
        let va = ar.atom(g(a)).unwrap();
        let vb = ar.atom(g(b)).unwrap();
        let qa = ar.pair(t1, va).unwrap();
        let qb = ar.pair(t1, vb).unwrap();
        let body = ar.pair(qa, qb).unwrap();
        ar.pair(t9, body).unwrap()
    }

    #[test]
    fn eq_equal_returns_zero() {
        let mut ar = Reduction::<1024>::new();
        let obj = ar.atom(g(0)).unwrap();
        let formula = make_eq(&mut ar, 5, 5);
        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap(), g(0)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn eq_unequal_returns_one() {
        let mut ar = Reduction::<1024>::new();
        let obj = ar.atom(g(0)).unwrap();
        let formula = make_eq(&mut ar, 5, 7);
        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap(), g(1)),
            o => panic!("{:?}", o),
        }
    }

    /// formula = [9 [[1 (1,2)] [1 (1,2)]]] — quote two structurally-equal pairs.
    /// Hash-cons means both sub-quotes resolve to the same Order, but the test
    /// is meaningful: the eq pattern must work on pairs via digest comparison.
    #[test]
    fn eq_pairs_structurally_equal_returns_zero() {
        let mut ar = Reduction::<1024>::new();
        let obj = ar.atom(g(0)).unwrap();
        let one = ar.atom(g(1)).unwrap();
        let two = ar.atom(g(2)).unwrap();
        let pair_a = ar.pair(one, two).unwrap();
        let pair_b = ar.pair(one, two).unwrap();
        assert_eq!(pair_a, pair_b, "hash-cons collapses identical pairs");
        let t1 = ar.atom(g(1)).unwrap();
        let qa = ar.pair(t1, pair_a).unwrap();
        let qb = ar.pair(t1, pair_b).unwrap();
        let body = ar.pair(qa, qb).unwrap();
        let t9 = ar.atom(g(9)).unwrap();
        let formula = ar.pair(t9, body).unwrap();
        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap(), g(0)),
            o => panic!("{:?}", o),
        }
    }

    /// eq(pair(1,2), pair(1,2)) — structurally equal pairs, result is 0, r[7]=0.
    #[test]
    fn eq_pairs_equal_r7_zero() {
        use crate::trace::VecTrace;
        let mut ar = Reduction::<1024>::new();
        let obj = ar.atom(g(0)).unwrap();
        let one = ar.atom(g(1)).unwrap();
        let two = ar.atom(g(2)).unwrap();
        let pair_a = ar.pair(one, two).unwrap();
        let pair_b = ar.pair(one, two).unwrap(); // hash-cons: same id
        let t1 = ar.atom(g(1)).unwrap();
        let qa = ar.pair(t1, pair_a).unwrap();
        let qb = ar.pair(t1, pair_b).unwrap();
        let body = ar.pair(qa, qb).unwrap();
        let t9 = ar.atom(g(9)).unwrap();
        let formula = ar.pair(t9, body).unwrap();
        let mut tracer = VecTrace::default();
        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut tracer) {
            Outcome::Ok(r, _) => {
                assert_eq!(ar.atom_value(r).unwrap(), g(0), "equal pairs return 0");
                let row = tracer.0.last().expect("trace row recorded");
                assert_eq!(row.r[7], 0, "r[7]=0 when operands are equal");
            }
            o => panic!("{:?}", o),
        }
    }

    /// eq(pair(1,2), pair(1,3)) — different pairs, result is 1, r[7] nonzero.
    #[test]
    fn eq_pairs_unequal_r7_nonzero() {
        use crate::trace::VecTrace;
        let mut ar = Reduction::<1024>::new();
        let obj = ar.atom(g(0)).unwrap();
        let one = ar.atom(g(1)).unwrap();
        let two = ar.atom(g(2)).unwrap();
        let three = ar.atom(g(3)).unwrap();
        let pair_a = ar.pair(one, two).unwrap();
        let pair_b = ar.pair(one, three).unwrap();
        let t1 = ar.atom(g(1)).unwrap();
        let qa = ar.pair(t1, pair_a).unwrap();
        let qb = ar.pair(t1, pair_b).unwrap();
        let body = ar.pair(qa, qb).unwrap();
        let t9 = ar.atom(g(9)).unwrap();
        let formula = ar.pair(t9, body).unwrap();
        let mut tracer = VecTrace::default();
        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut tracer) {
            Outcome::Ok(r, _) => {
                assert_eq!(ar.atom_value(r).unwrap(), g(1), "unequal pairs return 1");
                let row = tracer.0.last().expect("trace row recorded");
                assert_ne!(row.r[7], 0, "r[7] nonzero when operands are unequal");
            }
            o => panic!("{:?}", o),
        }
    }

    /// Cells with different contents — verify digest path returns 1.
    #[test]
    fn eq_pairs_different_returns_one() {
        let mut ar = Reduction::<1024>::new();
        let obj = ar.atom(g(0)).unwrap();
        let one = ar.atom(g(1)).unwrap();
        let two = ar.atom(g(2)).unwrap();
        let three = ar.atom(g(3)).unwrap();
        let pair_a = ar.pair(one, two).unwrap();
        let pair_b = ar.pair(one, three).unwrap();
        let t1 = ar.atom(g(1)).unwrap();
        let qa = ar.pair(t1, pair_a).unwrap();
        let qb = ar.pair(t1, pair_b).unwrap();
        let body = ar.pair(qa, qb).unwrap();
        let t9 = ar.atom(g(9)).unwrap();
        let formula = ar.pair(t9, body).unwrap();
        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap(), g(1)),
            o => panic!("{:?}", o),
        }
    }
}
