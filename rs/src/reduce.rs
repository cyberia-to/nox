// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! reduce — 16-pattern dispatch + budget metering
//!
//! reduce(object, formula, budget) → Outcome

use nebu::Goldilocks;
use crate::noun::{Arena, NounRef, NounInner, Tag};
use crate::hint::HintProvider;

#[derive(Debug)]
pub enum Outcome {
    Ok(NounRef, u64),
    Halt(u64),
    Error(ErrorKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    TypeError,
    AxisError,
    InvZero,
    Unavailable,
    Malformed,
    HintRejected,
}

fn cost(tag: u64) -> u64 {
    match tag { 8 => 64, 15 => 300, _ => 1 }
}

/// reduce: apply formula to object under budget
pub fn reduce<const N: usize>(
    arena: &mut Arena<N>,
    subject: NounRef,
    formula: NounRef,
    budget: u64,
    hints: &dyn HintProvider<N>,
) -> Outcome {
    let (tag_ref, body) = match arena.get(formula).inner {
        NounInner::Cell { left, right } => (left, right),
        NounInner::Atom { .. } => return Outcome::Error(ErrorKind::Malformed),
    };
    let tag = match arena.atom_value(tag_ref) {
        Some((v, _)) => v.as_u64(),
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    let c = cost(tag);
    if budget < c { return Outcome::Halt(budget); }
    let budget = budget - c;
    match tag {
        0  => axis(arena, subject, body, budget),
        1  => quote(body, budget),
        2  => compose(arena, subject, body, budget, hints),
        3  => cons(arena, subject, body, budget, hints),
        4  => branch(arena, subject, body, budget, hints),
        5  => add(arena, subject, body, budget, hints),
        6  => sub(arena, subject, body, budget, hints),
        7  => mul(arena, subject, body, budget, hints),
        8  => inv(arena, subject, body, budget, hints),
        9  => eq(arena, subject, body, budget, hints),
        10 => lt(arena, subject, body, budget, hints),
        11 => xor(arena, subject, body, budget, hints),
        12 => and(arena, subject, body, budget, hints),
        13 => not(arena, subject, body, budget, hints),
        14 => shl(arena, subject, body, budget, hints),
        15 => hash(arena, subject, body, budget, hints),
        16 => hint(arena, subject, body, budget, hints),
        _  => Outcome::Error(ErrorKind::Malformed),
    }
}

// === helpers ===

/// get (left, right) from a cell body
fn cell_pair<const N: usize>(arena: &Arena<N>, r: NounRef) -> Option<(NounRef, NounRef)> {
    match arena.get(r).inner {
        NounInner::Cell { left, right } => Some((left, right)),
        _ => None,
    }
}

/// evaluate sub-expression, propagate Halt/Error correctly (C-6 fix)
fn evaluate<const N: usize>(
    arena: &mut Arena<N>, subject: NounRef, formula: NounRef, budget: u64, hints: &dyn HintProvider<N>,
) -> core::result::Result<(NounRef, u64), Outcome> {
    match reduce(arena, subject, formula, budget, hints) {
        Outcome::Ok(r, b) => Ok((r, b)),
        other => Err(other), // Halt and Error propagate correctly
    }
}

/// evaluate to field value, propagate Halt/Error correctly
fn evaluate_field<const N: usize>(
    arena: &mut Arena<N>, subject: NounRef, formula: NounRef, budget: u64, hints: &dyn HintProvider<N>,
) -> core::result::Result<(Goldilocks, u64), Outcome> {
    let (result, budget) = evaluate(arena, subject, formula, budget, hints)?;
    match arena.atom_value(result) {
        Some((v, Tag::Field)) => Ok((v, budget)),
        Some((v, Tag::Word)) => Ok((v, budget)), // word coerces to field
        _ => Err(Outcome::Error(ErrorKind::TypeError)),
    }
}

/// evaluate to word value, propagate Halt/Error correctly
fn evaluate_word<const N: usize>(
    arena: &mut Arena<N>, subject: NounRef, formula: NounRef, budget: u64, hints: &dyn HintProvider<N>,
) -> core::result::Result<(u64, u64), Outcome> {
    let (result, budget) = evaluate(arena, subject, formula, budget, hints)?;
    match arena.atom_value(result) {
        Some((v, Tag::Word)) => Ok((v.as_u64(), budget)),
        Some((v, Tag::Field)) if v.as_u64() < (1u64 << 32) => Ok((v.as_u64(), budget)),
        _ => Err(Outcome::Error(ErrorKind::TypeError)),
    }
}

/// create field atom or return Unavailable
fn make_field<const N: usize>(arena: &mut Arena<N>, v: Goldilocks, budget: u64) -> Outcome {
    match arena.atom(v, Tag::Field) {
        Some(r) => Outcome::Ok(r, budget),
        None => Outcome::Error(ErrorKind::Unavailable),
    }
}

/// create word atom or return Unavailable
fn make_word<const N: usize>(arena: &mut Arena<N>, v: u64, budget: u64) -> Outcome {
    match arena.atom(Goldilocks::new(v & 0xFFFF_FFFF), Tag::Word) {
        Some(r) => Outcome::Ok(r, budget),
        None => Outcome::Error(ErrorKind::Unavailable),
    }
}

// === pattern 0: axis ===

fn axis<const N: usize>(arena: &mut Arena<N>, subject: NounRef, addr_ref: NounRef, budget: u64) -> Outcome {
    let addr = match arena.atom_value(addr_ref) {
        Some((v, _)) => v.as_u64(),
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    match addr {
        0 => {
            // axis(s, 0) = H(s) — hash introspection (C-3 fix)
            let digest = *arena.digest(subject);
            match arena.hash_noun(&digest) {
                Some(r) => Outcome::Ok(r, budget),
                None => Outcome::Error(ErrorKind::Unavailable),
            }
        }
        1 => Outcome::Ok(subject, budget),
        _ => {
            let bits = 64 - addr.leading_zeros() - 1;
            let mut node = subject;
            for i in (0..bits).rev() {
                match arena.get(node).inner {
                    NounInner::Cell { left, right } => {
                        node = if (addr >> i) & 1 == 1 { right } else { left };
                    }
                    _ => return Outcome::Error(ErrorKind::AxisError),
                }
            }
            Outcome::Ok(node, budget)
        }
    }
}

// === pattern 1: quote ===

fn quote(body: NounRef, budget: u64) -> Outcome {
    Outcome::Ok(body, budget)
}

// === pattern 2: compose ===

fn compose<const N: usize>(
    arena: &mut Arena<N>, subject: NounRef, body: NounRef, budget: u64, hints: &dyn HintProvider<N>,
) -> Outcome {
    let (a, b) = match cell_pair(arena, body) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (obj, budget) = match evaluate(arena, subject, a, budget, hints) { Ok(v) => v, Err(o) => return o };
    let (frm, budget) = match evaluate(arena, subject, b, budget, hints) { Ok(v) => v, Err(o) => return o };
    reduce(arena, obj, frm, budget, hints)
}

// === pattern 3: cons ===

fn cons<const N: usize>(
    arena: &mut Arena<N>, subject: NounRef, body: NounRef, budget: u64, hints: &dyn HintProvider<N>,
) -> Outcome {
    let (a, b) = match cell_pair(arena, body) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (left, budget) = match evaluate(arena, subject, a, budget, hints) { Ok(v) => v, Err(o) => return o };
    let (right, budget) = match evaluate(arena, subject, b, budget, hints) { Ok(v) => v, Err(o) => return o };
    match arena.cell(left, right) {
        Some(c) => Outcome::Ok(c, budget),
        None => Outcome::Error(ErrorKind::Unavailable),
    }
}

// === pattern 4: branch ===

fn branch<const N: usize>(
    arena: &mut Arena<N>, subject: NounRef, body: NounRef, budget: u64, hints: &dyn HintProvider<N>,
) -> Outcome {
    let (test_formula, rest) = match cell_pair(arena, body) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (yes_formula, no_formula) = match cell_pair(arena, rest) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (test_result, budget) = match evaluate(arena, subject, test_formula, budget, hints) { Ok(v) => v, Err(o) => return o };
    let test_value = match arena.atom_value(test_result) {
        Some((v, _)) => v.as_u64(),
        None => return Outcome::Error(ErrorKind::TypeError),
    };
    let chosen = if test_value == 0 { yes_formula } else { no_formula };
    reduce(arena, subject, chosen, budget, hints)
}

// === patterns 5-7: field arithmetic ===

fn field_binary_op<const N: usize>(
    arena: &mut Arena<N>, subject: NounRef, body: NounRef, budget: u64, hints: &dyn HintProvider<N>,
    op: fn(Goldilocks, Goldilocks) -> Goldilocks,
) -> Outcome {
    let (a, b) = match cell_pair(arena, body) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (va, budget) = match evaluate_field(arena, subject, a, budget, hints) { Ok(v) => v, Err(o) => return o };
    let (vb, budget) = match evaluate_field(arena, subject, b, budget, hints) { Ok(v) => v, Err(o) => return o };
    make_field(arena, op(va, vb), budget)
}

fn add<const N: usize>(ar: &mut Arena<N>, s: NounRef, b: NounRef, bg: u64, h: &dyn HintProvider<N>) -> Outcome {
    field_binary_op(ar, s, b, bg, h, |a, b| a + b)
}

fn sub<const N: usize>(ar: &mut Arena<N>, s: NounRef, b: NounRef, bg: u64, h: &dyn HintProvider<N>) -> Outcome {
    field_binary_op(ar, s, b, bg, h, |a, b| a - b)
}

fn mul<const N: usize>(ar: &mut Arena<N>, s: NounRef, b: NounRef, bg: u64, h: &dyn HintProvider<N>) -> Outcome {
    field_binary_op(ar, s, b, bg, h, |a, b| a * b)
}

// === pattern 8: inv ===

fn inv<const N: usize>(
    arena: &mut Arena<N>, subject: NounRef, body: NounRef, budget: u64, hints: &dyn HintProvider<N>,
) -> Outcome {
    let (v, budget) = match evaluate_field(arena, subject, body, budget, hints) { Ok(v) => v, Err(o) => return o };
    if v == Goldilocks::ZERO { return Outcome::Error(ErrorKind::InvZero); }
    make_field(arena, v.inv(), budget)
}

// === pattern 9: eq ===

fn eq<const N: usize>(
    arena: &mut Arena<N>, subject: NounRef, body: NounRef, budget: u64, hints: &dyn HintProvider<N>,
) -> Outcome {
    let (a, b) = match cell_pair(arena, body) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (ra, budget) = match evaluate(arena, subject, a, budget, hints) { Ok(v) => v, Err(o) => return o };
    let (rb, budget) = match evaluate(arena, subject, b, budget, hints) { Ok(v) => v, Err(o) => return o };
    // eq compares noun identity (hash), not just field values — works for ALL noun types
    let equal = arena.digest(ra) == arena.digest(rb);
    let result = if equal { Goldilocks::ZERO } else { Goldilocks::ONE };
    make_field(arena, result, budget)
}

// === pattern 10: lt ===

fn lt<const N: usize>(
    arena: &mut Arena<N>, subject: NounRef, body: NounRef, budget: u64, hints: &dyn HintProvider<N>,
) -> Outcome {
    let (a, b) = match cell_pair(arena, body) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (va, budget) = match evaluate_field(arena, subject, a, budget, hints) { Ok(v) => v, Err(o) => return o };
    let (vb, budget) = match evaluate_field(arena, subject, b, budget, hints) { Ok(v) => v, Err(o) => return o };
    let result = if va.as_u64() < vb.as_u64() { Goldilocks::ZERO } else { Goldilocks::ONE };
    make_field(arena, result, budget)
}

// === patterns 11-14: word operations ===

fn word_binary_op<const N: usize>(
    arena: &mut Arena<N>, subject: NounRef, body: NounRef, budget: u64, hints: &dyn HintProvider<N>,
    op: fn(u64, u64) -> u64,
) -> Outcome {
    let (a, b) = match cell_pair(arena, body) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (va, budget) = match evaluate_word(arena, subject, a, budget, hints) { Ok(v) => v, Err(o) => return o };
    let (vb, budget) = match evaluate_word(arena, subject, b, budget, hints) { Ok(v) => v, Err(o) => return o };
    make_word(arena, op(va, vb), budget)
}

fn xor<const N: usize>(ar: &mut Arena<N>, s: NounRef, b: NounRef, bg: u64, h: &dyn HintProvider<N>) -> Outcome {
    word_binary_op(ar, s, b, bg, h, |a, b| a ^ b)
}

fn and<const N: usize>(ar: &mut Arena<N>, s: NounRef, b: NounRef, bg: u64, h: &dyn HintProvider<N>) -> Outcome {
    word_binary_op(ar, s, b, bg, h, |a, b| a & b)
}

fn not<const N: usize>(
    arena: &mut Arena<N>, subject: NounRef, body: NounRef, budget: u64, hints: &dyn HintProvider<N>,
) -> Outcome {
    let (v, budget) = match evaluate_word(arena, subject, body, budget, hints) { Ok(v) => v, Err(o) => return o };
    make_word(arena, (!v) & 0xFFFF_FFFF, budget)
}

fn shl<const N: usize>(
    arena: &mut Arena<N>, subject: NounRef, body: NounRef, budget: u64, hints: &dyn HintProvider<N>,
) -> Outcome {
    let (a, n) = match cell_pair(arena, body) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (va, budget) = match evaluate_word(arena, subject, a, budget, hints) { Ok(v) => v, Err(o) => return o };
    let (vn, budget) = match evaluate_word(arena, subject, n, budget, hints) { Ok(v) => v, Err(o) => return o };
    let result = if vn >= 32 { 0 } else { (va << vn) & 0xFFFF_FFFF };
    make_word(arena, result, budget)
}

// === pattern 15: hash ===

fn hash<const N: usize>(
    arena: &mut Arena<N>, subject: NounRef, body: NounRef, budget: u64, hints: &dyn HintProvider<N>,
) -> Outcome {
    let (input, budget) = match evaluate(arena, subject, body, budget, hints) { Ok(v) => v, Err(o) => return o };
    // return H(input) as a hash noun: cell(cell(h0,h1), cell(h2,h3)) (C-4 fix)
    let digest = *arena.digest(input);
    match arena.hash_noun(&digest) {
        Some(r) => Outcome::Ok(r, budget),
        None => Outcome::Error(ErrorKind::Unavailable),
    }
}

// === pattern 16: hint ===

fn hint<const N: usize>(
    arena: &mut Arena<N>, subject: NounRef, body: NounRef, budget: u64, hints: &dyn HintProvider<N>,
) -> Outcome {
    let (tag_formula, check_formula) = match cell_pair(arena, body) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    // step 1: evaluate tag
    let (tag_result, budget) = match evaluate(arena, subject, tag_formula, budget, hints) {
        Ok(v) => v, Err(o) => return o,
    };
    let tag_value = match arena.atom_value(tag_result) {
        Some((v, _)) => v,
        None => return Outcome::Error(ErrorKind::TypeError),
    };
    // step 2: ask prover for witness
    let witness = match hints.provide(arena, tag_value, subject) {
        Some(w) => w,
        None => return Outcome::Halt(budget), // no hint = clean halt
    };
    // step 3: validate — reduce(check_formula, [witness subject])
    let witness_subject = match arena.cell(witness, subject) {
        Some(c) => c,
        None => return Outcome::Error(ErrorKind::Unavailable),
    };
    // check must produce field zero; result = witness (C-7 fix)
    match reduce(arena, witness_subject, check_formula, budget, hints) {
        Outcome::Ok(check_result, budget) => {
            match arena.atom_value(check_result) {
                Some((v, _)) if v == Goldilocks::ZERO => Outcome::Ok(witness, budget),
                _ => Outcome::Error(ErrorKind::HintRejected),
            }
        }
        Outcome::Error(_) => Outcome::Error(ErrorKind::HintRejected),
        Outcome::Halt(b) => Outcome::Halt(b),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hint::NullHints;

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    fn make_formula<const N: usize>(arena: &mut Arena<N>, tag: u64, body: NounRef) -> NounRef {
        let t = arena.atom(g(tag), Tag::Field).unwrap();
        arena.cell(t, body).unwrap()
    }

    fn make_binary_formula<const N: usize>(arena: &mut Arena<N>, tag: u64, a: NounRef, b: NounRef) -> NounRef {
        let body = arena.cell(a, b).unwrap();
        make_formula(arena, tag, body)
    }

    fn make_axis<const N: usize>(arena: &mut Arena<N>, n: u64) -> NounRef {
        let addr = arena.atom(g(n), Tag::Field).unwrap();
        make_formula(arena, 0, addr)
    }

    fn make_quote<const N: usize>(arena: &mut Arena<N>, v: NounRef) -> NounRef {
        make_formula(arena, 1, v)
    }

    fn make_quote_value<const N: usize>(arena: &mut Arena<N>, v: u64) -> NounRef {
        let a = arena.atom(g(v), Tag::Field).unwrap();
        make_quote(arena, a)
    }

    fn make_pair<const N: usize>(arena: &mut Arena<N>, a: u64, b: u64) -> NounRef {
        let la = arena.atom(g(a), Tag::Field).unwrap();
        let lb = arena.atom(g(b), Tag::Field).unwrap();
        arena.cell(la, lb).unwrap()
    }

    fn make_arith<const N: usize>(arena: &mut Arena<N>, tag: u64, ax_a: u64, ax_b: u64) -> NounRef {
        let a = make_axis(arena, ax_a);
        let b = make_axis(arena, ax_b);
        make_binary_formula(arena, tag, a, b)
    }

    #[test]
    fn quote_returns_body() {
        let mut ar = Arena::<1024>::new();
        let v = ar.atom(g(42), Tag::Field).unwrap();
        let f = make_quote(&mut ar, v);
        let s = ar.atom(g(0), Tag::Field).unwrap();
        match reduce(&mut ar, s, f, 100, &NullHints) {
            Outcome::Ok(r, b) => { assert_eq!(ar.atom_value(r).unwrap().0, g(42)); assert_eq!(b, 99); }
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn add_two_values() {
        let mut ar = Arena::<1024>::new();
        let s = make_pair(&mut ar, 1, 2);
        let f = make_arith(&mut ar, 5, 2, 3); // add(head, tail)
        match reduce(&mut ar, s, f, 100, &NullHints) {
            Outcome::Ok(r, b) => { assert_eq!(ar.atom_value(r).unwrap().0, g(3)); assert_eq!(b, 97); }
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn mul_two_values() {
        let mut ar = Arena::<1024>::new();
        let s = make_pair(&mut ar, 3, 7);
        let f = make_arith(&mut ar, 7, 2, 3);
        match reduce(&mut ar, s, f, 100, &NullHints) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(21)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn budget_exhaustion_halts() {
        let mut ar = Arena::<1024>::new();
        let v = ar.atom(g(42), Tag::Field).unwrap();
        let f = make_quote(&mut ar, v);
        let s = ar.atom(g(0), Tag::Field).unwrap();
        match reduce(&mut ar, s, f, 0, &NullHints) {
            Outcome::Halt(0) => {}
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn axis_navigates_tree() {
        let mut ar = Arena::<1024>::new();
        let s = make_pair(&mut ar, 10, 20);
        let ax2 = make_axis(&mut ar, 2);
        match reduce(&mut ar, s, ax2, 100, &NullHints) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(10)),
            o => panic!("{:?}", o),
        }
        let ax3 = make_axis(&mut ar, 3);
        match reduce(&mut ar, s, ax3, 100, &NullHints) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(20)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn axis_zero_returns_hash_noun() {
        let mut ar = Arena::<1024>::new();
        let s = ar.atom(g(42), Tag::Field).unwrap();
        let ax0 = make_axis(&mut ar, 0);
        match reduce(&mut ar, s, ax0, 100, &NullHints) {
            Outcome::Ok(r, _) => {
                // result is a cell (hash noun), not an atom
                assert!(ar.is_cell(r));
                // can read back as digest
                let d = ar.read_hash_noun(r).unwrap();
                assert_eq!(d, *ar.digest(s));
            }
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn branch_takes_yes_on_zero() {
        let mut ar = Arena::<1024>::new();
        let s = ar.atom(g(0), Tag::Field).unwrap();
        let test = make_quote_value(&mut ar, 0);
        let yes = make_quote_value(&mut ar, 42);
        let no = make_quote_value(&mut ar, 99);
        let branches = ar.cell(yes, no).unwrap();
        let body = ar.cell(test, branches).unwrap();
        let f = make_formula(&mut ar, 4, body);
        match reduce(&mut ar, s, f, 100, &NullHints) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(42)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn branch_takes_no_on_nonzero() {
        let mut ar = Arena::<1024>::new();
        let s = ar.atom(g(0), Tag::Field).unwrap();
        let test = make_quote_value(&mut ar, 1);
        let yes = make_quote_value(&mut ar, 42);
        let no = make_quote_value(&mut ar, 99);
        let branches = ar.cell(yes, no).unwrap();
        let body = ar.cell(test, branches).unwrap();
        let f = make_formula(&mut ar, 4, body);
        match reduce(&mut ar, s, f, 100, &NullHints) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(99)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn eq_returns_zero_when_equal() {
        let mut ar = Arena::<1024>::new();
        let s = make_pair(&mut ar, 5, 5);
        let f = make_arith(&mut ar, 9, 2, 3);
        match reduce(&mut ar, s, f, 100, &NullHints) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(0)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn eq_returns_one_when_not_equal() {
        let mut ar = Arena::<1024>::new();
        let s = make_pair(&mut ar, 5, 6);
        let f = make_arith(&mut ar, 9, 2, 3);
        match reduce(&mut ar, s, f, 100, &NullHints) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(1)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn inv_zero_errors() {
        let mut ar = Arena::<1024>::new();
        let s = ar.atom(g(0), Tag::Field).unwrap();
        let qz = make_quote_value(&mut ar, 0);
        let f = make_formula(&mut ar, 8, qz);
        match reduce(&mut ar, s, f, 200, &NullHints) {
            Outcome::Error(ErrorKind::InvZero) => {}
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn cons_builds_cell() {
        let mut ar = Arena::<1024>::new();
        let s = ar.atom(g(0), Tag::Field).unwrap();
        let a = make_quote_value(&mut ar, 10);
        let b = make_quote_value(&mut ar, 20);
        let f = make_binary_formula(&mut ar, 3, a, b);
        match reduce(&mut ar, s, f, 100, &NullHints) {
            Outcome::Ok(r, _) => {
                assert!(ar.is_cell(r));
                let h = ar.head(r).unwrap();
                let t = ar.tail(r).unwrap();
                assert_eq!(ar.atom_value(h).unwrap().0, g(10));
                assert_eq!(ar.atom_value(t).unwrap().0, g(20));
            }
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn hash_returns_hash_noun() {
        let mut ar = Arena::<1024>::new();
        let s = ar.atom(g(0), Tag::Field).unwrap();
        let input = make_quote_value(&mut ar, 42);
        let f = make_formula(&mut ar, 15, input);
        match reduce(&mut ar, s, f, 500, &NullHints) {
            Outcome::Ok(r, _) => {
                // result is a hash noun (cell of cells), not a single atom
                assert!(ar.is_cell(r));
                let d = ar.read_hash_noun(r);
                assert!(d.is_some());
            }
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn halt_propagates_through_arithmetic() {
        // C-6 fix: budget exhaustion in sub-expression → Halt, not TypeError
        let mut ar = Arena::<1024>::new();
        let s = make_pair(&mut ar, 1, 2);
        let f = make_arith(&mut ar, 5, 2, 3); // add(axis2, axis3) = 3 reduce calls
        // budget = 2: add(1) + axis(1) = 2, then axis needs 1 more but budget = 0
        match reduce(&mut ar, s, f, 2, &NullHints) {
            Outcome::Halt(_) => {} // correct: budget exhausted, propagated as Halt
            Outcome::Error(ErrorKind::TypeError) => panic!("C-6 bug: Halt converted to TypeError"),
            o => panic!("{:?}", o),
        }
    }
}
