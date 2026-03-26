//! pattern 15: hash — structural hash via H (dispatched by hash function)
//! returns hash noun: cell(cell(h0,h1), cell(h2,h3))

use crate::noun::{Order, NounId};
use crate::reduce::{Outcome, ErrorKind, evaluate};
use crate::hint::HintProvider;

pub fn hash<const N: usize>(
    order: &mut Order<N>, object: NounId, body: NounId, budget: u64, hints: &dyn HintProvider<N>,
) -> Outcome {
    let (input, budget) = match evaluate(order, object, body, budget, hints) { Ok(v) => v, Err(o) => return o };
    let digest = *order.digest(input);
    match order.hash_noun(&digest) {
        Some(r) => Outcome::Ok(r, budget),
        None => Outcome::Error(ErrorKind::Unavailable),
    }
}
