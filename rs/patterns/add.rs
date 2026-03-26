//! pattern 5: add — field addition

use crate::noun::{Order, NounId};
use crate::reduce::{Outcome, field_binary_op};
use crate::hint::HintProvider;

pub fn add<const N: usize>(order: &mut Order<N>, object: NounId, b: NounId, bg: u64, h: &dyn HintProvider<N>) -> Outcome {
    field_binary_op(order, object, b, bg, h, |a, b| a + b)
}
