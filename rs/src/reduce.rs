// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! reduce — 16-pattern dispatch + budget metering

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

pub fn reduce<const N: usize>(
    ar: &mut Arena<N>, subj: NounRef, form: NounRef, bud: u64, hp: &dyn HintProvider<N>,
) -> Outcome {
    let (tag_ref, body) = match ar.get(form).inner {
        NounInner::Cell { left, right } => (left, right),
        NounInner::Atom { .. } => return Outcome::Error(ErrorKind::Malformed),
    };
    let tag = match ar.atom_value(tag_ref) {
        Some((v, _)) => v.as_u64(),
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    let c = cost(tag);
    if bud < c { return Outcome::Halt(bud); }
    let b = bud - c;
    match tag {
        0 => p_axis(ar, subj, body, b),
        1 => Outcome::Ok(body, b),
        2 => p_compose(ar, subj, body, b, hp),
        3 => p_cons(ar, subj, body, b, hp),
        4 => p_branch(ar, subj, body, b, hp),
        5 => p_binf(ar, subj, body, b, hp, |a, c| a + c),
        6 => p_binf(ar, subj, body, b, hp, |a, c| a - c),
        7 => p_binf(ar, subj, body, b, hp, |a, c| a * c),
        8 => p_inv(ar, subj, body, b, hp),
        9 => p_eq(ar, subj, body, b, hp),
        10 => p_lt(ar, subj, body, b, hp),
        11 => p_binw(ar, subj, body, b, hp, |a, c| a ^ c),
        12 => p_binw(ar, subj, body, b, hp, |a, c| a & c),
        13 => p_not(ar, subj, body, b, hp),
        14 => p_shl(ar, subj, body, b, hp),
        15 => p_hash(ar, subj, body, b, hp),
        16 => p_hint(ar, subj, body, b, hp),
        _ => Outcome::Error(ErrorKind::Malformed),
    }
}

fn pair<const N: usize>(ar: &Arena<N>, r: NounRef) -> Option<(NounRef, NounRef)> {
    match ar.get(r).inner {
        NounInner::Cell { left, right } => Some((left, right)),
        _ => None,
    }
}

fn evf<const N: usize>(ar: &mut Arena<N>, s: NounRef, f: NounRef, b: u64, hp: &dyn HintProvider<N>) -> Option<(Goldilocks, u64)> {
    match reduce(ar, s, f, b, hp) {
        Outcome::Ok(r, b2) => match ar.atom_value(r) {
            Some((v, Tag::Field)) | Some((v, Tag::Word)) => Some((v, b2)),
            _ => None,
        },
        _ => None,
    }
}

fn evw<const N: usize>(ar: &mut Arena<N>, s: NounRef, f: NounRef, b: u64, hp: &dyn HintProvider<N>) -> Option<(u64, u64)> {
    match reduce(ar, s, f, b, hp) {
        Outcome::Ok(r, b2) => match ar.atom_value(r) {
            Some((v, Tag::Word)) => Some((v.as_u64(), b2)),
            Some((v, Tag::Field)) if v.as_u64() < (1u64 << 32) => Some((v.as_u64(), b2)),
            _ => None,
        },
        _ => None,
    }
}

fn p_axis<const N: usize>(ar: &Arena<N>, s: NounRef, addr_r: NounRef, b: u64) -> Outcome {
    let addr = match ar.atom_value(addr_r) {
        Some((v, _)) => v.as_u64(),
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    if addr <= 1 { return Outcome::Ok(s, b); }
    let bits = 64 - addr.leading_zeros() - 1;
    let mut node = s;
    for i in (0..bits).rev() {
        match ar.get(node).inner {
            NounInner::Cell { left, right } => {
                node = if (addr >> i) & 1 == 1 { right } else { left };
            }
            _ => return Outcome::Error(ErrorKind::AxisError),
        }
    }
    Outcome::Ok(node, b)
}

fn p_compose<const N: usize>(ar: &mut Arena<N>, s: NounRef, body: NounRef, b: u64, hp: &dyn HintProvider<N>) -> Outcome {
    let (a, bf) = match pair(ar, body) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (obj, b) = match reduce(ar, s, a, b, hp) { Outcome::Ok(r, b2) => (r, b2), o => return o };
    let (frm, b) = match reduce(ar, s, bf, b, hp) { Outcome::Ok(r, b2) => (r, b2), o => return o };
    reduce(ar, obj, frm, b, hp)
}

fn p_cons<const N: usize>(ar: &mut Arena<N>, s: NounRef, body: NounRef, b: u64, hp: &dyn HintProvider<N>) -> Outcome {
    let (a, bf) = match pair(ar, body) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (l, b) = match reduce(ar, s, a, b, hp) { Outcome::Ok(r, b2) => (r, b2), o => return o };
    let (r, b) = match reduce(ar, s, bf, b, hp) { Outcome::Ok(r, b2) => (r, b2), o => return o };
    match ar.cell(l, r) { Some(c) => Outcome::Ok(c, b), None => Outcome::Error(ErrorKind::Unavailable) }
}

fn p_branch<const N: usize>(ar: &mut Arena<N>, s: NounRef, body: NounRef, b: u64, hp: &dyn HintProvider<N>) -> Outcome {
    let (tf, rest) = match pair(ar, body) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (yf, nf) = match pair(ar, rest) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (tr, b) = match reduce(ar, s, tf, b, hp) { Outcome::Ok(r, b2) => (r, b2), o => return o };
    let tv = match ar.atom_value(tr) { Some((v, _)) => v.as_u64(), None => return Outcome::Error(ErrorKind::TypeError) };
    reduce(ar, s, if tv == 0 { yf } else { nf }, b, hp)
}

fn p_binf<const N: usize>(ar: &mut Arena<N>, s: NounRef, body: NounRef, b: u64, hp: &dyn HintProvider<N>, op: fn(Goldilocks, Goldilocks) -> Goldilocks) -> Outcome {
    let (a, bf) = match pair(ar, body) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (va, b) = match evf(ar, s, a, b, hp) { Some(v) => v, None => return Outcome::Error(ErrorKind::TypeError) };
    let (vb, b) = match evf(ar, s, bf, b, hp) { Some(v) => v, None => return Outcome::Error(ErrorKind::TypeError) };
    match ar.atom(op(va, vb), Tag::Field) { Some(r) => Outcome::Ok(r, b), None => Outcome::Error(ErrorKind::Unavailable) }
}

fn p_inv<const N: usize>(ar: &mut Arena<N>, s: NounRef, body: NounRef, b: u64, hp: &dyn HintProvider<N>) -> Outcome {
    let (v, b) = match evf(ar, s, body, b, hp) { Some(v) => v, None => return Outcome::Error(ErrorKind::TypeError) };
    if v == Goldilocks::ZERO { return Outcome::Error(ErrorKind::InvZero); }
    match ar.atom(v.inv(), Tag::Field) { Some(r) => Outcome::Ok(r, b), None => Outcome::Error(ErrorKind::Unavailable) }
}

fn p_eq<const N: usize>(ar: &mut Arena<N>, s: NounRef, body: NounRef, b: u64, hp: &dyn HintProvider<N>) -> Outcome {
    let (a, bf) = match pair(ar, body) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (va, b) = match evf(ar, s, a, b, hp) { Some(v) => v, None => return Outcome::Error(ErrorKind::TypeError) };
    let (vb, b) = match evf(ar, s, bf, b, hp) { Some(v) => v, None => return Outcome::Error(ErrorKind::TypeError) };
    let r = if va == vb { Goldilocks::ZERO } else { Goldilocks::ONE };
    match ar.atom(r, Tag::Field) { Some(r) => Outcome::Ok(r, b), None => Outcome::Error(ErrorKind::Unavailable) }
}

fn p_lt<const N: usize>(ar: &mut Arena<N>, s: NounRef, body: NounRef, b: u64, hp: &dyn HintProvider<N>) -> Outcome {
    let (a, bf) = match pair(ar, body) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (va, b) = match evf(ar, s, a, b, hp) { Some(v) => v, None => return Outcome::Error(ErrorKind::TypeError) };
    let (vb, b) = match evf(ar, s, bf, b, hp) { Some(v) => v, None => return Outcome::Error(ErrorKind::TypeError) };
    let r = if va.as_u64() < vb.as_u64() { Goldilocks::ZERO } else { Goldilocks::ONE };
    match ar.atom(r, Tag::Field) { Some(r) => Outcome::Ok(r, b), None => Outcome::Error(ErrorKind::Unavailable) }
}

fn p_binw<const N: usize>(ar: &mut Arena<N>, s: NounRef, body: NounRef, b: u64, hp: &dyn HintProvider<N>, op: fn(u64, u64) -> u64) -> Outcome {
    let (a, bf) = match pair(ar, body) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (va, b) = match evw(ar, s, a, b, hp) { Some(v) => v, None => return Outcome::Error(ErrorKind::TypeError) };
    let (vb, b) = match evw(ar, s, bf, b, hp) { Some(v) => v, None => return Outcome::Error(ErrorKind::TypeError) };
    match ar.atom(Goldilocks::new(op(va, vb) & 0xFFFF_FFFF), Tag::Word) { Some(r) => Outcome::Ok(r, b), None => Outcome::Error(ErrorKind::Unavailable) }
}

fn p_not<const N: usize>(ar: &mut Arena<N>, s: NounRef, body: NounRef, b: u64, hp: &dyn HintProvider<N>) -> Outcome {
    let (v, b) = match evw(ar, s, body, b, hp) { Some(v) => v, None => return Outcome::Error(ErrorKind::TypeError) };
    match ar.atom(Goldilocks::new((!v) & 0xFFFF_FFFF), Tag::Word) { Some(r) => Outcome::Ok(r, b), None => Outcome::Error(ErrorKind::Unavailable) }
}

fn p_shl<const N: usize>(ar: &mut Arena<N>, s: NounRef, body: NounRef, b: u64, hp: &dyn HintProvider<N>) -> Outcome {
    let (a, n) = match pair(ar, body) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (va, b) = match evw(ar, s, a, b, hp) { Some(v) => v, None => return Outcome::Error(ErrorKind::TypeError) };
    let (vn, b) = match evw(ar, s, n, b, hp) { Some(v) => v, None => return Outcome::Error(ErrorKind::TypeError) };
    let r = if vn >= 32 { 0 } else { (va << vn) & 0xFFFF_FFFF };
    match ar.atom(Goldilocks::new(r), Tag::Word) { Some(r) => Outcome::Ok(r, b), None => Outcome::Error(ErrorKind::Unavailable) }
}

fn p_hash<const N: usize>(ar: &mut Arena<N>, s: NounRef, body: NounRef, b: u64, hp: &dyn HintProvider<N>) -> Outcome {
    let (input, b) = match reduce(ar, s, body, b, hp) { Outcome::Ok(r, b2) => (r, b2), o => return o };
    let h = ar.hash(input);
    match ar.atom(h[0], Tag::Field) { Some(r) => Outcome::Ok(r, b), None => Outcome::Error(ErrorKind::Unavailable) }
}

fn p_hint<const N: usize>(ar: &mut Arena<N>, s: NounRef, body: NounRef, b: u64, hp: &dyn HintProvider<N>) -> Outcome {
    let (tf, cf) = match pair(ar, body) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (tr, b) = match reduce(ar, s, tf, b, hp) { Outcome::Ok(r, b2) => (r, b2), o => return o };
    let tag = match ar.atom_value(tr) { Some((v, _)) => v, None => return Outcome::Error(ErrorKind::TypeError) };
    let w = match hp.provide(ar, tag, s) { Some(w) => w, None => return Outcome::Halt(b) };
    let ws = match ar.cell(w, s) { Some(c) => c, None => return Outcome::Error(ErrorKind::Unavailable) };
    match reduce(ar, ws, cf, b, hp) {
        Outcome::Ok(r, b2) => Outcome::Ok(r, b2),
        Outcome::Error(_) => Outcome::Error(ErrorKind::HintRejected),
        o => o,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hint::NullHints;

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    fn binop<const N: usize>(ar: &mut Arena<N>, tag: u64, a: NounRef, b: NounRef) -> NounRef {
        let t = ar.atom(g(tag), Tag::Field).unwrap();
        let body = ar.cell(a, b).unwrap();
        ar.cell(t, body).unwrap()
    }

    fn unop<const N: usize>(ar: &mut Arena<N>, tag: u64, a: NounRef) -> NounRef {
        let t = ar.atom(g(tag), Tag::Field).unwrap();
        ar.cell(t, a).unwrap()
    }

    fn axis<const N: usize>(ar: &mut Arena<N>, n: u64) -> NounRef {
        let a = ar.atom(g(n), Tag::Field).unwrap();
        unop(ar, 0, a)
    }

    fn quote<const N: usize>(ar: &mut Arena<N>, v: NounRef) -> NounRef {
        unop(ar, 1, v)
    }

    fn make_pair<const N: usize>(ar: &mut Arena<N>, a: u64, b: u64) -> NounRef {
        let la = ar.atom(g(a), Tag::Field).unwrap();
        let lb = ar.atom(g(b), Tag::Field).unwrap();
        ar.cell(la, lb).unwrap()
    }

    fn make_binop<const N: usize>(ar: &mut Arena<N>, tag: u64, n1: u64, n2: u64) -> NounRef {
        let ax1 = axis(ar, n1);
        let ax2 = axis(ar, n2);
        binop(ar, tag, ax1, ax2)
    }

    fn make_quote_atom<const N: usize>(ar: &mut Arena<N>, v: u64) -> NounRef {
        let a = ar.atom(g(v), Tag::Field).unwrap();
        quote(ar, a)
    }

    #[test]
    fn test_quote() {
        let mut ar = Arena::<1024>::new();
        let v = ar.atom(g(42), Tag::Field).unwrap();
        let f = quote(&mut ar, v);
        let s = ar.atom(g(0), Tag::Field).unwrap();
        match reduce(&mut ar, s, f, 100, &NullHints) {
            Outcome::Ok(r, b) => { assert_eq!(ar.atom_value(r).unwrap().0, g(42)); assert_eq!(b, 99); }
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn test_add() {
        let mut ar = Arena::<1024>::new();
        let s = make_pair(&mut ar, 1, 2);
        let f = make_binop(&mut ar, 5, 2, 3);
        match reduce(&mut ar, s, f, 100, &NullHints) {
            Outcome::Ok(r, b) => { assert_eq!(ar.atom_value(r).unwrap().0, g(3)); assert_eq!(b, 97); }
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn test_halt() {
        let mut ar = Arena::<1024>::new();
        let v = ar.atom(g(42), Tag::Field).unwrap();
        let f = quote(&mut ar, v);
        let s = ar.atom(g(0), Tag::Field).unwrap();
        match reduce(&mut ar, s, f, 0, &NullHints) {
            Outcome::Halt(0) => {}
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn test_axis() {
        let mut ar = Arena::<1024>::new();
        let s = make_pair(&mut ar, 10, 20);
        let ax2 = axis(&mut ar, 2);
        match reduce(&mut ar, s, ax2, 100, &NullHints) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(10)),
            o => panic!("{:?}", o),
        }
        let ax3 = axis(&mut ar, 3);
        match reduce(&mut ar, s, ax3, 100, &NullHints) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(20)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn test_mul() {
        let mut ar = Arena::<1024>::new();
        let s = make_pair(&mut ar, 3, 7);
        let f = make_binop(&mut ar, 7, 2, 3);
        match reduce(&mut ar, s, f, 100, &NullHints) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(21)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn test_inv_zero() {
        let mut ar = Arena::<1024>::new();
        let s = ar.atom(g(0), Tag::Field).unwrap();
        let qz = make_quote_atom(&mut ar, 0);
        let f = unop(&mut ar, 8, qz);
        match reduce(&mut ar, s, f, 200, &NullHints) {
            Outcome::Error(ErrorKind::InvZero) => {}
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn test_branch() {
        let mut ar = Arena::<1024>::new();
        let s = ar.atom(g(0), Tag::Field).unwrap();
        let test = make_quote_atom(&mut ar, 0);
        let yes = make_quote_atom(&mut ar, 42);
        let no = make_quote_atom(&mut ar, 99);
        let branches = ar.cell(yes, no).unwrap();
        let body = ar.cell(test, branches).unwrap();
        let tag4 = ar.atom(g(4), Tag::Field).unwrap();
        let f = ar.cell(tag4, body).unwrap();
        match reduce(&mut ar, s, f, 100, &NullHints) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(42)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn test_eq() {
        let mut ar = Arena::<1024>::new();
        let s = make_pair(&mut ar, 5, 5);
        let f = make_binop(&mut ar, 9, 2, 3);
        match reduce(&mut ar, s, f, 100, &NullHints) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(0)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn test_cons() {
        let mut ar = Arena::<1024>::new();
        let s = ar.atom(g(0), Tag::Field).unwrap();
        let a = make_quote_atom(&mut ar, 10);
        let b = make_quote_atom(&mut ar, 20);
        let f = binop(&mut ar, 3, a, b);
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
}
