//! pattern 9: eq — equality by noun identity (hash comparison)
//!
//! works for ALL noun types: atoms, cells, hash nouns. returns 0 if equal,
//! 1 if not equal. when both operands are atoms, r4/r5 carry their canonical
//! field values and r7 carries the non-equality inverse hint so zheng can
//! constrain the gadget (r4 - r5) * r7 = r6 with r6 ∈ {0, 1}.

use nebu::Goldilocks;
use crate::noun::{Order, NounId};
use crate::reduce::{Outcome, ErrorKind, cell_pair, evaluate_binary, make_field};
use crate::call::CallProvider;
use crate::trace::{Tracer, TraceRow};

pub fn eq<const N: usize, T: Tracer>(
    order: &mut Order<N>, object: NounId, body: NounId, budget: u64,
    hints: &dyn CallProvider<N>, tracer: &mut T, depth: u64,
    row: &mut TraceRow,
) -> Outcome {
    let (a, b) = match cell_pair(order, body) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    let (ra, rb, budget) = match evaluate_binary(order, object, a, b, budget, hints, tracer, depth) {
        Ok(v) => v, Err(o) => return o,
    };
    let da = match order.digest(ra) {
        Some(d) => *d,
        None => return Outcome::Error(ErrorKind::Unavailable),
    };
    let db = match order.digest(rb) {
        Some(d) => *d,
        None => return Outcome::Error(ErrorKind::Unavailable),
    };
    let equal = da == db;
    let r6 = if equal { 0u64 } else { 1u64 };
    // when both operands are atoms, expose canonical values and inverse hint.
    // otherwise fall back to NounIds (digest equality is the binding constraint).
    let va = order.atom_value(ra).map(|(v, _)| v);
    let vb = order.atom_value(rb).map(|(v, _)| v);
    match (va, vb) {
        (Some(va), Some(vb)) => {
            row.r[4] = va.as_u64();
            row.r[5] = vb.as_u64();
            row.r[6] = r6;
            // inverse hint: (va - vb)^-1 when unequal, 0 when equal
            row.r[7] = if equal { 0 } else { (va - vb).inv().as_u64() };
        }
        _ => {
            // cells / hash nouns — equality binds via digest, not field values.
            // r4/r5 carry NounIds; verifier dispatches on operand type.
            row.r[4] = ra as u64;
            row.r[5] = rb as u64;
            row.r[6] = r6;
            row.r[7] = 0;
        }
    }
    let result = if equal { Goldilocks::ZERO } else { Goldilocks::ONE };
    make_field(order, result, budget)
}

#[cfg(test)]
mod tests {
    use crate::reduce::{reduce, Outcome};
    use crate::call::NullCalls;
    use crate::trace::NoTrace;
    use crate::noun::{Order, Tag};
    use nebu::Goldilocks;

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    /// formula = [9 [[1 a] [1 b]]]
    fn make_eq<const N: usize>(ar: &mut Order<N>, a: u64, b: u64) -> crate::noun::NounId {
        let t9 = ar.atom(g(9), Tag::Field).unwrap();
        let t1 = ar.atom(g(1), Tag::Field).unwrap();
        let va = ar.atom(g(a), Tag::Field).unwrap();
        let vb = ar.atom(g(b), Tag::Field).unwrap();
        let qa = ar.cell(t1, va).unwrap();
        let qb = ar.cell(t1, vb).unwrap();
        let body = ar.cell(qa, qb).unwrap();
        ar.cell(t9, body).unwrap()
    }

    #[test]
    fn eq_equal_returns_zero() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        let formula = make_eq(&mut ar, 5, 5);
        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(0)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn eq_unequal_returns_one() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        let formula = make_eq(&mut ar, 5, 7);
        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(1)),
            o => panic!("{:?}", o),
        }
    }

    /// formula = [9 [[1 (1,2)] [1 (1,2)]]] — quote two structurally-equal cells.
    /// Hash-cons means both sub-quotes resolve to the same NounId, but the test
    /// is meaningful: the eq pattern must work on cells via digest comparison.
    #[test]
    fn eq_cells_structurally_equal_returns_zero() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        let one = ar.atom(g(1), Tag::Field).unwrap();
        let two = ar.atom(g(2), Tag::Field).unwrap();
        let cell_a = ar.cell(one, two).unwrap();
        let cell_b = ar.cell(one, two).unwrap();
        assert_eq!(cell_a, cell_b, "hash-cons collapses identical cells");
        let t1 = ar.atom(g(1), Tag::Field).unwrap();
        let qa = ar.cell(t1, cell_a).unwrap();
        let qb = ar.cell(t1, cell_b).unwrap();
        let body = ar.cell(qa, qb).unwrap();
        let t9 = ar.atom(g(9), Tag::Field).unwrap();
        let formula = ar.cell(t9, body).unwrap();
        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(0)),
            o => panic!("{:?}", o),
        }
    }

    /// Cells with different contents — verify digest path returns 1.
    #[test]
    fn eq_cells_different_returns_one() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        let one = ar.atom(g(1), Tag::Field).unwrap();
        let two = ar.atom(g(2), Tag::Field).unwrap();
        let three = ar.atom(g(3), Tag::Field).unwrap();
        let cell_a = ar.cell(one, two).unwrap();
        let cell_b = ar.cell(one, three).unwrap();
        let t1 = ar.atom(g(1), Tag::Field).unwrap();
        let qa = ar.cell(t1, cell_a).unwrap();
        let qb = ar.cell(t1, cell_b).unwrap();
        let body = ar.cell(qa, qb).unwrap();
        let t9 = ar.atom(g(9), Tag::Field).unwrap();
        let formula = ar.cell(t9, body).unwrap();
        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(1)),
            o => panic!("{:?}", o),
        }
    }
}
