//! pattern 13: not — bitwise complement (word type)

use crate::noun::{Order, NounId};
use crate::reduce::{Outcome, evaluate_word, make_word};
use crate::call::CallProvider;
use crate::trace::Tracer;

pub fn not<const N: usize, T: Tracer>(
    order: &mut Order<N>, object: NounId, body: NounId, budget: u64,
    hints: &dyn CallProvider<N>, tracer: &mut T,
) -> Outcome {
    let (v, budget) = match evaluate_word(order, object, body, budget, hints, tracer) {
        Ok(v) => v, Err(o) => return o,
    };
    make_word(order, (!v) & 0xFFFF_FFFF, budget)
}
