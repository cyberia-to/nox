//! pattern 16: call — non-deterministic witness injection
//! step 1: evaluate tag formula
//! step 2: call prover for witness (None -> Halt)
//! step 3: validate check(witness, object) = 0
//! step 4: return witness

use nebu::Goldilocks;
use crate::noun::{Order, NounId};
use crate::reduce::{reduce, Outcome, ErrorKind, cell_pair, evaluate};
use crate::call::CallProvider;

pub fn call_witness<const N: usize>(
    order: &mut Order<N>, object: NounId, body: NounId, budget: u64, calls: &dyn CallProvider<N>,
) -> Outcome {
    let (tag_formula, check_formula) = match cell_pair(order, body) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    let (tag_result, budget) = match evaluate(order, object, tag_formula, budget, calls) {
        Ok(v) => v, Err(o) => return o,
    };
    let tag_value = match order.atom_value(tag_result) {
        Some((v, _)) => v,
        None => return Outcome::Error(ErrorKind::TypeError),
    };
    let witness = match calls.provide(order, tag_value, object) {
        Some(w) => w,
        None => return Outcome::Halt(budget),
    };
    let witness_object = match order.cell(witness, object) {
        Some(c) => c,
        None => return Outcome::Error(ErrorKind::Unavailable),
    };
    match reduce(order, witness_object, check_formula, budget, calls) {
        Outcome::Ok(check_result, budget) => {
            match order.atom_value(check_result) {
                Some((v, _)) if v == Goldilocks::ZERO => Outcome::Ok(witness, budget),
                _ => Outcome::Error(ErrorKind::CallRejected),
            }
        }
        Outcome::Error(_) => Outcome::Error(ErrorKind::CallRejected),
        Outcome::Halt(b) => Outcome::Halt(b),
    }
}
