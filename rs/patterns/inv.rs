//! pattern 8: inv — field inversion (Fermat, cost 64)
//! inv(0) = Error(InvZero)
//!
//! Uses Fermat's little theorem: a^(p-2) mod p = a^-1
//! P_MINUS_2 = 0xFFFFFFFEFFFFFFFF (Goldilocks prime - 2)
//! Emits 64 rows: 1 init row + 63 step rows (one per exponent bit processed).

use nebu::Goldilocks;
use crate::noun::{Order, NounId};
use crate::reduce::{Outcome, ErrorKind, evaluate_field};
use crate::call::CallProvider;
use crate::trace::{Tracer, TraceRow};
use crate::noun::NIL;

const P_MINUS_2: u64 = 0xFFFFFFFEFFFFFFFF;

pub fn inv<const N: usize, T: Tracer>(
    order: &mut Order<N>, object: NounId, body: NounId, budget: u64,
    hints: &dyn CallProvider<N>, tracer: &mut T, depth: u64,
    row: &mut TraceRow,
) -> Outcome {
    let (v, budget) = match evaluate_field(order, object, body, budget, hints, tracer, depth) {
        Ok(v) => v,
        Err(o) => {
            row.r[3] = NIL as u64;
            row.r[9] = 0;
            row.r[10] = match &o {
                Outcome::Error(k) => *k as u64,
                _ => 0,
            };
            tracer.record(*row);
            return o;
        }
    };

    if v == Goldilocks::ZERO {
        row.r[4] = 0;
        row.r[3] = NIL as u64;
        row.r[10] = ErrorKind::InvZero as u64;
        tracer.record(*row);
        return Outcome::Error(ErrorKind::InvZero);
    }

    // Square-and-multiply: p-2 in binary, bit 63 is always 1 (MSB of p-2).
    // Initialize accumulator with v (corresponding to bit 63 = 1).
    let mut acc = v;

    // Row 0: initialization row (bit 63 processed, acc = v)
    row.r[4] = v.as_u64();
    row.r[10] = acc.as_u64();
    row.r[11] = 1; // bit 63 is 1
    row.r[12] = 0; // step index 0
    tracer.record(*row);

    // Steps 1..=63: process bits 62 down to 0
    for step in 1u64..=63 {
        let bit_pos = 63 - step;
        let bit = (P_MINUS_2 >> bit_pos) & 1;
        acc = acc * acc;
        if bit == 1 {
            acc = acc * v;
        }

        let mut step_row = TraceRow::default();
        // copy common registers from initial row
        step_row.r[0] = row.r[0];
        step_row.r[1] = row.r[1];
        step_row.r[2] = row.r[2];
        step_row.r[8] = row.r[8];

        step_row.r[4] = v.as_u64();
        step_row.r[10] = acc.as_u64();
        step_row.r[11] = bit;
        step_row.r[12] = step;

        if step == 63 {
            // final step: allocate result noun
            match order.atom(acc, crate::noun::Tag::Field) {
                Some(result) => {
                    step_row.r[3] = result as u64;
                    step_row.r[6] = acc.as_u64();
                    step_row.r[9] = budget;
                    tracer.record(step_row);
                    return Outcome::Ok(result, budget);
                }
                None => {
                    step_row.r[3] = NIL as u64;
                    step_row.r[10] = ErrorKind::Unavailable as u64;
                    tracer.record(step_row);
                    return Outcome::Error(ErrorKind::Unavailable);
                }
            }
        } else {
            tracer.record(step_row);
        }
    }

    // unreachable: loop covers steps 1..=63
    Outcome::Error(ErrorKind::Malformed)
}

#[cfg(test)]
mod tests {
    use crate::reduce::{reduce, Outcome, ErrorKind};
    use crate::call::NullCalls;
    use crate::trace::{NoTrace, VecTrace};
    use crate::noun::{Order, Tag};
    use nebu::Goldilocks;

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    /// formula = [8 [1 v]]  (inv of a quoted field element)
    fn make_inv<const N: usize>(ar: &mut Order<N>, v: u64) -> crate::noun::NounId {
        let t8 = ar.atom(g(8), Tag::Field).unwrap();
        let t1 = ar.atom(g(1), Tag::Field).unwrap();
        let val = ar.atom(g(v), Tag::Field).unwrap();
        let body = ar.cell(t1, val).unwrap();
        ar.cell(t8, body).unwrap()
    }

    #[test]
    fn inv_nonzero() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        let formula = make_inv(&mut ar, 2);
        match reduce(&mut ar, obj, formula, 10000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => {
                let (inv2, _) = ar.atom_value(r).unwrap();
                // inv(2) * 2 should equal 1 in the Goldilocks field
                assert_eq!(inv2 * g(2), g(1));
            }
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn inv_zero_errors() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        let formula = make_inv(&mut ar, 0);
        match reduce(&mut ar, obj, formula, 10000, &NullCalls, &mut NoTrace) {
            Outcome::Error(ErrorKind::InvZero) => {}
            o => panic!("expected InvZero, got {:?}", o),
        }
    }

    #[test]
    fn inv_emits_64_rows() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        let formula = make_inv(&mut ar, 3);
        let mut tracer = VecTrace::default();
        match reduce(&mut ar, obj, formula, 10000, &NullCalls, &mut tracer) {
            Outcome::Ok(_, _) => {}
            o => panic!("{:?}", o),
        }
        let inv_rows = tracer.0.iter().filter(|r| r.r[0] == 8).count();
        assert_eq!(inv_rows, 64, "inv emits 64 rows for its own computation");
    }
}
