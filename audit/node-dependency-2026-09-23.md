# Node dependency review — 2026-09-23

The Cyber dependency snapshot now includes the existing jet input admission
repair and its tests. The reviewed branch starts at `85ede1d` and preserves the
owner working tree. Nox is `0.3.0`, its CLI is `0.2.0`, and its optional Brakedown
dependency accepts the sibling `cyber-lens-brakedown 0.2.0` source.

## Reviewed behavior

Exact NTT and polynomial jets validate bounded input metadata, budget and shape
before allocating their decoded inputs. A declined runtime jet takes pure
pattern dispatch with the original budget. Regression tests compare the result,
trace and remaining budget with a registry-free execution.

The patch also preserves pure semantics when the polynomial self-reference
changes, when a depth-zero value is a pair, and when the NTT formula describes
only a butterfly. Full NTT kernel replacement is restricted to the cases where
it equals that actual formula. Acceleration bounds leave the pure VM path
available; they are not data storage limits.

## Validation

Native checks ran on macOS arm64 with Rust `1.95.0`:

- `cargo check --workspace --tests --locked`: passed without compiler warnings.
- `cargo test --workspace --locked`: 169 passed, zero failed or ignored.
- `cargo test --workspace --all-features --locked`: 175 passed, zero failed or
  ignored; includes native Honeycrisp admission parity and Brakedown look tests.
- `cargo bench --locked --bench reduction -- --quick`: all nine existing
  benchmarks completed. This is an execution smoke test; no comparative
  performance baseline was established.
- `git diff --check`: passed.

An all-feature `x86_64-unknown-linux-gnu` cross-check passed using rustup Rust
`1.98.0`, with `RUSTC` explicitly set to that toolchain. The shell's Homebrew
compiler has no Linux standard library. Linux execution was not performed.

The optional Apple backend uses the committed local Honeycrisp source
`9a32c93bdd1e679127506c460eb057bb69641fe4`; Linux selects CPU fallback. The sibling
Lens 0.2 source must accompany this snapshot. Release assembly must pin all of
these repositories together; a path dependency alone supplies no source lock.

This review does not establish proof soundness for every jet, complete the
recursive NTT formula, or turn the WGPU fallback into a GPU implementation.
