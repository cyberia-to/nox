//! pattern 12: and — bitwise AND (word type)

use crate::noun::{Order, NounId};
use crate::reduce::{Outcome, word_binary_op};
use crate::call::CallProvider;

pub fn and<const N: usize>(order: &mut Order<N>, object: NounId, b: NounId, bg: u64, h: &dyn CallProvider<N>) -> Outcome {
    word_binary_op(order, object, b, bg, h, |a, b| a & b)
}
