//! pattern 2: compose — evaluate two sub-formulas, apply second to first
//! reduce(s, [2 [x y]], b) = reduce(reduce(s,x), reduce(s,y), b')

use crate::noun::{Order, NounId};
use crate::reduce::{reduce, Outcome, ErrorKind, cell_pair, evaluate};
use crate::hint::HintProvider;

pub fn compose<const N: usize>(
    arena: &mut Order<N>, subject: NounId, body: NounId, budget: u64, hints: &dyn HintProvider<N>,
) -> Outcome {
    let (a, b) = match cell_pair(arena, body) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (obj, budget) = match evaluate(arena, subject, a, budget, hints) { Ok(v) => v, Err(o) => return o };
    let (frm, budget) = match evaluate(arena, subject, b, budget, hints) { Ok(v) => v, Err(o) => return o };
    reduce(arena, obj, frm, budget, hints)
}
