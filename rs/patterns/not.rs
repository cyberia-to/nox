//! pattern 13: not — bitwise complement (word type)

use crate::noun::{Order, NounId};
use crate::reduce::{Outcome, evaluate_word, make_word};
use crate::call::CallProvider;

pub fn not<const N: usize>(
    order: &mut Order<N>, object: NounId, body: NounId, budget: u64, hints: &dyn CallProvider<N>,
) -> Outcome {
    let (v, budget) = match evaluate_word(order, object, body, budget, hints) { Ok(v) => v, Err(o) => return o };
    make_word(order, (!v) & 0xFFFF_FFFF, budget)
}
