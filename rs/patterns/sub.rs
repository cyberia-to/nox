//! pattern 6: sub — field subtraction

use crate::noun::{Order, NounId};
use crate::reduce::{Outcome, field_binary_op};
use crate::call::CallProvider;

pub fn sub<const N: usize>(order: &mut Order<N>, object: NounId, b: NounId, bg: u64, h: &dyn CallProvider<N>) -> Outcome {
    field_binary_op(order, object, b, bg, h, |a, b| a - b)
}
