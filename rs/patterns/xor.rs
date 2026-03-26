//! pattern 11: xor — bitwise XOR (word type)

use crate::noun::{Order, NounId};
use crate::reduce::{Outcome, word_binary_op};
use crate::hint::HintProvider;

pub fn xor<const N: usize>(ar: &mut Order<N>, s: NounId, b: NounId, bg: u64, h: &dyn HintProvider<N>) -> Outcome {
    word_binary_op(ar, s, b, bg, h, |a, b| a ^ b)
}
