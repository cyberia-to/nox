// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! nox — proof-native virtual machine over the Goldilocks field
//!
//! 16 deterministic patterns + 1 non-deterministic hint.
//! every execution produces a trace that IS the STARK witness.
//!
//! reduce(object, formula, budget) → Result

#![no_std]

extern crate alloc;

pub mod noun;
pub mod reduce;
pub mod hint;

pub use noun::{Arena, NounRef, NounInner, Tag, Digest, NIL};
pub use reduce::{reduce, Outcome, ErrorKind};
pub use hint::{HintProvider, NullHints};
