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

pub mod artifact;
pub mod bound;
pub mod call;
pub mod data;
pub mod encode;
pub mod jets;
pub mod parallel;
pub mod patterns;
pub mod reduce;
pub mod sequential;
pub mod trace;

#[cfg(feature = "brakedown")]
pub mod brakedown_look;

pub use data::{Digest, NIL, Order, Reduction};
// Data and DataEntry are intentionally NOT re-exported. External callers that
// need pattern-matching access can import them through `nox::data::{Data, DataEntry}`,
// signaling reliance on internal representation. The default surface is the
// safe accessors on Reduction (head/tail/atom_value/digest).
pub use bound::{Cost, bound};
pub use call::{CallProvider, LookProvider, NullCalls, NullLooks};
pub use encode::{
    DecodeError, DecodedData, Particle, WireEntry, WireMessage, decode, encode_atom, encode_pair,
    encode_tree, encoded_bytes, parse_message, particle_id, particle_of, write_push, write_request,
    write_response,
};
pub use jets::registry::{DigestKey, JetRegistry, digest_key};
pub use parallel::{PathStep, StructuralIndex, reduce_parallel};
pub use reduce::{ErrorKind, Outcome, reduce, reduce_with_registry};
pub use trace::{NoTrace, TraceRow, Tracer, VecTrace};

#[cfg(feature = "brakedown")]
pub use brakedown_look::{BrakedownLookProvider, LookOpening};
#[cfg(feature = "brakedown")]
pub use encode::{encode_poly, poly_content_id};
