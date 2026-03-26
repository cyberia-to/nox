//! pattern 8: inv — field inversion (Fermat, cost 64)
//! inv(0) = Error(InvZero)

use nebu::Goldilocks;
use crate::noun::{Order, NounId};
use crate::reduce::{Outcome, ErrorKind, evaluate_field, make_field};
use crate::hint::HintProvider;

pub fn inv<const N: usize>(
    arena: &mut Order<N>, subject: NounId, body: NounId, budget: u64, hints: &dyn HintProvider<N>,
) -> Outcome {
    let (v, budget) = match evaluate_field(arena, subject, body, budget, hints) { Ok(v) => v, Err(o) => return o };
    if v == Goldilocks::ZERO { return Outcome::Error(ErrorKind::InvZero); }
    make_field(arena, v.inv(), budget)
}
