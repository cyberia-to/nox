// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! reduce — dispatch + budget metering + shared helpers
//! pattern implementations live in patterns/

use nebu::Goldilocks;
use crate::noun::{Arena, NounRef, NounInner, Tag};
use crate::hint::HintProvider;
use crate::patterns;

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

pub fn reduce<const N: usize>(
    arena: &mut Arena<N>, subject: NounRef, formula: NounRef, budget: u64, hints: &dyn HintProvider<N>,
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
        0  => patterns::axis::axis(arena, subject, body, budget),
        1  => patterns::quote::quote(body, budget),
        2  => patterns::compose::compose(arena, subject, body, budget, hints),
        3  => patterns::cons::cons(arena, subject, body, budget, hints),
        4  => patterns::branch::branch(arena, subject, body, budget, hints),
        5  => patterns::add::add(arena, subject, body, budget, hints),
        6  => patterns::sub::sub(arena, subject, body, budget, hints),
        7  => patterns::mul::mul(arena, subject, body, budget, hints),
        8  => patterns::inv::inv(arena, subject, body, budget, hints),
        9  => patterns::eq::eq(arena, subject, body, budget, hints),
        10 => patterns::lt::lt(arena, subject, body, budget, hints),
        11 => patterns::xor::xor(arena, subject, body, budget, hints),
        12 => patterns::and::and(arena, subject, body, budget, hints),
        13 => patterns::not::not(arena, subject, body, budget, hints),
        14 => patterns::shl::shl(arena, subject, body, budget, hints),
        15 => patterns::hash::hash(arena, subject, body, budget, hints),
        16 => patterns::hint::hint(arena, subject, body, budget, hints),
        _  => Outcome::Error(ErrorKind::Malformed),
    }
}

// === public helpers used by pattern implementations ===

pub fn cell_pair<const N: usize>(arena: &Arena<N>, r: NounRef) -> Option<(NounRef, NounRef)> {
    match arena.get(r).inner {
        NounInner::Cell { left, right } => Some((left, right)),
        _ => None,
    }
}

pub fn evaluate<const N: usize>(
    arena: &mut Arena<N>, subject: NounRef, formula: NounRef, budget: u64, hints: &dyn HintProvider<N>,
) -> core::result::Result<(NounRef, u64), Outcome> {
    match reduce(arena, subject, formula, budget, hints) {
        Outcome::Ok(r, b) => Ok((r, b)),
        other => Err(other),
    }
}

pub fn evaluate_field<const N: usize>(
    arena: &mut Arena<N>, subject: NounRef, formula: NounRef, budget: u64, hints: &dyn HintProvider<N>,
) -> core::result::Result<(Goldilocks, u64), Outcome> {
    let (result, budget) = evaluate(arena, subject, formula, budget, hints)?;
    match arena.atom_value(result) {
        Some((v, Tag::Field)) | Some((v, Tag::Word)) => Ok((v, budget)),
        _ => Err(Outcome::Error(ErrorKind::TypeError)),
    }
}

pub fn evaluate_word<const N: usize>(
    arena: &mut Arena<N>, subject: NounRef, formula: NounRef, budget: u64, hints: &dyn HintProvider<N>,
) -> core::result::Result<(u64, u64), Outcome> {
    let (result, budget) = evaluate(arena, subject, formula, budget, hints)?;
    match arena.atom_value(result) {
        Some((v, Tag::Word)) => Ok((v.as_u64(), budget)),
        Some((v, Tag::Field)) if v.as_u64() < (1u64 << 32) => Ok((v.as_u64(), budget)),
        _ => Err(Outcome::Error(ErrorKind::TypeError)),
    }
}

pub fn make_field<const N: usize>(arena: &mut Arena<N>, v: Goldilocks, budget: u64) -> Outcome {
    match arena.atom(v, Tag::Field) { Some(r) => Outcome::Ok(r, budget), None => Outcome::Error(ErrorKind::Unavailable) }
}

pub fn make_word<const N: usize>(arena: &mut Arena<N>, v: u64, budget: u64) -> Outcome {
    match arena.atom(Goldilocks::new(v & 0xFFFF_FFFF), Tag::Word) { Some(r) => Outcome::Ok(r, budget), None => Outcome::Error(ErrorKind::Unavailable) }
}

pub fn field_binary_op<const N: usize>(
    arena: &mut Arena<N>, subject: NounRef, body: NounRef, budget: u64, hints: &dyn HintProvider<N>,
    op: fn(Goldilocks, Goldilocks) -> Goldilocks,
) -> Outcome {
    let (a, b) = match cell_pair(arena, body) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (va, budget) = match evaluate_field(arena, subject, a, budget, hints) { Ok(v) => v, Err(o) => return o };
    let (vb, budget) = match evaluate_field(arena, subject, b, budget, hints) { Ok(v) => v, Err(o) => return o };
    make_field(arena, op(va, vb), budget)
}

pub fn word_binary_op<const N: usize>(
    arena: &mut Arena<N>, subject: NounRef, body: NounRef, budget: u64, hints: &dyn HintProvider<N>,
    op: fn(u64, u64) -> u64,
) -> Outcome {
    let (a, b) = match cell_pair(arena, body) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (va, budget) = match evaluate_word(arena, subject, a, budget, hints) { Ok(v) => v, Err(o) => return o };
    let (vb, budget) = match evaluate_word(arena, subject, b, budget, hints) { Ok(v) => v, Err(o) => return o };
    make_word(arena, op(va, vb), budget)
}
