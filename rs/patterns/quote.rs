//! pattern 1: quote — return body as literal

use crate::noun::NounId;
use crate::reduce::Outcome;

pub fn quote(body: NounId, budget: u64) -> Outcome {
    Outcome::Ok(body, budget)
}
