//! pattern 3: cons — evaluate two sub-formulas, construct cell from results

use crate::noun::{Arena, NounRef};
use crate::reduce::{Outcome, ErrorKind, cell_pair, evaluate};
use crate::hint::HintProvider;

pub fn cons<const N: usize>(
    arena: &mut Arena<N>, subject: NounRef, body: NounRef, budget: u64, hints: &dyn HintProvider<N>,
) -> Outcome {
    let (a, b) = match cell_pair(arena, body) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (left, budget) = match evaluate(arena, subject, a, budget, hints) { Ok(v) => v, Err(o) => return o };
    let (right, budget) = match evaluate(arena, subject, b, budget, hints) { Ok(v) => v, Err(o) => return o };
    match arena.cell(left, right) {
        Some(c) => Outcome::Ok(c, budget),
        None => Outcome::Error(ErrorKind::Unavailable),
    }
}
