// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! nox — proof-native virtual machine — algebra-parameterized
//!
//! 16 deterministic patterns + 1 non-deterministic call + 1 deterministic look.
//! every execution produces a trace that IS the STARK witness.
//!
//! reduce(object, formula, budget) -> Outcome

#![no_std]

extern crate alloc;

pub mod noun;
pub mod reduce;
pub mod call;
pub mod patterns;

pub use noun::{Order, NounId, Noun, Tag, Digest, NIL};
pub use reduce::{reduce, Outcome, ErrorKind};
pub use call::{CallProvider, NullCalls, LookProvider, NullLooks};
