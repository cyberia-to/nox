//! pattern 3: cons — evaluate two sub-formulas, construct cell from results

use crate::noun::{Order, NounId};
use crate::reduce::{Outcome, ErrorKind, cell_pair, evaluate};
use crate::hint::HintProvider;

pub fn cons<const N: usize>(
    order: &mut Order<N>, object: NounId, body: NounId, budget: u64, hints: &dyn HintProvider<N>,
) -> Outcome {
    let (a, b) = match cell_pair(order, body) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (left, budget) = match evaluate(order, object, a, budget, hints) { Ok(v) => v, Err(o) => return o };
    let (right, budget) = match evaluate(order, object, b, budget, hints) { Ok(v) => v, Err(o) => return o };
    match order.cell(left, right) {
        Some(c) => Outcome::Ok(c, budget),
        None => Outcome::Error(ErrorKind::Unavailable),
    }
}
