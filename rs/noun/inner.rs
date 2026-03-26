//! atom or cell — the two kinds of noun. nothing else.

use nebu::Goldilocks;
use super::tag::Tag;
use super::NounId;

/// the two kinds of noun
#[derive(Debug, Clone, Copy)]
pub enum Noun {
    Atom { value: Goldilocks, tag: Tag },
    Cell { left: NounId, right: NounId },
}
