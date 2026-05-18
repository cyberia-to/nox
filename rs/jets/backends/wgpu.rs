// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! WGPU GPU backend for genesis jets.
//!
//! Cross-platform GPU dispatch via wgpu + WGSL kernels.
//! For 0.1.0, all jets fall back to the CPU backend.
//! WGSL kernel dispatch will be added in 0.2.0.
//!
//! Availability check: always returns false in 0.1.0 so the
//! backend selection in mod.rs will skip to CPU.

use crate::jets::registry::JetRegistry;

/// Returns true if a WGPU-capable device is available.
/// Always false in 0.1.0; full device detection added in 0.2.0.
pub fn available() -> bool {
    false
}

/// Build the genesis registry using WGPU-accelerated jets.
/// Falls back to the CPU backend for 0.1.0.
pub fn genesis_wgpu<const N: usize>() -> JetRegistry<N> {
    super::cpu::genesis_cpu()
}
