//! pattern 13: not — bitwise complement (word type, 32-bit, multi-row)
//!
//! cost 32. emits 32 rows. unary: r5 / r11 (b register) are zero on every row.
//! per-row constraint: c_k = 1 - a_k (with a_k boolean).

use crate::data::{Reduction, Order};
use crate::reduce::{Outcome, ErrorKind, evaluate_unary_word, emit_bit_row, WORD_MASK};
use crate::call::CallProvider;
use crate::trace::{Tracer, TraceRow};
use crate::jets::registry::JetRegistry;
use nebu::Goldilocks;

pub fn not<const N: usize, T: Tracer>(
    reduction: &mut Reduction<N>, object: Order, body: Order, budget: u64,
    hints: &dyn CallProvider<N>, tracer: &mut T, depth: u64,
    row: &mut TraceRow, registry: &JetRegistry<N>,
) -> Outcome {
    let (a, budget) = match evaluate_unary_word(reduction, object, body, budget, hints, tracer, depth, registry) {
        Ok(v) => v, Err(o) => return o,
    };
    let c = (!a) & WORD_MASK;
    let result = match reduction.atom(Goldilocks::new(c)) {
        Some(r) => r,
        None => return Outcome::Error(ErrorKind::Unavailable),
    };
    for k in 0..32u32 {
        let a_k = (a >> k) & 1;
        let c_k = (c >> k) & 1;
        emit_bit_row(row, tracer, a, 0, c, k, a_k, 0, c_k, k == 31, result, budget);
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

    fn make_not<const N: usize>(ar: &mut Reduction<N>, v: u64) -> crate::data::Order {
        let t = ar.atom(g(13)).unwrap();
        let t1 = ar.atom(g(1)).unwrap();
        let vv = ar.atom(g(v)).unwrap();
        let body = ar.pair(t1, vv).unwrap();
        ar.pair(t, body).unwrap()
    }

    #[test]
    fn not_zero_gives_all_ones() {
        let mut ar = Reduction::<1024>::new();
        let obj = ar.atom(g(0)).unwrap();
        let formula = make_not(&mut ar, 0);
        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().as_u64(), 0xFFFF_FFFF),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn not_pattern() {
        let mut ar = Reduction::<1024>::new();
        let obj = ar.atom(g(0)).unwrap();
        let formula = make_not(&mut ar, 0xAAAA_AAAA);
        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().as_u64(), 0x5555_5555),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn not_emits_32_rows() {
        let mut ar = Reduction::<1024>::new();
        let obj = ar.atom(g(0)).unwrap();
        let formula = make_not(&mut ar, 0xDEADBEEF);
        let mut tr = VecTrace::default();
        reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut tr);
        let n = tr.0.iter().filter(|r| r.r[0] == 13).count();
        assert_eq!(n, 32);
    }

    #[test]
    fn not_bit_witnesses() {
        let mut ar = Reduction::<1024>::new();
        let obj = ar.atom(g(0)).unwrap();
        let a = 0xC0FFEEFEu64;
        let formula = make_not(&mut ar, a);
        let mut tr = VecTrace::default();
        reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut tr);
        let rows: Vec<_> = tr.0.iter().filter(|r| r.r[0] == 13).collect();
        for (k, r) in rows.iter().enumerate() {
            let a_k = (a >> k) & 1;
            let c_k = 1 - a_k;
            assert_eq!(r.r[10], a_k);
            assert_eq!(r.r[11], 0, "b is unused for unary not");
            assert_eq!(r.r[12], c_k);
        }
    }
}
