//! pattern 5: add — field addition

use crate::noun::{Order, NounId};
use crate::reduce::{Outcome, field_binary_op};
use crate::call::CallProvider;
use crate::trace::{Tracer, TraceRow};

pub fn add<const N: usize, T: Tracer>(
    order: &mut Order<N>, object: NounId, b: NounId, bg: u64,
    h: &dyn CallProvider<N>, tracer: &mut T, depth: u64,
    row: &mut TraceRow,
) -> Outcome {
    field_binary_op(order, object, b, bg, h, tracer, depth, row, |a, b| a + b)
}

#[cfg(test)]
mod tests {
    use crate::reduce::{reduce, Outcome};
    use crate::call::NullCalls;
    use crate::trace::NoTrace;
    use crate::noun::{Order, Tag};
    use nebu::Goldilocks;

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    /// formula = [5 [[1 a] [1 b]]]
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
    fn add_field_elements() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        let formula = make_field_binop(&mut ar, 5, 3, 5);
        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(8)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn add_zero_identity() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        let formula = make_field_binop(&mut ar, 5, 7, 0);
        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(7)),
            o => panic!("{:?}", o),
        }
    }
}
