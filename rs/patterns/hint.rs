//! pattern 16: hint — non-deterministic witness injection
//! step 1: evaluate tag formula
//! step 2: ask prover for witness (None → Halt)
//! step 3: validate check(witness, object) = 0
//! step 4: return witness

use nebu::Goldilocks;
use crate::noun::{Order, NounId};
use crate::reduce::{reduce, Outcome, ErrorKind, cell_pair, evaluate};
use crate::hint::HintProvider;

pub fn hint<const N: usize>(
    order: &mut Order<N>, object: NounId, body: NounId, budget: u64, hints: &dyn HintProvider<N>,
) -> Outcome {
    let (tag_formula, check_formula) = match cell_pair(order, body) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    let (tag_result, budget) = match evaluate(order, object, tag_formula, budget, hints) {
        Ok(v) => v, Err(o) => return o,
    };
    let tag_value = match order.atom_value(tag_result) {
        Some((v, _)) => v,
        None => return Outcome::Error(ErrorKind::TypeError),
    };
    let witness = match hints.provide(order, tag_value, object) {
        Some(w) => w,
        None => return Outcome::Halt(budget),
    };
    let witness_object = match order.cell(witness, object) {
        Some(c) => c,
        None => return Outcome::Error(ErrorKind::Unavailable),
    };
    match reduce(order, witness_object, check_formula, budget, hints) {
        Outcome::Ok(check_result, budget) => {
            match order.atom_value(check_result) {
                Some((v, _)) if v == Goldilocks::ZERO => Outcome::Ok(witness, budget),
                _ => Outcome::Error(ErrorKind::HintRejected),
            }
        }
        Outcome::Error(_) => Outcome::Error(ErrorKind::HintRejected),
        Outcome::Halt(b) => Outcome::Halt(b),
    }
}
