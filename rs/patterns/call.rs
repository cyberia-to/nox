//! pattern 16: call — non-deterministic witness injection
//! step 1: evaluate tag formula
//! step 2: call prover for witness (None -> Halt)
//! step 3: validate check(witness, object) = 0
//! step 4: return witness

use nebu::Goldilocks;
use crate::noun::{Order, NounId};
use crate::reduce::{Outcome, ErrorKind, cell_pair, evaluate};
use crate::call::CallProvider;
use crate::trace::{Tracer, TraceRow};
use crate::noun::NIL;

pub fn call_witness<const N: usize, T: Tracer>(
    order: &mut Order<N>, object: NounId, body: NounId, budget: u64,
    calls: &dyn CallProvider<N>, tracer: &mut T, depth: u64,
    row: &mut TraceRow,
) -> Outcome {
    let (tag_formula, check_formula) = match cell_pair(order, body) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    let (tag_result, budget) = match evaluate(order, object, tag_formula, budget, calls, tracer, depth) {
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
    // propagate the actual error/halt from check formula evaluation —
    // a Malformed/TypeError/Unavailable in check_f is a bug in check_f
    // itself, NOT a witness rejection. only a finished check returning
    // non-zero counts as CallRejected (handled below).
    let (check_result, budget) = match evaluate(order, witness_object, check_formula, budget, calls, tracer, depth) {
        Ok(v) => v,
        Err(o) => return o,
    };
    match order.atom_value(check_result) {
        Some((v, _)) if v == Goldilocks::ZERO => {
            row.r[4] = tag_value.as_u64();
            row.r[5] = witness as u64;
            row.r[6] = check_result as u64;
            Outcome::Ok(witness, budget)
        }
        _ => {
            row.r[4] = tag_value.as_u64();
            row.r[5] = witness as u64;
            row.r[6] = NIL as u64;
            Outcome::Error(ErrorKind::CallRejected)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::reduce::{reduce, Outcome, ErrorKind};
    use crate::call::{CallProvider, LookProvider, NullCalls};
    use crate::trace::NoTrace;
    use crate::noun::{Order, NounId, Tag};
    use nebu::Goldilocks;

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    #[test]
    fn call_null_provider_halts() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        // formula = [16 [[1 0] [1 [0 1]]]]
        // tag = quote(0), check = quote(identity = axis(1))
        let t16 = ar.atom(g(16), Tag::Field).unwrap();
        let t1 = ar.atom(g(1), Tag::Field).unwrap();
        let zero = ar.atom(g(0), Tag::Field).unwrap();
        let tag_formula = ar.cell(t1, zero).unwrap();
        let t0 = ar.atom(g(0), Tag::Field).unwrap();
        let axis1 = ar.cell(t0, t1).unwrap(); // [0 1] = axis(1)
        let check_formula = ar.cell(t1, axis1).unwrap(); // [1 [0 1]] = quote(axis(1))
        let body = ar.cell(tag_formula, check_formula).unwrap();
        let formula = ar.cell(t16, body).unwrap();
        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Halt(_) => {}
            o => panic!("expected Halt, got {:?}", o),
        }
    }

    #[test]
    fn call_rejected_on_bad_witness() {
        struct BadWitness;
        impl LookProvider for BadWitness {
            fn look(&self, _: Goldilocks, _: Goldilocks) -> Option<Goldilocks> { None }
        }
        impl<const N: usize> CallProvider<N> for BadWitness {
            fn provide(&self, order: &mut Order<N>, _tag: Goldilocks, _object: NounId) -> Option<NounId> {
                // provide witness = 99, but check expects 0
                Some(order.atom(g(99), Tag::Field).unwrap())
            }
        }

        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        // formula = [16 [[1 0] [1 99]]]
        // tag=quote(0), check=quote(99) — check returns 99 ≠ 0 → CallRejected
        let t16 = ar.atom(g(16), Tag::Field).unwrap();
        let t1 = ar.atom(g(1), Tag::Field).unwrap();
        let zero = ar.atom(g(0), Tag::Field).unwrap();
        let tag_f = ar.cell(t1, zero).unwrap();
        let ninety_nine = ar.atom(g(99), Tag::Field).unwrap();
        let check_f = ar.cell(t1, ninety_nine).unwrap();
        let body = ar.cell(tag_f, check_f).unwrap();
        let formula = ar.cell(t16, body).unwrap();
        match reduce(&mut ar, obj, formula, 1000, &BadWitness, &mut NoTrace) {
            Outcome::Error(ErrorKind::CallRejected) => {}
            o => panic!("expected CallRejected, got {:?}", o),
        }
    }
}
