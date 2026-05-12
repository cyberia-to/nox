//! pattern 12: and — bitwise AND (word type)

use crate::noun::{Order, NounId};
use crate::reduce::{Outcome, word_binary_op};
use crate::call::CallProvider;
use crate::trace::Tracer;

pub fn and<const N: usize, T: Tracer>(
    order: &mut Order<N>, object: NounId, b: NounId, bg: u64,
    h: &dyn CallProvider<N>, tracer: &mut T,
) -> Outcome {
    word_binary_op(order, object, b, bg, h, tracer, |a, b| a & b)
}
