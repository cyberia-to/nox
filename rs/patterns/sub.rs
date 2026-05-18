//! pattern 6: sub — field subtraction

use crate::noun::{Order, NounId};
use crate::reduce::{Outcome, field_binary_op};
use crate::call::CallProvider;
use crate::trace::{Tracer, TraceRow};
use crate::jets::registry::JetRegistry;

pub fn sub<const N: usize, T: Tracer>(
    order: &mut Order<N>, object: NounId, b: NounId, bg: u64,
    h: &dyn CallProvider<N>, tracer: &mut T, depth: u64,
    row: &mut TraceRow, registry: &JetRegistry<N>,
) -> Outcome {
    field_binary_op(order, object, b, bg, h, tracer, depth, row, registry, |a, b| a - b)
}

#[cfg(test)]
mod tests {
    use crate::reduce::{reduce, Outcome};
    use crate::call::NullCalls;
    use crate::trace::NoTrace;
    use crate::noun::{Order, Tag};
    use nebu::Goldilocks;

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    fn make_field_binop<const N: usize>(
        ar: &mut Order<N>, tag: u64, a: u64, b: u64,
    ) -> crate::noun::NounId {
        let t = ar.atom(g(tag), Tag::Field).unwrap();
        let t1 = ar.atom(g(1), Tag::Field).unwrap();
        let va = ar.atom(g(a), Tag::Field).unwrap();
        let vb = ar.atom(g(b), Tag::Field).unwrap();
        let qa = ar.cell(t1, va).unwrap();
        let qb = ar.cell(t1, vb).unwrap();
        let body = ar.cell(qa, qb).unwrap();
        ar.cell(t, body).unwrap()
    }

    #[test]
    fn sub_field_elements() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        let formula = make_field_binop(&mut ar, 6, 8, 5);
        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(3)),
            o => panic!("{:?}", o),
        }
    }

    /// Field boundary: sub(0, 1) = p-1; sub(1, 2) = p-1 (unsigned wrap in field).
    #[test]
    fn sub_field_boundary() {
        const P: u64 = 0xFFFF_FFFF_0000_0001;
        // sub(0, 1) == p-1
        {
            let mut ar = Order::<1024>::new();
            let obj = ar.atom(g(0), Tag::Field).unwrap();
            let formula = make_field_binop(&mut ar, 6, 0, 1);
            match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
                Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(P - 1)),
                o => panic!("sub(0, 1): {:?}", o),
            }
        }
        // sub(1, 2) == p-1  (1 - 2 = -1 = p-1 in the field)
        {
            let mut ar = Order::<1024>::new();
            let obj = ar.atom(g(0), Tag::Field).unwrap();
            let formula = make_field_binop(&mut ar, 6, 1, 2);
            match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
                Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(P - 1)),
                o => panic!("sub(1, 2): {:?}", o),
            }
        }
    }

    /// 0 - 1 = p - 1 (mod p) — boundary case for modular subtraction.
    #[test]
    fn sub_underflow_wraps() {
        const P: u64 = 0xFFFF_FFFF_0000_0001;
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        let formula = make_field_binop(&mut ar, 6, 0, 1);
        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(P - 1)),
            o => panic!("{:?}", o),
        }
    }
}
