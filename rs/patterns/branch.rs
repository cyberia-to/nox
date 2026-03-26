//! pattern 4: branch — evaluate test, take yes (0) or no (nonzero)

use crate::noun::{Arena, NounRef};
use crate::reduce::{reduce, Outcome, ErrorKind, cell_pair, evaluate};
use crate::hint::HintProvider;

pub fn branch<const N: usize>(
    arena: &mut Arena<N>, subject: NounRef, body: NounRef, budget: u64, hints: &dyn HintProvider<N>,
) -> Outcome {
    let (test_formula, rest) = match cell_pair(arena, body) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (yes_formula, no_formula) = match cell_pair(arena, rest) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (test_result, budget) = match evaluate(arena, subject, test_formula, budget, hints) { Ok(v) => v, Err(o) => return o };
    let test_value = match arena.atom_value(test_result) {
        Some((v, _)) => v.as_u64(),
        None => return Outcome::Error(ErrorKind::TypeError),
    };
    let chosen = if test_value == 0 { yes_formula } else { no_formula };
    reduce(arena, subject, chosen, budget, hints)
}
