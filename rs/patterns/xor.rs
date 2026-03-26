//! pattern 11: xor — bitwise XOR (32-bit word)

use crate::noun::{Arena, NounRef};
use crate::reduce::{Outcome, word_binary_op};
use crate::hint::HintProvider;

pub fn xor<const N: usize>(ar: &mut Arena<N>, s: NounRef, b: NounRef, bg: u64, h: &dyn HintProvider<N>) -> Outcome {
    word_binary_op(ar, s, b, bg, h, |a, b| a ^ b)
}
