//! pattern 10: lt — less-than on canonical field representatives
//! returns 0 if a < b, 1 otherwise

use nebu::Goldilocks;
use crate::noun::{Order, NounId};
use crate::reduce::{Outcome, ErrorKind, cell_pair, evaluate_field, make_field};
use crate::call::CallProvider;

pub fn lt<const N: usize>(
    order: &mut Order<N>, object: NounId, body: NounId, budget: u64, hints: &dyn CallProvider<N>,
) -> Outcome {
    let (a, b) = match cell_pair(order, body) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (va, budget) = match evaluate_field(order, object, a, budget, hints) { Ok(v) => v, Err(o) => return o };
    let (vb, budget) = match evaluate_field(order, object, b, budget, hints) { Ok(v) => v, Err(o) => return o };
    let result = if va.as_u64() < vb.as_u64() { Goldilocks::ZERO } else { Goldilocks::ONE };
    make_field(order, result, budget)
}
