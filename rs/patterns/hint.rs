//! pattern 16: hint — non-deterministic witness injection
//! step 1: evaluate tag formula
//! step 2: ask prover for witness (None → Halt)
//! step 3: validate check(witness, subject) = 0
//! step 4: return witness

use nebu::Goldilocks;
use crate::noun::{Arena, NounRef};
use crate::reduce::{reduce, Outcome, ErrorKind, cell_pair, evaluate};
use crate::hint::HintProvider;

pub fn hint<const N: usize>(
    arena: &mut Arena<N>, subject: NounRef, body: NounRef, budget: u64, hints: &dyn HintProvider<N>,
) -> Outcome {
    let (tag_formula, check_formula) = match cell_pair(arena, body) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    let (tag_result, budget) = match evaluate(arena, subject, tag_formula, budget, hints) {
        Ok(v) => v, Err(o) => return o,
    };
    let tag_value = match arena.atom_value(tag_result) {
        Some((v, _)) => v,
        None => return Outcome::Error(ErrorKind::TypeError),
    };
    let witness = match hints.provide(arena, tag_value, subject) {
        Some(w) => w,
        None => return Outcome::Halt(budget),
    };
    let witness_subject = match arena.cell(witness, subject) {
        Some(c) => c,
        None => return Outcome::Error(ErrorKind::Unavailable),
    };
    match reduce(arena, witness_subject, check_formula, budget, hints) {
        Outcome::Ok(check_result, budget) => {
            match arena.atom_value(check_result) {
                Some((v, _)) if v == Goldilocks::ZERO => Outcome::Ok(witness, budget),
                _ => Outcome::Error(ErrorKind::HintRejected),
            }
        }
        Outcome::Error(_) => Outcome::Error(ErrorKind::HintRejected),
        Outcome::Halt(b) => Outcome::Halt(b),
    }
}
