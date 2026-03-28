//! pattern 9: eq — equality by noun identity (hash comparison)
//! works for ALL noun types: atoms, cells, hash nouns
//! returns 0 if equal, 1 if not equal

use nebu::Goldilocks;
use crate::noun::{Order, NounId};
use crate::reduce::{Outcome, ErrorKind, cell_pair, evaluate, make_field};
use crate::call::CallProvider;

pub fn eq<const N: usize>(
    order: &mut Order<N>, object: NounId, body: NounId, budget: u64, hints: &dyn CallProvider<N>,
) -> Outcome {
    let (a, b) = match cell_pair(order, body) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (ra, budget) = match evaluate(order, object, a, budget, hints) { Ok(v) => v, Err(o) => return o };
    let (rb, budget) = match evaluate(order, object, b, budget, hints) { Ok(v) => v, Err(o) => return o };
    let equal = order.digest(ra) == order.digest(rb);
    let result = if equal { Goldilocks::ZERO } else { Goldilocks::ONE };
    make_field(order, result, budget)
}
