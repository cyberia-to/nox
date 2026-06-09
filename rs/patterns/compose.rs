//! pattern 2: compose — evaluate two sub-formulas, apply second to first
//! reduce(s, [2 [x y]], b) = reduce(reduce(s,x), reduce(s,y), b')

use crate::data::{Order, OrderId};
use crate::reduce::{reduce_inner, Outcome, ErrorKind, pair_children, evaluate_binary};
use crate::call::CallProvider;
use crate::trace::{Tracer, TraceRow};
use crate::jets::registry::JetRegistry;

pub fn compose<const N: usize, T: Tracer>(
    order: &mut Order<N>, object: OrderId, body: OrderId, budget: u64,
    hints: &dyn CallProvider<N>, tracer: &mut T, depth: u64,
    row: &mut TraceRow, registry: &JetRegistry<N>,
) -> Outcome {
    let (a, b) = match pair_children(order, body) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    // Initial phase: evaluate the two sub-formulas under the partition rule.
    // Both arms are statically bounded (their bounds may themselves contain
    // DYNAMIC markers, in which case evaluate_binary falls back to sequential).
    let (obj, frm, budget) = match evaluate_binary(order, object, a, b, budget, hints, tracer, depth, registry) {
        Ok(v) => v, Err(o) => return o,
    };
    row.r[4] = obj as u64;
    row.r[5] = frm as u64;
    row.r[6] = a as u64;
    row.r[7] = b as u64;
    // Continuation phase: reduce(obj, frm) is the dynamic-cost frontier.
    // Sequential semantics — frm is only known now (runtime value).
    reduce_inner(order, obj, frm, budget, hints, tracer, depth + 1, registry)
}

#[cfg(test)]
mod tests {
    use crate::reduce::{reduce, Outcome};
    use crate::call::NullCalls;
    use crate::trace::NoTrace;
    use crate::data::{Order};
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
        let obj = ar.atom(g(5)).unwrap();

        // [0 1] — axis identity formula
        let t0 = ar.atom(g(0)).unwrap();
        let t1 = ar.atom(g(1)).unwrap();
        let axis_id = ar.pair(t0, t1).unwrap();

        // [1 [0 1]] — quote of axis_id
        let t1b = ar.atom(g(1)).unwrap();
        let quote_axis = ar.pair(t1b, axis_id).unwrap();

        // body = [[0 1] [1 [0 1]]]
        let body = ar.pair(axis_id, quote_axis).unwrap();

        // formula = [2 body]
        let t2 = ar.atom(g(2)).unwrap();
        let formula = ar.pair(t2, body).unwrap();

        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(r, obj),
            o => panic!("{:?}", o),
        }
    }
}
