// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! State jets — cyber knowledge graph state machine operations.
//!
//! Six state operations, split by registration method:
//!
//!   Exact-match (fixed formula hash):
//!     CYBERLINK — record a directed link between two content IDs.
//!
//!   Template predicates (recognized by formula tag value > 17):
//!     TRANSFER  — transfer tokens between accounts.
//!     INSERT    — insert a new key→value entry into state.
//!     UPDATE    — update an existing key→value entry.
//!     AGGREGATE — fold a list of values using a combining formula.
//!     CONSERVE  — assert a conservation law across inputs and outputs.
//!
//! All jets read operation data from `object`. The formula encodes the
//! operation type via its tag value (constants defined in formulas.rs).
//! Jets delegate to the call provider (hints) for actual state mutation.
//! With NullCalls, state operations return Goldilocks::ZERO (failure).

use nebu::Goldilocks;
use crate::data::{Reduction, Order, Data};
use crate::reduce::{Outcome, ErrorKind, pair_children};
use crate::call::CallProvider;
use crate::trace::{Tracer, TraceRow};
use crate::jets::formulas::{
    TRANSFER_TAG, INSERT_TAG, UPDATE_TAG, AGGREGATE_TAG, CONSERVE_TAG,
};

/// Call-provider tags for each state operation.
const CYBERLINK_CALL_TAG:  u64 = 0xCEB1_1000;
const TRANSFER_CALL_TAG:   u64 = 0xCEB1_1001;
const INSERT_CALL_TAG:     u64 = 0xCEB1_1002;
const UPDATE_CALL_TAG:     u64 = 0xCEB1_1003;
const AGGREGATE_CALL_TAG:  u64 = 0xCEB1_1004;
const CONSERVE_CALL_TAG:   u64 = 0xCEB1_1005;

// ── helpers ───────────────────────────────────────────────────────────────────

/// Read the tag atom of a formula data (head of the pair).
fn formula_tag<const N: usize>(reduction: &Reduction<N>, formula: Order) -> Option<u64> {
    let inner = reduction.get(formula)?.inner;
    match inner {
        Data::Pair { left, .. } => {
            let v = reduction.atom_value(left)?;
            Some(v.as_u64())
        }
        _ => None,
    }
}

/// Delegate a state operation to the call provider.
/// Returns Goldilocks::ZERO if hints cannot service the request.
fn delegate<const N: usize>(
    reduction: &mut Reduction<N>, hints: &dyn CallProvider<N>,
    call_tag: u64, object: Order, budget: u64,
    row: &mut TraceRow,
) -> Outcome {
    if budget < 1 { return Outcome::Halt(budget); }
    row.r[4] = call_tag;
    row.r[5] = object as u64;
    let tag = Goldilocks::new(call_tag);
    let result = match hints.provide(reduction, tag, object) {
        Some(r) => r,
        None => match reduction.atom(Goldilocks::ZERO) {
            Some(r) => r,
            None => return Outcome::Error(ErrorKind::Unavailable),
        },
    };
    Outcome::Ok(result, budget - 1)
}

// ── CYBERLINK ─────────────────────────────────────────────────────────────────

/// Record a directed link between two content IDs.
///
/// object = [from_cid | to_cid]
/// Returns Field atom 1 (success) or 0 (failure) per hints response.
pub fn cyberlink_jet<const N: usize>(
    reduction: &mut Reduction<N>, object: Order, _body: Order, budget: u64,
    hints: &dyn CallProvider<N>, _tracer: &mut dyn Tracer, _depth: u64,
    row: &mut TraceRow,
) -> Outcome {
    if budget < 1 { return Outcome::Halt(budget); }
    let (from_id, to_id) = match pair_children(reduction, object) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    row.r[4] = from_id as u64;
    row.r[5] = to_id as u64;
    delegate(reduction, hints, CYBERLINK_CALL_TAG, object, budget, row)
}

// ── TRANSFER ─────────────────────────────────────────────────────────────────

/// Template predicate: formula tag == TRANSFER_TAG.
pub fn is_transfer<const N: usize>(reduction: &Reduction<N>, formula: Order) -> bool {
    formula_tag(reduction, formula) == Some(TRANSFER_TAG)
}

/// Transfer tokens between accounts.
///
/// object = [sender | [recipient | amount]]
pub fn transfer_jet<const N: usize>(
    reduction: &mut Reduction<N>, object: Order, _body: Order, budget: u64,
    hints: &dyn CallProvider<N>, _tracer: &mut dyn Tracer, _depth: u64,
    row: &mut TraceRow,
) -> Outcome {
    delegate(reduction, hints, TRANSFER_CALL_TAG, object, budget, row)
}

// ── INSERT ────────────────────────────────────────────────────────────────────

/// Template predicate: formula tag == INSERT_TAG.
pub fn is_insert<const N: usize>(reduction: &Reduction<N>, formula: Order) -> bool {
    formula_tag(reduction, formula) == Some(INSERT_TAG)
}

/// Insert a new key→value entry into the state graph.
///
/// object = [key | value]
pub fn insert_jet<const N: usize>(
    reduction: &mut Reduction<N>, object: Order, _body: Order, budget: u64,
    hints: &dyn CallProvider<N>, _tracer: &mut dyn Tracer, _depth: u64,
    row: &mut TraceRow,
) -> Outcome {
    delegate(reduction, hints, INSERT_CALL_TAG, object, budget, row)
}

// ── UPDATE ────────────────────────────────────────────────────────────────────

/// Template predicate: formula tag == UPDATE_TAG.
pub fn is_update<const N: usize>(reduction: &Reduction<N>, formula: Order) -> bool {
    formula_tag(reduction, formula) == Some(UPDATE_TAG)
}

/// Update an existing key→value entry in the state graph.
///
/// object = [key | new_value]
pub fn update_jet<const N: usize>(
    reduction: &mut Reduction<N>, object: Order, _body: Order, budget: u64,
    hints: &dyn CallProvider<N>, _tracer: &mut dyn Tracer, _depth: u64,
    row: &mut TraceRow,
) -> Outcome {
    delegate(reduction, hints, UPDATE_CALL_TAG, object, budget, row)
}

// ── AGGREGATE ────────────────────────────────────────────────────────────────

/// Template predicate: formula tag == AGGREGATE_TAG.
pub fn is_aggregate<const N: usize>(reduction: &Reduction<N>, formula: Order) -> bool {
    formula_tag(reduction, formula) == Some(AGGREGATE_TAG)
}

/// Aggregate a list of state values using a combining formula.
///
/// object = [combiner_formula | [values_list | accumulator]]
/// Delegates to hints for the actual aggregation logic.
pub fn aggregate_jet<const N: usize>(
    reduction: &mut Reduction<N>, object: Order, _body: Order, budget: u64,
    hints: &dyn CallProvider<N>, _tracer: &mut dyn Tracer, _depth: u64,
    row: &mut TraceRow,
) -> Outcome {
    delegate(reduction, hints, AGGREGATE_CALL_TAG, object, budget, row)
}

// ── CONSERVE ─────────────────────────────────────────────────────────────────

/// Template predicate: formula tag == CONSERVE_TAG.
pub fn is_conserve<const N: usize>(reduction: &Reduction<N>, formula: Order) -> bool {
    formula_tag(reduction, formula) == Some(CONSERVE_TAG)
}

/// Assert a conservation law: sum of inputs equals sum of outputs.
///
/// object = [inputs_list | outputs_list]
/// Returns Field atom 1 if conservation holds, 0 otherwise.
pub fn conserve_jet<const N: usize>(
    reduction: &mut Reduction<N>, object: Order, _body: Order, budget: u64,
    hints: &dyn CallProvider<N>, _tracer: &mut dyn Tracer, _depth: u64,
    row: &mut TraceRow,
) -> Outcome {
    delegate(reduction, hints, CONSERVE_CALL_TAG, object, budget, row)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::call::NullCalls;
    use crate::trace::{NoTrace, TraceRow};
    use crate::data::{Reduction};

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    #[test]
    fn cyberlink_with_null_calls_returns_zero() {
        let mut ar = Reduction::<256>::new();
        let from = ar.atom(g(1)).unwrap();
        let to   = ar.atom(g(2)).unwrap();
        let obj  = ar.pair(from, to).unwrap();
        let body = ar.atom(g(0)).unwrap();
        let mut row = TraceRow::default();
        match cyberlink_jet(&mut ar, obj, body, 100, &NullCalls, &mut NoTrace, 0, &mut row) {
            Outcome::Ok(r, rem) => {
                assert_eq!(ar.atom_value(r).unwrap(), g(0));
                assert_eq!(rem, 99);
            }
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn transfer_with_null_calls_returns_zero() {
        let mut ar = Reduction::<256>::new();
        let sender = ar.atom(g(10)).unwrap();
        let recip  = ar.atom(g(20)).unwrap();
        let amount = ar.atom(g(100)).unwrap();
        let inner  = ar.pair(recip, amount).unwrap();
        let obj    = ar.pair(sender, inner).unwrap();
        let body   = ar.atom(g(0)).unwrap();
        let mut row = TraceRow::default();
        match transfer_jet(&mut ar, obj, body, 100, &NullCalls, &mut NoTrace, 0, &mut row) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap(), g(0)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn predicates_match_correct_tags() {
        let mut ar = Reduction::<256>::new();
        // Build formulas with the expected tags.
        let mk = |ar: &mut Reduction<256>, tag: u64| -> Order {
            let t = ar.atom(g(tag)).unwrap();
            let b = ar.atom(g(0)).unwrap();
            ar.pair(t, b).unwrap()
        };
        let f_transfer  = mk(&mut ar, TRANSFER_TAG);
        let f_insert    = mk(&mut ar, INSERT_TAG);
        let f_update    = mk(&mut ar, UPDATE_TAG);
        let f_aggregate = mk(&mut ar, AGGREGATE_TAG);
        let f_conserve  = mk(&mut ar, CONSERVE_TAG);
        let f_other     = mk(&mut ar, 42); // unrelated tag

        assert!( is_transfer(&ar, f_transfer));
        assert!(!is_transfer(&ar, f_insert));
        assert!( is_insert(&ar, f_insert));
        assert!(!is_insert(&ar, f_update));
        assert!( is_update(&ar, f_update));
        assert!( is_aggregate(&ar, f_aggregate));
        assert!( is_conserve(&ar, f_conserve));
        assert!(!is_conserve(&ar, f_other));
    }

    #[test]
    fn budget_zero_halts() {
        let mut ar = Reduction::<256>::new();
        let obj  = ar.atom(g(0)).unwrap();
        let body = ar.atom(g(0)).unwrap();
        let mut row = TraceRow::default();
        match cyberlink_jet(&mut ar, obj, body, 0, &NullCalls, &mut NoTrace, 0, &mut row) {
            Outcome::Halt(_) => {}
            o => panic!("expected Halt, got {:?}", o),
        }
    }
}
