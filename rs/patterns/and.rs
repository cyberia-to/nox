//! pattern 12: and — bitwise AND (word type, 32-bit, multi-row)
//!
//! cost 32. emits 32 rows, one per bit. zheng constraint per row:
//! c_k = a_k * b_k, with row-linking sum(2^k * a_k) = a etc.

use crate::data::{Reduction, Order};
use crate::reduce::{Outcome, ErrorKind, pair_children, evaluate_binary_word, emit_bit_row, WORD_MASK};
use crate::call::CallProvider;
use crate::trace::{Tracer, TraceRow};
use crate::jets::registry::JetRegistry;
use nebu::Goldilocks;

pub fn and<const N: usize, T: Tracer>(
    reduction: &mut Reduction<N>, object: Order, body: Order, budget: u64,
    hints: &dyn CallProvider<N>, tracer: &mut T, depth: u64,
    row: &mut TraceRow, registry: &JetRegistry<N>,
) -> Outcome {
    let (af, bf) = match pair_children(reduction, body) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    let (a, b, budget) = match evaluate_binary_word(reduction, object, af, bf, budget, hints, tracer, depth, registry) {
        Ok(v) => v, Err(o) => return o,
    };
    let c = (a & b) & WORD_MASK;
    let result = match reduction.atom(Goldilocks::new(c)) {
        Some(r) => r,
        None => return Outcome::Error(ErrorKind::Unavailable),
    };
    for k in 0..32u32 {
        let a_k = (a >> k) & 1;
        let b_k = (b >> k) & 1;
        let c_k = (c >> k) & 1;
        emit_bit_row(row, tracer, a, b, c, k, a_k, b_k, c_k, k == 31, result, budget);
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
    use crate::data::{Reduction};
    use nebu::Goldilocks;

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    fn make_and<const N: usize>(ar: &mut Reduction<N>, a: u64, b: u64) -> crate::data::Order {
        let t = ar.atom(g(12)).unwrap();
        let t1 = ar.atom(g(1)).unwrap();
        let va = ar.atom(g(a)).unwrap();
        let vb = ar.atom(g(b)).unwrap();
        let qa = ar.pair(t1, va).unwrap();
        let qb = ar.pair(t1, vb).unwrap();
        let body = ar.pair(qa, qb).unwrap();
        ar.pair(t, body).unwrap()
    }

    #[test]
    fn and_basic() {
        let mut ar = Reduction::<1024>::new();
        let obj = ar.atom(g(0)).unwrap();
        let formula = make_and(&mut ar, 0b1100, 0b1010);
        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().as_u64(), 0b1000),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn and_with_zero() {
        let mut ar = Reduction::<1024>::new();
        let obj = ar.atom(g(0)).unwrap();
        let formula = make_and(&mut ar, 0xFF, 0);
        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().as_u64(), 0),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn and_emits_32_rows() {
        let mut ar = Reduction::<1024>::new();
        let obj = ar.atom(g(0)).unwrap();
        let formula = make_and(&mut ar, 0xDEADBEEF, 0xCAFEBABE);
        let mut tr = VecTrace::default();
        reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut tr);
        let n = tr.0.iter().filter(|r| r.r[0] == 12).count();
        assert_eq!(n, 32);
    }

    #[test]
    fn and_bit_witnesses_match_packed() {
        let mut ar = Reduction::<1024>::new();
        let obj = ar.atom(g(0)).unwrap();
        let a = 0xA5A5_A5A5u64;
        let b = 0xF0F0_F0F0u64;
        let formula = make_and(&mut ar, a, b);
        let mut tr = VecTrace::default();
        reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut tr);
        let rows: Vec<_> = tr.0.iter().filter(|r| r.r[0] == 12).collect();
        for (k, r) in rows.iter().enumerate() {
            assert_eq!(r.r[12], ((a & b) >> k) & 1);
        }
    }
}
