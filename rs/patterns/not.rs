//! pattern 13: not — bitwise complement (32-bit word)

use crate::noun::{Arena, NounRef};
use crate::reduce::{Outcome, evaluate_word, make_word};
use crate::hint::HintProvider;

pub fn not<const N: usize>(
    arena: &mut Arena<N>, subject: NounRef, body: NounRef, budget: u64, hints: &dyn HintProvider<N>,
) -> Outcome {
    let (v, budget) = match evaluate_word(arena, subject, body, budget, hints) { Ok(v) => v, Err(o) => return o };
    make_word(arena, (!v) & 0xFFFF_FFFF, budget)
}
