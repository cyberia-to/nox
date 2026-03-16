//! nox — proof-native virtual machine over the Goldilocks field
//!
//! Sixteen deterministic reduction patterns (Layer 1), one non-deterministic
//! hint pattern (Layer 2), five jets for efficient recursive STARK
//! verification (Layer 3).
//!
//! # Module structure
//!
//! ```text
//! noun      — Atom, Cell, Noun (binary tree of field elements)
//! reduce    — Layer 1 reduction engine (16 patterns)
//! hint      — Layer 2 non-deterministic witness injection
//! jet       — Layer 3 jets (hash, poly_eval, merkle_verify, fri_fold, ntt)
//! trace     — execution trace recording (becomes the STARK witness)
//! encode    — canonical noun serialization (deterministic wire format)
//! memo      — content-addressed computation cache
//! focus     — resource metering (attention budget)
//! ```

pub mod noun;
pub mod reduce;
pub mod hint;
pub mod jet;
pub mod trace;
pub mod encode;
pub mod memo;
pub mod focus;
