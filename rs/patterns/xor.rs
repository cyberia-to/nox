//! pattern 11: xor — bitwise XOR (word type, 32-bit, multi-row)
//!
//! cost 32. emits 32 rows, one per bit position. each row exposes
//! a_k, b_k, c_k = a_k XOR b_k so zheng can constrain the XOR gadget
//! c_k = a_k + b_k - 2 * a_k * b_k per row and bind sum(2^k * a_k) = a
//! across the 32-row block.

use crate::data::{Order, OrderId};
use crate::reduce::{Outcome, ErrorKind, pair_children, evaluate_binary_word, emit_bit_row, WORD_MASK};
use crate::call::CallProvider;
use crate::trace::{Tracer, TraceRow};
use crate::jets::registry::JetRegistry;
use nebu::Goldilocks;

pub fn xor<const N: usize, T: Tracer>(
    order: &mut Order<N>, object: OrderId, body: OrderId, budget: u64,
    hints: &dyn CallProvider<N>, tracer: &mut T, depth: u64,
    row: &mut TraceRow, registry: &JetRegistry<N>,
) -> Outcome {
    let (af, bf) = match pair_children(order, body) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    let (a, b, budget) = match evaluate_binary_word(order, object, af, bf, budget, hints, tracer, depth, registry) {
        Ok(v) => v, Err(o) => return o,
    };
    let c = (a ^ b) & WORD_MASK;
    let result = match order.atom(Goldilocks::new(c)) {
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
    use crate::data::{Order};
    use nebu::Goldilocks;

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    fn make_xor<const N: usize>(ar: &mut Order<N>, a: u64, b: u64) -> crate::data::OrderId {
        let t = ar.atom(g(11)).unwrap();
        let t1 = ar.atom(g(1)).unwrap();
        let va = ar.atom(g(a)).unwrap();
        let vb = ar.atom(g(b)).unwrap();
        let qa = ar.pair(t1, va).unwrap();
        let qb = ar.pair(t1, vb).unwrap();
        let body = ar.pair(qa, qb).unwrap();
        ar.pair(t, body).unwrap()
    }

    #[test]
    fn xor_basic() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0)).unwrap();
        let formula = make_xor(&mut ar, 0b1100, 0b1010);
        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().as_u64(), 0b0110),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn xor_self_is_zero() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0)).unwrap();
        let formula = make_xor(&mut ar, 0xDEADBEEF, 0xDEADBEEF);
        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().as_u64(), 0),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn xor_emits_32_rows() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0)).unwrap();
        let formula = make_xor(&mut ar, 0xDEADBEEF, 0xCAFEBABE);
        let mut tr = VecTrace::default();
        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut tr) {
            Outcome::Ok(_, _) => {}
            o => panic!("{:?}", o),
        }
        let xor_rows = tr.0.iter().filter(|r| r.r[0] == 11).count();
        assert_eq!(xor_rows, 32, "xor emits one row per bit");
    }

    #[test]
    fn xor_bit_witnesses_match_packed() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0)).unwrap();
        let a = 0xA5A5_A5A5u64;
        let b = 0x5555_5555u64;
        let formula = make_xor(&mut ar, a, b);
        let mut tr = VecTrace::default();
        reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut tr);
        let rows: Vec<_> = tr.0.iter().filter(|r| r.r[0] == 11).collect();
        assert_eq!(rows.len(), 32);
        for (k, r) in rows.iter().enumerate() {
            assert_eq!(r.r[7], k as u64, "row k has bit position k");
            assert_eq!(r.r[10], (a >> k) & 1, "r10 = a bit k");
            assert_eq!(r.r[11], (b >> k) & 1, "r11 = b bit k");
            assert_eq!(r.r[12], ((a ^ b) >> k) & 1, "r12 = (a^b) bit k");
        }
    }
}
