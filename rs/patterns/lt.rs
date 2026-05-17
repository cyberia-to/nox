//! pattern 10: lt — less-than on canonical Goldilocks representatives (multi-row)
//!
//! cost 64. emits 64 rows, one per bit of the 64-bit canonical representative.
//! exposes bit decomposition of both operands so zheng can constrain ordering
//! with a row-by-row "first differing bit" gadget and range-bind both inputs.
//!
//! per-row registers (k = 0..63, LSB-first):
//!   r4  = a (full)         r5  = b (full)
//!   r6  = result (0 = lt, 1 = ge)
//!   r7  = k (bit position)
//!   r10 = a_k              r11 = b_k
//!
//! result follows spec/patterns/10-lt.md: 0 when a < b, 1 otherwise.

use nebu::Goldilocks;
use crate::noun::{Order, NounId, Tag};
use crate::reduce::{Outcome, ErrorKind, cell_pair, evaluate_binary_field};
use crate::call::CallProvider;
use crate::trace::{Tracer, TraceRow};

pub fn lt<const N: usize, T: Tracer>(
    order: &mut Order<N>, object: NounId, body: NounId, budget: u64,
    hints: &dyn CallProvider<N>, tracer: &mut T, depth: u64,
    row: &mut TraceRow,
) -> Outcome {
    let (af, bf) = match cell_pair(order, body) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    let (va, vb, budget) = match evaluate_binary_field(order, object, af, bf, budget, hints, tracer, depth) {
        Ok(v) => v, Err(o) => return o,
    };
    let a = va.as_u64();
    let b = vb.as_u64();
    let is_lt = a < b;
    let result_val = if is_lt { Goldilocks::ZERO } else { Goldilocks::ONE };
    let result_id = match order.atom(result_val, Tag::Field) {
        Some(r) => r,
        None => return Outcome::Error(ErrorKind::Unavailable),
    };
    let result_u64: u64 = if is_lt { 0 } else { 1 };
    for k in 0..64u32 {
        let a_k = (a >> k) & 1;
        let b_k = (b >> k) & 1;
        let mut r = TraceRow::default();
        r.r[0] = row.r[0];
        r.r[1] = row.r[1];
        r.r[2] = row.r[2];
        r.r[4] = a;
        r.r[5] = b;
        r.r[6] = result_u64;
        r.r[7] = k as u64;
        r.r[8] = row.r[8];
        r.r[10] = a_k;
        r.r[11] = b_k;
        if k == 63 {
            r.r[3] = result_id as u64;
            r.r[9] = budget;
        }
        tracer.record(r);
    }
    Outcome::Ok(result_id, budget)
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::vec::Vec;
    use crate::reduce::{reduce, Outcome};
    use crate::call::NullCalls;
    use crate::trace::{NoTrace, VecTrace};
    use crate::noun::{Order, Tag};
    use nebu::Goldilocks;

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    fn make_lt<const N: usize>(ar: &mut Order<N>, a: u64, b: u64) -> crate::noun::NounId {
        let t10 = ar.atom(g(10), Tag::Field).unwrap();
        let t1 = ar.atom(g(1), Tag::Field).unwrap();
        let va = ar.atom(g(a), Tag::Field).unwrap();
        let vb = ar.atom(g(b), Tag::Field).unwrap();
        let qa = ar.cell(t1, va).unwrap();
        let qb = ar.cell(t1, vb).unwrap();
        let body = ar.cell(qa, qb).unwrap();
        ar.cell(t10, body).unwrap()
    }

    #[test]
    fn lt_less_returns_zero() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        let formula = make_lt(&mut ar, 3, 5);
        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(0)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn lt_greater_returns_one() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        let formula = make_lt(&mut ar, 5, 3);
        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(1)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn lt_equal_returns_one() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        let formula = make_lt(&mut ar, 5, 5);
        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(1)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn lt_emits_64_rows() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        let formula = make_lt(&mut ar, 100, 200);
        let mut tr = VecTrace::default();
        reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut tr);
        let n = tr.0.iter().filter(|r| r.r[0] == 10).count();
        assert_eq!(n, 64);
    }

    #[test]
    fn lt_bit_witnesses_correct() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        let a = 0xDEAD_BEEFu64;
        let b = 0xCAFE_BABEu64;
        let formula = make_lt(&mut ar, a, b);
        let mut tr = VecTrace::default();
        reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut tr);
        let rows: Vec<_> = tr.0.iter().filter(|r| r.r[0] == 10).collect();
        for (k, r) in rows.iter().enumerate() {
            assert_eq!(r.r[7], k as u64);
            assert_eq!(r.r[10], (a >> k) & 1);
            assert_eq!(r.r[11], (b >> k) & 1);
        }
    }
}
