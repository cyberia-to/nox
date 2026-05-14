//! pattern 14: shl — shift left (word type, 32-bit, multi-row)
//!
//! cost 32. emits 32 rows, one per output bit position k.
//!
//! per-row registers:
//!   r4  = a (full input)        r5  = n (shift amount)
//!   r6  = c (full output)       r7  = k (output bit position)
//!   r10 = a_k                   (bit k of input — bound by sum decomp)
//!   r11 = source bit value      (= a_{k-n} when k >= n and k-n < 32, else 0)
//!   r12 = c_k                   (= r11 by definition of shl)
//!   r13 = source bit index      (k - n if in range, else 32 sentinel)
//!
//! zheng constraints: r10/r11/r12 are bits; sum 2^k * r10 = a; sum 2^k * r12 = c;
//! r12 = r11 per row; r11 ties to r10 of row (k - n).

use crate::noun::{Order, NounId, Tag};
use crate::reduce::{Outcome, ErrorKind, cell_pair, evaluate_word, WORD_MASK};
use crate::call::CallProvider;
use crate::trace::{Tracer, TraceRow};
use nebu::Goldilocks;

const SHIFT_OUT_OF_RANGE: u64 = 32;

pub fn shl<const N: usize, T: Tracer>(
    order: &mut Order<N>, object: NounId, body: NounId, budget: u64,
    hints: &dyn CallProvider<N>, tracer: &mut T, depth: u64,
    row: &mut TraceRow,
) -> Outcome {
    let (af, nf) = match cell_pair(order, body) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    let (a, budget) = match evaluate_word(order, object, af, budget, hints, tracer, depth) {
        Ok(v) => v, Err(o) => return o,
    };
    let (n, budget) = match evaluate_word(order, object, nf, budget, hints, tracer, depth) {
        Ok(v) => v, Err(o) => return o,
    };
    let c = if n >= 32 { 0 } else { (a << n) & WORD_MASK };
    let result = match order.atom(Goldilocks::new(c), Tag::Word) {
        Some(r) => r,
        None => return Outcome::Error(ErrorKind::Unavailable),
    };
    for k in 0..32u32 {
        let a_k = (a >> k) & 1;
        let c_k = (c >> k) & 1;
        let (src_bit, src_idx) = if (k as u64) >= n && ((k as u64) - n) < 32 {
            let idx = (k as u64) - n;
            ((a >> idx) & 1, idx)
        } else {
            (0, SHIFT_OUT_OF_RANGE)
        };
        let mut r = TraceRow::default();
        r.r[0] = row.r[0];
        r.r[1] = row.r[1];
        r.r[2] = row.r[2];
        r.r[4] = a;
        r.r[5] = n;
        r.r[6] = c;
        r.r[7] = k as u64;
        r.r[8] = row.r[8];
        r.r[10] = a_k;
        r.r[11] = src_bit;
        r.r[12] = c_k;
        r.r[13] = src_idx;
        if k == 31 {
            r.r[3] = result as u64;
            r.r[9] = budget;
        }
        tracer.record(r);
    }
    Outcome::Ok(result, budget)
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

    fn make_shl<const N: usize>(ar: &mut Order<N>, v: u64, sh: u64) -> crate::noun::NounId {
        let t = ar.atom(g(14), Tag::Field).unwrap();
        let t1 = ar.atom(g(1), Tag::Field).unwrap();
        let vv = ar.atom(g(v), Tag::Word).unwrap();
        let vs = ar.atom(g(sh), Tag::Word).unwrap();
        let qa = ar.cell(t1, vv).unwrap();
        let qb = ar.cell(t1, vs).unwrap();
        let body = ar.cell(qa, qb).unwrap();
        ar.cell(t, body).unwrap()
    }

    #[test]
    fn shl_basic() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        let formula = make_shl(&mut ar, 1, 3);
        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0.as_u64(), 8),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn shl_overflow_clamps_to_zero() {
        for sh in [32u64, 33, 63, 64, u32::MAX as u64] {
            let mut ar = Order::<1024>::new();
            let obj = ar.atom(g(0), Tag::Field).unwrap();
            let formula = make_shl(&mut ar, 1, sh);
            match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
                Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0.as_u64(), 0,
                                                "shift {} should give 0", sh),
                o => panic!("{:?}", o),
            }
        }
    }

    #[test]
    fn shl_masks_to_32_bits() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        let formula = make_shl(&mut ar, 0xFFFF_FFFF, 1);
        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0.as_u64(), 0xFFFF_FFFE),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn shl_emits_32_rows() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        let formula = make_shl(&mut ar, 0xDEADBEEF, 4);
        let mut tr = VecTrace::default();
        reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut tr);
        let n = tr.0.iter().filter(|r| r.r[0] == 14).count();
        assert_eq!(n, 32);
    }

    #[test]
    fn shl_bit_witnesses_correct() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        let a = 0xDEAD_BEEFu64;
        let n_shift = 4u64;
        let c = (a << n_shift) & 0xFFFF_FFFF;
        let formula = make_shl(&mut ar, a, n_shift);
        let mut tr = VecTrace::default();
        reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut tr);
        let rows: Vec<_> = tr.0.iter().filter(|r| r.r[0] == 14).collect();
        for (k, r) in rows.iter().enumerate() {
            let k = k as u32;
            assert_eq!(r.r[10], (a >> k) & 1);
            assert_eq!(r.r[12], (c >> k) & 1);
            if (k as u64) >= n_shift && ((k as u64) - n_shift) < 32 {
                let src = (k as u64) - n_shift;
                assert_eq!(r.r[11], (a >> src) & 1);
                assert_eq!(r.r[13], src);
            } else {
                assert_eq!(r.r[11], 0);
                assert_eq!(r.r[13], 32);
            }
        }
    }
}
