//! pattern 15: hash — hemera structural hash (cost 300)
//! returns hash noun: cell(cell(h0,h1), cell(h2,h3))

use crate::noun::{Arena, NounRef};
use crate::reduce::{Outcome, ErrorKind, evaluate};
use crate::hint::HintProvider;

pub fn hash<const N: usize>(
    arena: &mut Arena<N>, subject: NounRef, body: NounRef, budget: u64, hints: &dyn HintProvider<N>,
) -> Outcome {
    let (input, budget) = match evaluate(arena, subject, body, budget, hints) { Ok(v) => v, Err(o) => return o };
    let digest = *arena.digest(input);
    match arena.hash_noun(&digest) {
        Some(r) => Outcome::Ok(r, budget),
        None => Outcome::Error(ErrorKind::Unavailable),
    }
}
