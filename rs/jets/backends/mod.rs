// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! Backend selection for genesis jets.
//!
//! Selection order at runtime:
//!   1. Honeycrisp (Apple Silicon AMX+Metal) — if feature "honeycrisp" and aarch64 macOS
//!   2. WGPU (cross-platform GPU) — if feature "wgpu"
//!   3. CPU/Rust — always available, zero external deps
//!
//! All backends are observationally equivalent. The jet functions are
//! identical at the interface; only the inner computation path differs.

pub mod cpu;

#[cfg(feature = "wgpu")]
pub mod wgpu;

#[cfg(all(feature = "honeycrisp", target_os = "macos", target_arch = "aarch64"))]
pub mod honeycrisp;

use crate::jets::registry::JetRegistry;

/// Select the best available backend and return the genesis registry.
pub fn select<const N: usize>() -> JetRegistry<N> {
    #[cfg(all(feature = "honeycrisp", target_os = "macos", target_arch = "aarch64"))]
    {
        if honeycrisp::available() {
            return honeycrisp::genesis_honeycrisp();
        }
    }
    #[cfg(feature = "wgpu")]
    {
        if wgpu::available() {
            return wgpu::genesis_wgpu();
        }
    }
    cpu::genesis_cpu()
}
