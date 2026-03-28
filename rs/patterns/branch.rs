//! pattern 4: branch — evaluate test, take yes (0) or no (nonzero)

use crate::noun::{Order, NounId};
use crate::reduce::{reduce, Outcome, ErrorKind, cell_pair, evaluate};
use crate::call::CallProvider;

pub fn branch<const N: usize>(
    order: &mut Order<N>, object: NounId, body: NounId, budget: u64, hints: &dyn CallProvider<N>,
) -> Outcome {
    let (test_formula, rest) = match cell_pair(order, body) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (yes_formula, no_formula) = match cell_pair(order, rest) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (test_result, budget) = match evaluate(order, object, test_formula, budget, hints) { Ok(v) => v, Err(o) => return o };
    let test_value = match order.atom_value(test_result) {
        Some((v, _)) => v.as_u64(),
        None => return Outcome::Error(ErrorKind::TypeError),
    };
    let chosen = if test_value == 0 { yes_formula } else { no_formula };
    reduce(order, object, chosen, budget, hints)
}
