//! pattern 3: cons — evaluate two sub-formulas, construct cell from results

use crate::noun::{Order, NounId};
use crate::reduce::{Outcome, ErrorKind, cell_pair, evaluate_binary};
use crate::call::CallProvider;
use crate::trace::{Tracer, TraceRow};

pub fn cons<const N: usize, T: Tracer>(
    order: &mut Order<N>, object: NounId, body: NounId, budget: u64,
    hints: &dyn CallProvider<N>, tracer: &mut T, depth: u64,
    row: &mut TraceRow,
) -> Outcome {
    let (a, b) = match cell_pair(order, body) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    let (left, right, budget) = match evaluate_binary(order, object, a, b, budget, hints, tracer, depth) {
        Ok(v) => v, Err(o) => return o,
    };
    row.r[4] = left as u64;
    row.r[5] = right as u64;
    match order.cell(left, right) {
        Some(c) => Outcome::Ok(c, budget),
        None => Outcome::Error(ErrorKind::Unavailable),
    }
}

#[cfg(test)]
mod tests {
    use crate::reduce::{reduce, Outcome};
    use crate::call::NullCalls;
    use crate::trace::NoTrace;
    use crate::noun::{Order, Tag};
    use nebu::Goldilocks;

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    /// cons([1 10], [1 20]) → cell(10, 20)
    /// formula = [3 [[1 10] [1 20]]]
    #[test]
    fn cons_builds_cell() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();

        let t1 = ar.atom(g(1), Tag::Field).unwrap();
        let v10 = ar.atom(g(10), Tag::Field).unwrap();
        let v20 = ar.atom(g(20), Tag::Field).unwrap();

        let q10 = ar.cell(t1, v10).unwrap();
        let q20 = ar.cell(t1, v20).unwrap();
        let body = ar.cell(q10, q20).unwrap();

        let t3 = ar.atom(g(3), Tag::Field).unwrap();
        let formula = ar.cell(t3, body).unwrap();

        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => {
                assert!(ar.is_cell(r));
                let head = ar.head(r).unwrap();
                let tail = ar.tail(r).unwrap();
                assert_eq!(ar.atom_value(head).unwrap().0, g(10));
                assert_eq!(ar.atom_value(tail).unwrap().0, g(20));
            }
            o => panic!("{:?}", o),
        }
    }
}
