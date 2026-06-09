//! pattern 7: mul — field multiplication

use crate::data::{Reduction, Order};
use crate::reduce::{Outcome, field_binary_op};
use crate::call::CallProvider;
use crate::trace::{Tracer, TraceRow};
use crate::jets::registry::JetRegistry;

pub fn mul<const N: usize, T: Tracer>(
    reduction: &mut Reduction<N>, object: Order, b: Order, bg: u64,
    h: &dyn CallProvider<N>, tracer: &mut T, depth: u64,
    row: &mut TraceRow, registry: &JetRegistry<N>,
) -> Outcome {
    field_binary_op(reduction, object, b, bg, h, tracer, depth, row, registry, |a, b| a * b)
}

#[cfg(test)]
mod tests {
    use crate::reduce::{reduce, Outcome};
    use crate::call::NullCalls;
    use crate::trace::NoTrace;
    use crate::data::{Reduction};
    use nebu::Goldilocks;

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    fn make_field_binop<const N: usize>(
        ar: &mut Reduction<N>, tag: u64, a: u64, b: u64,
    ) -> crate::data::Order {
        let t = ar.atom(g(tag)).unwrap();
        let t1 = ar.atom(g(1)).unwrap();
        let va = ar.atom(g(a)).unwrap();
        let vb = ar.atom(g(b)).unwrap();
        let qa = ar.pair(t1, va).unwrap();
        let qb = ar.pair(t1, vb).unwrap();
        let body = ar.pair(qa, qb).unwrap();
        ar.pair(t, body).unwrap()
    }

    #[test]
    fn mul_field_elements() {
        let mut ar = Reduction::<1024>::new();
        let obj = ar.atom(g(0)).unwrap();
        let formula = make_field_binop(&mut ar, 7, 3, 5);
        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap(), g(15)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn mul_zero() {
        let mut ar = Reduction::<1024>::new();
        let obj = ar.atom(g(0)).unwrap();
        let formula = make_field_binop(&mut ar, 7, 7, 0);
        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap(), g(0)),
            o => panic!("{:?}", o),
        }
    }

    /// Field boundary: mul(p-1, p-1) = 1, since (-1)*(-1) = 1 in the field.
    #[test]
    fn mul_field_boundary() {
        const P: u64 = 0xFFFF_FFFF_0000_0001;
        let mut ar = Reduction::<1024>::new();
        let obj = ar.atom(g(0)).unwrap();
        let formula = make_field_binop(&mut ar, 7, P - 1, P - 1);
        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap(), g(1)),
            o => panic!("mul(p-1, p-1): {:?}", o),
        }
    }

    /// (p-1) * (p-1) = 1 (mod p) — boundary case for modular multiplication.
    #[test]
    fn mul_neg_one_squared_is_one() {
        const P: u64 = 0xFFFF_FFFF_0000_0001;
        let mut ar = Reduction::<1024>::new();
        let obj = ar.atom(g(0)).unwrap();
        let formula = make_field_binop(&mut ar, 7, P - 1, P - 1);
        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap(), g(1)),
            o => panic!("{:?}", o),
        }
    }
}
