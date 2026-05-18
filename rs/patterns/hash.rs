//! pattern 15: hash — structural hash (multi-row, cost 25)
//!
//! Drives a [`hemera::StepSponge`] one round at a time and emits one trace
//! row per Poseidon2 round (24 rounds) plus a final squeeze row — 25 rows
//! total. The input rate is the noun's cached structural digest (4 elements,
//! zero-padded to RATE = 8). The output is a 4-element digest derived from
//! the post-permutation state, wrapped into a hash-noun `cell(cell(h0,h1),
//! cell(h2,h3))`.
//!
//! Per-round register layout (k = 0..23):
//!   r0  = 15 (tag)               r1 = object NounId
//!   r2  = formula NounId         r3 = NIL (set only on squeeze row)
//!   r4..r7  = state[0..3]        r10..r13 = state[4..7] (rate region)
//!   r8  = budget_in              r9 = 0 (set only on squeeze row)
//!   r14 = round index k          r15 = input NounId
//!
//! Squeeze row (k = 24):
//!   r3  = result NounId          r4..r7 = output digest (4 elements)
//!   r9  = budget_out             r14 = 24 (sentinel)
//!   r10..r13 = state[4..7] of the final post-permutation state
//!
//! Reduce.rs skips post-hoc recording for tag=15 (see is_multi_row).

use hemera::StepSponge;
use hemera::field::Goldilocks as HemeraGold;
use nebu::Goldilocks;
use crate::noun::{Order, NounId};
use crate::noun::hash::Digest;
use crate::reduce::{Outcome, ErrorKind, evaluate_unary};
use crate::call::CallProvider;
use crate::trace::{Tracer, TraceRow};
use crate::jets::registry::JetRegistry;

const RATE: usize = 8;
const ROUNDS_TOTAL: u64 = 24;

pub fn hash<const N: usize, T: Tracer>(
    order: &mut Order<N>, object: NounId, body: NounId, budget: u64,
    hints: &dyn CallProvider<N>, tracer: &mut T, depth: u64,
    row: &mut TraceRow, registry: &JetRegistry<N>,
) -> Outcome {
    let (input, budget) = match evaluate_unary(order, object, body, budget, hints, tracer, depth, registry) {
        Ok(v) => v, Err(o) => return o,
    };
    let in_digest: Digest = match order.digest(input) {
        Some(d) => *d,
        None => return Outcome::Error(ErrorKind::Unavailable),
    };

    // Pack the 4-element digest into the 8-element rate input.
    // nebu::Goldilocks and hemera::field::Goldilocks both wrap u64 over the
    // same Goldilocks prime — they are distinct Rust types but interchangeable
    // by canonical value. Convert via as_u64().
    let mut rate = [HemeraGold::ZERO; RATE];
    for i in 0..4 {
        rate[i] = HemeraGold::new(in_digest[i].as_u64());
    }

    let template = *row;
    let budget_in = template.r()[8];

    let mut sponge = StepSponge::absorb(&rate);
    let mut last_state = [HemeraGold::ZERO; 16];
    let mut k: u64 = 0;
    while !sponge.done() {
        let state = sponge.step();
        last_state = state;
        emit_round_row(tracer, &template, &state, k, input, budget_in);
        k += 1;
    }

    // Squeeze: 4-element output digest is state[0..4] of the post-permutation state.
    let out_digest: Digest = [
        Goldilocks::new(last_state[0].as_canonical_u64()),
        Goldilocks::new(last_state[1].as_canonical_u64()),
        Goldilocks::new(last_state[2].as_canonical_u64()),
        Goldilocks::new(last_state[3].as_canonical_u64()),
    ];
    let result = match order.hash_noun(&out_digest) {
        Some(r) => r,
        None => return Outcome::Error(ErrorKind::Unavailable),
    };
    emit_squeeze_row(tracer, &template, &last_state, &out_digest, result, input, budget_in, budget);

    Outcome::Ok(result, budget)
}

fn emit_round_row<T: Tracer>(
    tracer: &mut T, template: &TraceRow,
    state: &[HemeraGold; 16], k: u64, input: NounId, budget_in: u64,
) {
    let mut r = TraceRow::default();
    r.r[0] = template.r()[0];
    r.r[1] = template.r()[1];
    r.r[2] = template.r()[2];
    r.r[4] = state[0].as_canonical_u64();
    r.r[5] = state[1].as_canonical_u64();
    r.r[6] = state[2].as_canonical_u64();
    r.r[7] = state[3].as_canonical_u64();
    r.r[8] = budget_in;
    r.r[10] = state[4].as_canonical_u64();
    r.r[11] = state[5].as_canonical_u64();
    r.r[12] = state[6].as_canonical_u64();
    r.r[13] = state[7].as_canonical_u64();
    r.r[14] = k;
    r.r[15] = input as u64;
    tracer.record(r);
}

fn emit_squeeze_row<T: Tracer>(
    tracer: &mut T, template: &TraceRow,
    final_state: &[HemeraGold; 16], out_digest: &Digest,
    result: NounId, input: NounId, budget_in: u64, budget_out: u64,
) {
    let mut r = TraceRow::default();
    r.r[0] = template.r()[0];
    r.r[1] = template.r()[1];
    r.r[2] = template.r()[2];
    r.r[3] = result as u64;
    r.r[4] = out_digest[0].as_u64();
    r.r[5] = out_digest[1].as_u64();
    r.r[6] = out_digest[2].as_u64();
    r.r[7] = out_digest[3].as_u64();
    r.r[8] = budget_in;
    r.r[9] = budget_out;
    r.r[10] = final_state[4].as_canonical_u64();
    r.r[11] = final_state[5].as_canonical_u64();
    r.r[12] = final_state[6].as_canonical_u64();
    r.r[13] = final_state[7].as_canonical_u64();
    r.r[14] = ROUNDS_TOTAL; // 24 — sentinel for squeeze
    r.r[15] = input as u64;
    tracer.record(r);
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::vec::Vec;
    use crate::reduce::{reduce, Outcome};
    use crate::call::NullCalls;
    use crate::trace::{NoTrace, VecTrace};
    use crate::noun::{Order, Tag, Noun};
    use nebu::Goldilocks;

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    fn make_hash<const N: usize>(ar: &mut Order<N>, val: u64) -> crate::noun::NounId {
        let t15 = ar.atom(g(15), Tag::Field).unwrap();
        let t1 = ar.atom(g(1), Tag::Field).unwrap();
        let vval = ar.atom(g(val), Tag::Field).unwrap();
        let body = ar.cell(t1, vval).unwrap();
        ar.cell(t15, body).unwrap()
    }

    #[test]
    fn hash_returns_cell_of_cells() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        let formula = make_hash(&mut ar, 42);
        match reduce(&mut ar, obj, formula, 10000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => {
                // result should be [[h0 h1] [h2 h3]]
                assert!(matches!(ar.get(r).unwrap().inner, Noun::Cell { .. }));
            }
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn hash_emits_25_rows() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        let formula = make_hash(&mut ar, 7);
        let mut tracer = VecTrace::default();
        match reduce(&mut ar, obj, formula, 10000, &NullCalls, &mut tracer) {
            Outcome::Ok(_, _) => {}
            o => panic!("{:?}", o),
        }
        let hash_rows = tracer.0.iter().filter(|r| r.r()[0] == 15).count();
        assert_eq!(hash_rows, 25, "hash emits 24 round rows + 1 squeeze row");
    }

    #[test]
    fn hash_round_counter_increments() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        let formula = make_hash(&mut ar, 11);
        let mut tracer = VecTrace::default();
        reduce(&mut ar, obj, formula, 10000, &NullCalls, &mut tracer);
        let hash_rows: Vec<_> = tracer.0.iter().filter(|r| r.r()[0] == 15).collect();
        for (i, row) in hash_rows.iter().enumerate() {
            assert_eq!(row.r()[14], i as u64, "row {} has counter {}", i, row.r()[14]);
        }
    }

    #[test]
    fn hash_squeeze_row_carries_result() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        let formula = make_hash(&mut ar, 11);
        let mut tracer = VecTrace::default();
        let result_id = match reduce(&mut ar, obj, formula, 10000, &NullCalls, &mut tracer) {
            Outcome::Ok(r, _) => r,
            o => panic!("{:?}", o),
        };
        let last = tracer.0.iter().filter(|r| r.r()[0] == 15).last().unwrap();
        assert_eq!(last.r()[3], result_id as u64);
        assert_eq!(last.r()[14], 24, "squeeze sentinel is 24");
    }

    #[test]
    fn hash_deterministic() {
        let mut ar1 = Order::<1024>::new();
        let obj1 = ar1.atom(g(0), Tag::Field).unwrap();
        let f1 = make_hash(&mut ar1, 99);
        let r1 = match reduce(&mut ar1, obj1, f1, 10000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => ar1.read_hash_noun(r).unwrap(),
            o => panic!("{:?}", o),
        };

        let mut ar2 = Order::<1024>::new();
        let obj2 = ar2.atom(g(0), Tag::Field).unwrap();
        let f2 = make_hash(&mut ar2, 99);
        let r2 = match reduce(&mut ar2, obj2, f2, 10000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => ar2.read_hash_noun(r).unwrap(),
            o => panic!("{:?}", o),
        };

        assert_eq!(r1, r2);
    }
}
