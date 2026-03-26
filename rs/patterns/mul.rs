//! pattern 7: mul — field multiplication

use crate::noun::{Arena, NounRef};
use crate::reduce::{Outcome, field_binary_op};
use crate::hint::HintProvider;

pub fn mul<const N: usize>(ar: &mut Arena<N>, s: NounRef, b: NounRef, bg: u64, h: &dyn HintProvider<N>) -> Outcome {
    field_binary_op(ar, s, b, bg, h, |a, b| a * b)
}
