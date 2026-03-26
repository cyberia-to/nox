//! pattern 1: quote — return body as literal

use crate::noun::NounRef;
use crate::reduce::Outcome;

pub fn quote(body: NounRef, budget: u64) -> Outcome {
    Outcome::Ok(body, budget)
}
