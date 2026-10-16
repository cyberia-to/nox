// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! jet: decider — legacy, retired in phase 3
//!
//! Legacy decider jet of the retired zheng design. It charges a fixed cost
//! and verifies nothing itself: the accept/reject bit is whatever the call
//! provider (hints) returns, and no sumcheck, CCS evaluation or commitment
//! opening is checked here (launch #40, nox PR #20 audit). Never treat its
//! result as evidence.
//!
//! Phase 3 of the proof-system repair (soft3 proposals/proof-system-repair)
//! retires it: the nox machine becomes one uniform step relation accumulated
//! in zheng, and the decider is one WHIR opening of the final accumulator,
//! run by zheng, not by a nox jet.
//!
//! Registry calling convention:
//!   object = [proof_data | instance_data]
//!   proof_data    = opaque proof bytes handed to the call provider
//!   instance_data = the public instance handed to the call provider
//!
//! Returns: whatever the call provider returns, unchecked (expected atom 1 or
//! 0); atom 0 when no provider answers.
//! Budget: a fixed legacy charge (DECIDER_COST); it measures no verification.
//!
//! The formula hash uniquely identifies this jet; see jets/formulas.rs.

use crate::call::CallProvider;
use crate::data::{Order, Reduction};
use crate::reduce::{ErrorKind, Outcome, pair_children};
use crate::trace::{TraceRow, Tracer};
use nebu::Goldilocks;

/// Fixed legacy charge of the retired decider jet; it corresponds to no
/// verification work (the jet verifies nothing).
const DECIDER_COST: u64 = 825;

/// Call-provider tag for the decider operation.
const DECIDER_CALL_TAG: u64 = 0xDEC1_DEC1;

pub fn decider_jet<const N: usize>(
    reduction: &mut Reduction<N>,
    object: Order,
    _body: Order,
    budget: u64,
    hints: &dyn CallProvider<N>,
    _tracer: &mut dyn Tracer,
    _depth: u64,
    row: &mut TraceRow,
) -> Outcome {
    // object = [proof | instance]
    let (proof_id, instance_id) = match pair_children(reduction, object) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };

    if budget < DECIDER_COST {
        return Outcome::Halt(budget);
    }
    let remaining = budget - DECIDER_COST;

    row.r[4] = proof_id as u64;
    row.r[5] = instance_id as u64;

    let tag = Goldilocks::new(DECIDER_CALL_TAG);
    let result = match hints.provide(reduction, tag, object) {
        Some(r) => r,
        None => match reduction.atom(Goldilocks::ZERO) {
            Some(r) => r,
            None => return Outcome::Error(ErrorKind::Unavailable),
        },
    };
    Outcome::Ok(result, remaining)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::call::NullCalls;
    use crate::data::Reduction;
    use crate::trace::{NoTrace, TraceRow};

    fn g(v: u64) -> Goldilocks {
        Goldilocks::new(v)
    }

    #[test]
    fn decider_with_null_calls_returns_zero() {
        let mut ar = Reduction::<256>::new();
        let proof = ar.atom(g(1)).unwrap();
        let instance = ar.atom(g(2)).unwrap();
        let obj = ar.pair(proof, instance).unwrap();
        let body = ar.atom(g(0)).unwrap();
        let mut row = TraceRow::default();
        match decider_jet(
            &mut ar,
            obj,
            body,
            10_000,
            &NullCalls,
            &mut NoTrace,
            0,
            &mut row,
        ) {
            Outcome::Ok(r, rem) => {
                assert_eq!(ar.atom_value(r).unwrap(), g(0));
                assert_eq!(rem, 10_000 - 825);
            }
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn decider_exhausted_budget_halts() {
        let mut ar = Reduction::<256>::new();
        let proof = ar.atom(g(1)).unwrap();
        let instance = ar.atom(g(2)).unwrap();
        let obj = ar.pair(proof, instance).unwrap();
        let body = ar.atom(g(0)).unwrap();
        let mut row = TraceRow::default();
        match decider_jet(
            &mut ar,
            obj,
            body,
            100,
            &NullCalls,
            &mut NoTrace,
            0,
            &mut row,
        ) {
            Outcome::Halt(_) => {}
            o => panic!("expected Halt, got {:?}", o),
        }
    }
}
