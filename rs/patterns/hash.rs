//! pattern 15: hash — structural hash (multi-row, cost 300)
//!
//! returns hash noun: cell(cell(h0,h1), cell(h2,h3))
//!
//! Status: SUMMARY-ROW STUB. The hash *result* is correct — the digest is
//! taken from the hash-consed structural hash maintained by Order, not
//! recomputed here. The *trace* is incomplete: we emit one summary row
//! instead of the ~300 rows the constraint system needs.
//!
//! To match specs/trace.md §pattern 15 the multi-row emission must cover:
//!   - absorption row(s): input bytes folded into the sponge state (rate phase)
//!   - permutation rows: per-round Poseidon2 state evolution (72 rounds total —
//!     4 + 4 full rounds with x^7 S-box, plus 16 partial rounds with x^-1 S-box)
//!   - squeeze row(s): output digest extraction from the sponge state
//!
//! Blocked on: hemera step-level API (`hemera::tree::step` or equivalent)
//! that exposes per-round state for trace emission. Until then, the prover
//! cannot bind the digest in r3 to a witness of the Poseidon2 computation.
//! reduce.rs skips post-hoc recording for tag=15 (see is_multi_row).

use crate::noun::{Order, NounId, NIL};
use crate::reduce::{Outcome, ErrorKind, evaluate};
use crate::call::CallProvider;
use crate::trace::{Tracer, TraceRow};

pub fn hash<const N: usize, T: Tracer>(
    order: &mut Order<N>, object: NounId, body: NounId, budget: u64,
    hints: &dyn CallProvider<N>, tracer: &mut T, depth: u64,
    row: &mut TraceRow,
) -> Outcome {
    let (input, budget) = match evaluate(order, object, body, budget, hints, tracer, depth) {
        Ok(v) => v, Err(o) => return o,
    };
    let digest = match order.digest(input) {
        Some(d) => *d,
        None => return Outcome::Error(ErrorKind::Unavailable),
    };
    let outcome = match order.hash_noun(&digest) {
        Some(r) => Outcome::Ok(r, budget),
        None => Outcome::Error(ErrorKind::Unavailable),
    };
    row.r[3] = match &outcome { Outcome::Ok(r, _) => *r as u64, _ => NIL as u64 };
    row.r[4] = input as u64;
    row.r[9] = match &outcome { Outcome::Ok(_, b) | Outcome::Halt(b) => *b, Outcome::Error(_) => 0 };
    row.r[10] = match &outcome { Outcome::Error(k) => *k as u64, _ => 0 };
    tracer.record(*row);
    outcome
}

#[cfg(test)]
mod tests {
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
    fn hash_emits_one_row() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        let formula = make_hash(&mut ar, 7);
        let mut tracer = VecTrace::default();
        match reduce(&mut ar, obj, formula, 10000, &NullCalls, &mut tracer) {
            Outcome::Ok(_, _) => {}
            o => panic!("{:?}", o),
        }
        let hash_rows = tracer.0.iter().filter(|r| r.r[0] == 15).count();
        assert_eq!(hash_rows, 1, "hash stub emits 1 row for its own computation");
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
