//! pattern 8: inv — field inversion (Fermat, cost 64)
//! inv(0) = Error(InvZero)

use nebu::Goldilocks;
use crate::noun::{Order, NounId};
use crate::reduce::{Outcome, ErrorKind, evaluate_field, make_field};
use crate::call::CallProvider;
use crate::trace::Tracer;

pub fn inv<const N: usize, T: Tracer>(
    order: &mut Order<N>, object: NounId, body: NounId, budget: u64,
    hints: &dyn CallProvider<N>, tracer: &mut T,
) -> Outcome {
    let (v, budget) = match evaluate_field(order, object, body, budget, hints, tracer) {
        Ok(v) => v, Err(o) => return o,
    };
    if v == Goldilocks::ZERO { return Outcome::Error(ErrorKind::InvZero); }
    make_field(order, v.inv(), budget)
}
