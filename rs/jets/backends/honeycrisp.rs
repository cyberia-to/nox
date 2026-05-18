// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! Honeycrisp Apple Silicon AMX+Metal backend for genesis jets.
//!
//! Targets aarch64 macOS using Apple Matrix Coprocessor (AMX) for polynomial
//! arithmetic and Metal/MSL kernels for hash (Poseidon2) and NTT.
//! For 0.1.0, all jets fall back to the CPU backend.
//! AMX and Metal kernels will be added in 0.2.0.
//!
//! Availability check: always returns false in 0.1.0 so the
//! backend selection in mod.rs will skip to CPU.

use crate::jets::registry::JetRegistry;

/// Returns true if Apple Silicon AMX is available.
/// Always false in 0.1.0; AMX feature detection added in 0.2.0.
pub fn available() -> bool {
    false
}

/// Build the genesis registry using Honeycrisp (AMX+Metal) jets.
/// Falls back to the CPU backend for 0.1.0.
pub fn genesis_honeycrisp<const N: usize>() -> JetRegistry<N> {
    super::cpu::genesis_cpu()
}
