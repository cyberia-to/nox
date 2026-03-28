//! pattern 11: xor — bitwise XOR (word type)

use crate::noun::{Order, NounId};
use crate::reduce::{Outcome, word_binary_op};
use crate::call::CallProvider;

pub fn xor<const N: usize>(order: &mut Order<N>, object: NounId, b: NounId, bg: u64, h: &dyn CallProvider<N>) -> Outcome {
    word_binary_op(order, object, b, bg, h, |a, b| a ^ b)
}
