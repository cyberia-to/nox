//! pattern 2: compose — evaluate two sub-formulas, apply second to first
//! reduce(s, [2 [x y]], b) = reduce(reduce(s,x), reduce(s,y), b')

use crate::noun::{Order, NounId};
use crate::reduce::{reduce_inner, Outcome, ErrorKind, cell_pair, evaluate};
use crate::call::CallProvider;
use crate::trace::{Tracer, TraceRow};

pub fn compose<const N: usize, T: Tracer>(
    order: &mut Order<N>, object: NounId, body: NounId, budget: u64,
    hints: &dyn CallProvider<N>, tracer: &mut T, depth: u64,
    row: &mut TraceRow,
) -> Outcome {
    let (a, b) = match cell_pair(order, body) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    let (obj, budget) = match evaluate(order, object, a, budget, hints, tracer, depth) {
        Ok(v) => v, Err(o) => return o,
    };
    let (frm, budget) = match evaluate(order, object, b, budget, hints, tracer, depth) {
        Ok(v) => v, Err(o) => return o,
    };
    row.r[4] = obj as u64;
    row.r[5] = frm as u64;
    row.r[6] = a as u64;
    row.r[7] = b as u64;
    reduce_inner(order, obj, frm, budget, hints, tracer, depth + 1)
}

#[cfg(test)]
mod tests {
    use crate::reduce::{reduce, Outcome};
    use crate::call::NullCalls;
    use crate::trace::NoTrace;
    use crate::noun::{Order, Tag};
    use nebu::Goldilocks;

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    /// compose: [quote 42] then [axis 1 = identity] → 42
    /// formula = [2 [1 42] [0 1]]
    /// reduces(s, [1 42]) → 42, reduces(s, [0 1]) → s=obj, reduce(42, obj=s) ...
    /// Actually: compose evaluates both sub-formulas against object, then
    /// applies frm to obj.  With [1 42] we get 42, with [0 1] we get identity(s).
    /// Then we reduce(42, s_formula, budget) which doesn't make sense for identity.
    ///
    /// Correct test: object=5, formula=[2 [0 1] [1 [0 1]]]
    /// step1: evaluate(5, [0 1]) → 5 (axis identity)
    /// step2: evaluate(5, [1 [0 1]]) → [0 1] (quote of axis formula)
    /// step3: reduce(5, [0 1], budget) → 5 (axis identity)
    #[test]
    fn compose_chains_reductions() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(5), Tag::Field).unwrap();

        // [0 1] — axis identity formula
        let t0 = ar.atom(g(0), Tag::Field).unwrap();
        let t1 = ar.atom(g(1), Tag::Field).unwrap();
        let axis_id = ar.cell(t0, t1).unwrap();

        // [1 [0 1]] — quote of axis_id
        let t1b = ar.atom(g(1), Tag::Field).unwrap();
        let quote_axis = ar.cell(t1b, axis_id).unwrap();

        // body = [[0 1] [1 [0 1]]]
        let body = ar.cell(axis_id, quote_axis).unwrap();

        // formula = [2 body]
        let t2 = ar.atom(g(2), Tag::Field).unwrap();
        let formula = ar.cell(t2, body).unwrap();

        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(r, obj),
            o => panic!("{:?}", o),
        }
    }
}
