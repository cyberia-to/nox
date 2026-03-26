//! pattern 14: shl — shift left (word type)

use crate::noun::{Order, NounId};
use crate::reduce::{Outcome, ErrorKind, cell_pair, evaluate_word, make_word};
use crate::hint::HintProvider;

pub fn shl<const N: usize>(
    order: &mut Order<N>, object: NounId, body: NounId, budget: u64, hints: &dyn HintProvider<N>,
) -> Outcome {
    let (a, n) = match cell_pair(order, body) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (va, budget) = match evaluate_word(order, object, a, budget, hints) { Ok(v) => v, Err(o) => return o };
    let (vn, budget) = match evaluate_word(order, object, n, budget, hints) { Ok(v) => v, Err(o) => return o };
    let result = if vn >= 32 { 0 } else { (va << vn) & 0xFFFF_FFFF };
    make_word(order, result, budget)
}
