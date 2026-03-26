//! pattern 9: eq — equality by noun identity (hash comparison)
//! works for ALL noun types: atoms, cells, hash nouns
//! returns 0 if equal, 1 if not equal

use nebu::Goldilocks;
use crate::noun::{Order, NounId};
use crate::reduce::{Outcome, ErrorKind, cell_pair, evaluate, make_field};
use crate::hint::HintProvider;

pub fn eq<const N: usize>(
    arena: &mut Order<N>, subject: NounId, body: NounId, budget: u64, hints: &dyn HintProvider<N>,
) -> Outcome {
    let (a, b) = match cell_pair(arena, body) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (ra, budget) = match evaluate(arena, subject, a, budget, hints) { Ok(v) => v, Err(o) => return o };
    let (rb, budget) = match evaluate(arena, subject, b, budget, hints) { Ok(v) => v, Err(o) => return o };
    let equal = arena.digest(ra) == arena.digest(rb);
    let result = if equal { Goldilocks::ZERO } else { Goldilocks::ONE };
    make_field(arena, result, budget)
}
