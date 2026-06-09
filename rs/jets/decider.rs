// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! jet: decider — HyperNova/SuperSpartan proof verifier
//!
//! The decider verifies succinct proofs of nox execution. It is the
//! on-chain verification gate for cyber's proof-native state machine.
//!
//! Architecture: 89 primary constraints + 825 cross-term constraints
//! (HyperNova multi-folding scheme, SuperSpartan relaxed R1CS).
//!
//! Registry calling convention:
//!   object = [proof_data | instance_data]
//!   proof_data    = the folded proof (compressed witness)
//!   instance_data = the public instance to verify against
//!
//! Returns: Field atom 1 (valid proof) or 0 (invalid).
//! Budget: 825 (one unit per cross-term constraint verified).
//!
//! Implementation note: For 0.1.0, verification logic is delegated to the
//! call provider (hints). A full CPU verifier will be added in 0.2.0.
//! The formula hash uniquely identifies this jet; see jets/formulas.rs.

use nebu::Goldilocks;
use crate::data::{Order, OrderId};
use crate::reduce::{Outcome, ErrorKind, pair_children};
use crate::call::CallProvider;
use crate::trace::{Tracer, TraceRow};

/// Cost matching 825 cross-term constraints of HyperNova.
const DECIDER_COST: u64 = 825;

/// Call-provider tag for the decider operation.
const DECIDER_CALL_TAG: u64 = 0xDEC1_DEC1;

pub fn decider_jet<const N: usize>(
    order: &mut Order<N>, object: OrderId, _body: OrderId, budget: u64,
    hints: &dyn CallProvider<N>, _tracer: &mut dyn Tracer, _depth: u64,
    row: &mut TraceRow,
) -> Outcome {
    // object = [proof | instance]
    let (proof_id, instance_id) = match pair_children(order, object) {
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
    let result = match hints.provide(order, tag, object) {
        Some(r) => r,
        None => match order.atom(Goldilocks::ZERO) {
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
    use crate::trace::{NoTrace, TraceRow};
    use crate::data::{Order};

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    #[test]
    fn decider_with_null_calls_returns_zero() {
        let mut ar = Order::<256>::new();
        let proof    = ar.atom(g(1)).unwrap();
        let instance = ar.atom(g(2)).unwrap();
        let obj      = ar.pair(proof, instance).unwrap();
        let body     = ar.atom(g(0)).unwrap();
        let mut row  = TraceRow::default();
        match decider_jet(&mut ar, obj, body, 10_000, &NullCalls, &mut NoTrace, 0, &mut row) {
            Outcome::Ok(r, rem) => {
                assert_eq!(ar.atom_value(r).unwrap(), g(0));
                assert_eq!(rem, 10_000 - 825);
            }
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn decider_exhausted_budget_halts() {
        let mut ar = Order::<256>::new();
        let proof    = ar.atom(g(1)).unwrap();
        let instance = ar.atom(g(2)).unwrap();
        let obj      = ar.pair(proof, instance).unwrap();
        let body     = ar.atom(g(0)).unwrap();
        let mut row  = TraceRow::default();
        match decider_jet(&mut ar, obj, body, 100, &NullCalls, &mut NoTrace, 0, &mut row) {
            Outcome::Halt(_) => {}
            o => panic!("expected Halt, got {:?}", o),
        }
    }
}
