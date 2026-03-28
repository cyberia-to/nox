//! pattern 17: look — deterministic external lookup
//! step 1: evaluate tag formula to get lookup key
//! step 2: evaluate body formula to get lookup argument
//! step 3: return looked-up value
//!
//! unlike call (pattern 16), look is deterministic — the same key
//! always produces the same result. no prover witness needed.

use crate::noun::{Order, NounId};
use crate::reduce::{Outcome, ErrorKind, cell_pair};
use crate::call::CallProvider;

pub fn look<const N: usize>(
    order: &mut Order<N>, _object: NounId, body: NounId, _budget: u64, _asks: &dyn CallProvider<N>,
) -> Outcome {
    let (_key_formula, _arg_formula) = match cell_pair(order, body) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    // TODO: implement deterministic lookup dispatch
    Outcome::Error(ErrorKind::Malformed)
}
