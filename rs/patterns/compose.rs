//! pattern 2: compose — evaluate two sub-formulas, apply second to first
//! reduce(s, [2 [x y]], b) = reduce(reduce(s,x), reduce(s,y), b')

use crate::noun::{Order, NounId};
use crate::reduce::{reduce, Outcome, ErrorKind, cell_pair, evaluate};
use crate::call::CallProvider;

pub fn compose<const N: usize>(
    order: &mut Order<N>, object: NounId, body: NounId, budget: u64, hints: &dyn CallProvider<N>,
) -> Outcome {
    let (a, b) = match cell_pair(order, body) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (obj, budget) = match evaluate(order, object, a, budget, hints) { Ok(v) => v, Err(o) => return o };
    let (frm, budget) = match evaluate(order, object, b, budget, hints) { Ok(v) => v, Err(o) => return o };
    reduce(order, obj, frm, budget, hints)
}
