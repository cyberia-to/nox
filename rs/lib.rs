// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! nox — proof-native virtual machine — algebra-parameterized
//!
//! 16 deterministic patterns + 1 non-deterministic call + 1 deterministic look.
//! every execution produces a trace that IS the zheng witness.
//!
//! reduce(object, formula, budget) -> Outcome

#![no_std]
#![allow(clippy::too_many_arguments)]

extern crate alloc;

pub mod noun;
pub mod reduce;
pub mod call;
pub mod patterns;
pub mod trace;
pub mod bound;
pub mod parallel;

pub use noun::{Order, NounId, Tag, Digest, NIL};
// Noun and NounEntry are intentionally NOT re-exported. External callers that
// need pattern-matching access can import them through `nox::noun::{Noun, NounEntry}`,
// signaling reliance on internal representation. The default surface is the
// safe accessors on Order (head/tail/atom_value/digest).
pub use reduce::{reduce, Outcome, ErrorKind};
pub use call::{CallProvider, NullCalls, LookProvider, NullLooks};
pub use trace::{TraceRow, Tracer, NoTrace, VecTrace};
pub use bound::{bound, Cost};
pub use parallel::{reduce_parallel, StructuralIndex, PathStep};
