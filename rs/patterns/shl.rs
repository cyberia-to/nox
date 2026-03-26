//! pattern 14: shl — shift left (32-bit word)

use crate::noun::{Arena, NounRef};
use crate::reduce::{Outcome, ErrorKind, cell_pair, evaluate_word, make_word};
use crate::hint::HintProvider;

pub fn shl<const N: usize>(
    arena: &mut Arena<N>, subject: NounRef, body: NounRef, budget: u64, hints: &dyn HintProvider<N>,
) -> Outcome {
    let (a, n) = match cell_pair(arena, body) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (va, budget) = match evaluate_word(arena, subject, a, budget, hints) { Ok(v) => v, Err(o) => return o };
    let (vn, budget) = match evaluate_word(arena, subject, n, budget, hints) { Ok(v) => v, Err(o) => return o };
    let result = if vn >= 32 { 0 } else { (va << vn) & 0xFFFF_FFFF };
    make_word(arena, result, budget)
}
