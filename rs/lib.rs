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

// When the `std` feature is disabled, operate in no_std + alloc mode.
// When enabled, link against the full std crate (required for threading).
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]

extern crate alloc;

pub mod noun;
pub mod reduce;
pub mod call;
pub mod patterns;
pub mod trace;
pub mod bound;
pub mod parallel;
pub mod encode;
pub mod jets;

#[cfg(feature = "brakedown")]
pub mod brakedown_look;

pub use noun::{Order, NounId, Tag, Digest, NIL};
// Noun and NounEntry are intentionally NOT re-exported. External callers that
// need pattern-matching access can import them through `nox::noun::{Noun, NounEntry}`,
// signaling reliance on internal representation. The default surface is the
// safe accessors on Order (head/tail/atom_value/digest).
pub use reduce::{reduce, reduce_with_registry, Outcome, ErrorKind};
pub use jets::registry::{JetRegistry, DigestKey, digest_key};
pub use call::{CallProvider, NullCalls, LookProvider, NullLooks};
pub use trace::{TraceRow, Tracer, NoTrace, VecTrace};
pub use bound::{bound, Cost};
pub use parallel::{reduce_parallel, StructuralIndex, PathStep};
pub use encode::{
    ContentId, DecodeError, DecodedNoun, WireEntry, WireMessage,
    encode_field, encode_word, encode_hash, encode_cell,
    noun_id, encoded_bytes, content_id, encode_tree,
    decode, parse_message, write_push, write_request, write_response,
};

#[cfg(feature = "brakedown")]
pub use encode::{poly_content_id, encode_poly};
#[cfg(feature = "brakedown")]
pub use brakedown_look::{BrakedownLookProvider, LookOpening};
