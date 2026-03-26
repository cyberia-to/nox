//! atom or cell — the two kinds of noun. nothing else.

use nebu::Goldilocks;
use super::tag::Tag;
use super::NounRef;

/// the two kinds of noun
#[derive(Debug, Clone, Copy)]
pub enum NounInner {
    Atom { value: Goldilocks, tag: Tag },
    Cell { left: NounRef, right: NounRef },
}
