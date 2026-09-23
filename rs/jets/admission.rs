//! Bounded, allocation-free admission shared by CPU and accelerated backends.
use super::registry::{DigestKey, digest_key};
use crate::data::{Order, Reduction};
use crate::reduce::{ErrorKind, pair_children};

pub const MAX_ACCELERATED_DEPTH: u32 = 16;
pub const MAX_ACCELERATED_WORDS: usize = 1 << MAX_ACCELERATED_DEPTH;

pub(crate) struct Input {
    pub depth: u32,
    pub words: usize,
    pub tree: Order,
    pub parameter: Order,
    pub self_reference: Order,
}
impl Input {
    pub fn ntt_cost(&self) -> u64 {
        u64::from(self.depth) * self.words as u64
    }
    pub fn poly_cost(&self) -> u64 {
        self.words as u64
    }
}
pub(crate) fn metadata<const N: usize>(
    r: &Reduction<N>,
    object: Order,
) -> Result<Input, ErrorKind> {
    let (lhs, rhs) = pair_children(r, object).ok_or(ErrorKind::Malformed)?;
    let (exponent, self_reference) = pair_children(r, lhs).ok_or(ErrorKind::Malformed)?;
    let (tree, parameter) = pair_children(r, rhs).ok_or(ErrorKind::Malformed)?;
    let raw = r.atom_value(exponent).ok_or(ErrorKind::TypeError)?.as_u64();
    if raw > u64::from(MAX_ACCELERATED_DEPTH) {
        return Err(ErrorKind::Unavailable);
    }
    let depth = u32::try_from(raw).map_err(|_| ErrorKind::Unavailable)?;
    let words = 1usize
        .checked_shl(depth)
        .filter(|n| *n <= MAX_ACCELERATED_WORDS)
        .ok_or(ErrorKind::Unavailable)?;
    Ok(Input {
        depth,
        words,
        tree,
        parameter,
        self_reference,
    })
}
/// Exactly balanced depth, with a maximum fixed expansion and recursion depth.
pub(crate) fn balanced<const N: usize>(r: &Reduction<N>, tree: Order, depth: u32) -> bool {
    if depth > MAX_ACCELERATED_DEPTH {
        return false;
    }
    if depth == 0 {
        return r.atom_value(tree).is_some();
    }
    pair_children(r, tree)
        .is_some_and(|(l, h)| balanced(r, l, depth - 1) && balanced(r, h, depth - 1))
}
pub(crate) fn point<const N: usize>(r: &Reduction<N>, mut p: Order, depth: u32) -> bool {
    for _ in 0..depth {
        let Some((head, tail)) = pair_children(r, p) else {
            return false;
        };
        if r.atom_value(head).is_none() {
            return false;
        }
        p = tail;
    }
    true
}
pub(crate) fn ntt_admitted<const N: usize>(
    r: &Reduction<N>,
    object: Order,
    _key: &DigestKey,
    budget: u64,
) -> bool {
    let Ok(i) = metadata(r, object) else {
        return false;
    };
    // The existing pure identity is only one butterfly, not recursive Cooley-Tukey.
    if i.depth > 1 || budget < i.ntt_cost() {
        return false;
    }
    let Some(omega) = r.atom_value(i.parameter) else {
        return false;
    };
    (i.depth == 0 || omega.as_u64() == 1) && balanced(r, i.tree, i.depth)
}
pub(crate) fn poly_admitted<const N: usize>(
    r: &Reduction<N>,
    object: Order,
    key: &DigestKey,
    budget: u64,
) -> bool {
    let Ok(i) = metadata(r, object) else {
        return false;
    };
    if budget < i.poly_cost() {
        return false;
    }
    if i.depth > 0 && r.digest(i.self_reference).map(digest_key).as_ref() != Some(key) {
        return false;
    }
    balanced(r, i.tree, i.depth) && point(r, i.parameter, i.depth)
}

pub(crate) fn flatten<const N: usize>(
    r: &Reduction<N>,
    tree: Order,
    depth: u32,
    out: &mut alloc::vec::Vec<nebu::Goldilocks>,
) -> Option<()> {
    if depth == 0 {
        out.push(r.atom_value(tree)?);
        return Some(());
    }
    let (l, h) = pair_children(r, tree)?;
    flatten(r, l, depth - 1, out)?;
    flatten(r, h, depth - 1, out)
}

pub(crate) struct NttInput {
    pub input: Input,
    pub omega: nebu::Goldilocks,
    pub values: alloc::vec::Vec<nebu::Goldilocks>,
    pub remaining: u64,
}
pub(crate) struct PolyInput {
    pub input: Input,
    pub values: alloc::vec::Vec<nebu::Goldilocks>,
    pub point: alloc::vec::Vec<nebu::Goldilocks>,
    pub remaining: u64,
}
fn values<const N: usize>(
    r: &Reduction<N>,
    i: &Input,
) -> Result<alloc::vec::Vec<nebu::Goldilocks>, crate::reduce::Outcome> {
    use crate::reduce::Outcome;
    if !balanced(r, i.tree, i.depth) {
        return Err(Outcome::Error(ErrorKind::TypeError));
    }
    let mut v = alloc::vec::Vec::new();
    v.try_reserve_exact(i.words)
        .map_err(|_| Outcome::Error(ErrorKind::Unavailable))?;
    flatten(r, i.tree, i.depth, &mut v).ok_or(Outcome::Error(ErrorKind::TypeError))?;
    Ok(v)
}
pub(crate) fn prepare_ntt<const N: usize>(
    r: &Reduction<N>,
    object: Order,
    budget: u64,
) -> Result<NttInput, crate::reduce::Outcome> {
    use crate::reduce::Outcome;
    let input = metadata(r, object).map_err(Outcome::Error)?;
    let omega = r
        .atom_value(input.parameter)
        .ok_or(Outcome::Error(ErrorKind::TypeError))?;
    let remaining = budget
        .checked_sub(input.ntt_cost())
        .ok_or(Outcome::Halt(budget))?;
    let values = values(r, &input)?;
    Ok(NttInput {
        input,
        omega,
        values,
        remaining,
    })
}
pub(crate) fn prepare_poly<const N: usize>(
    r: &Reduction<N>,
    object: Order,
    budget: u64,
) -> Result<PolyInput, crate::reduce::Outcome> {
    use crate::reduce::Outcome;
    let input = metadata(r, object).map_err(Outcome::Error)?;
    let remaining = budget
        .checked_sub(input.poly_cost())
        .ok_or(Outcome::Halt(budget))?;
    if !point(r, input.parameter, input.depth) {
        return Err(Outcome::Error(ErrorKind::TypeError));
    }
    let values = values(r, &input)?;
    let mut coords = alloc::vec::Vec::new();
    coords
        .try_reserve_exact(input.depth as usize)
        .map_err(|_| Outcome::Error(ErrorKind::Unavailable))?;
    let mut cur = input.parameter;
    for _ in 0..input.depth {
        let (h, t) = pair_children(r, cur).ok_or(Outcome::Error(ErrorKind::TypeError))?;
        coords.push(
            r.atom_value(h)
                .ok_or(Outcome::Error(ErrorKind::TypeError))?,
        );
        cur = t;
    }
    Ok(PolyInput {
        input,
        values,
        point: coords,
        remaining,
    })
}
