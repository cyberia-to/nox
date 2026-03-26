//! pattern 13: not — bitwise complement (word type)

use crate::noun::{Order, NounId};
use crate::reduce::{Outcome, evaluate_word, make_word};
use crate::hint::HintProvider;

pub fn not<const N: usize>(
    arena: &mut Order<N>, subject: NounId, body: NounId, budget: u64, hints: &dyn HintProvider<N>,
) -> Outcome {
    let (v, budget) = match evaluate_word(arena, subject, body, budget, hints) { Ok(v) => v, Err(o) => return o };
    make_word(arena, (!v) & 0xFFFF_FFFF, budget)
}
