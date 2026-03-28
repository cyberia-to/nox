//! pattern 8: inv — field inversion (Fermat, cost 64)
//! inv(0) = Error(InvZero)

use nebu::Goldilocks;
use crate::noun::{Order, NounId};
use crate::reduce::{Outcome, ErrorKind, evaluate_field, make_field};
use crate::call::CallProvider;

pub fn inv<const N: usize>(
    order: &mut Order<N>, object: NounId, body: NounId, budget: u64, hints: &dyn CallProvider<N>,
) -> Outcome {
    let (v, budget) = match evaluate_field(order, object, body, budget, hints) { Ok(v) => v, Err(o) => return o };
    if v == Goldilocks::ZERO { return Outcome::Error(ErrorKind::InvZero); }
    make_field(order, v.inv(), budget)
}
