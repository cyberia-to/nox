//! pattern 10: lt — less-than on canonical field representatives
//! returns 0 if a < b, 1 otherwise

use nebu::Goldilocks;
use crate::noun::{Order, NounId};
use crate::reduce::{Outcome, ErrorKind, cell_pair, evaluate_field, make_field};
use crate::hint::HintProvider;

pub fn lt<const N: usize>(
    arena: &mut Order<N>, subject: NounId, body: NounId, budget: u64, hints: &dyn HintProvider<N>,
) -> Outcome {
    let (a, b) = match cell_pair(arena, body) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (va, budget) = match evaluate_field(arena, subject, a, budget, hints) { Ok(v) => v, Err(o) => return o };
    let (vb, budget) = match evaluate_field(arena, subject, b, budget, hints) { Ok(v) => v, Err(o) => return o };
    let result = if va.as_u64() < vb.as_u64() { Goldilocks::ZERO } else { Goldilocks::ONE };
    make_field(arena, result, budget)
}
