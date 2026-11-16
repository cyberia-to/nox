# Canonical complete artifacts — 0.4 delivery

Date: 2026-09-23. Base: `f8807c5a5b173c286b23d3125ac9ec480a2669b5`.
Branch: `feat/0.4-canonical-artifacts`; integration: `release/0.4`.
Contract: [specs/artifact.md](../specs/artifact.md).

`nox::artifact` implements the versioned NOXDAG01 root/DAG container. It uses
native Model B node payloads and full four-limb particle identities, with
left-first unique postorder rather than allocation order. Encode and decode
enforce independent byte, node and longest-path depth limits. Decoding checks
the complete format before VM allocation; allocation failure returns no root.
Existing network codecs and reduction/trace semantics are unchanged.

Validation on macOS aarch64, Rust1.95.0:

```sh
cargo check --workspace --all-targets --locked
cargo check -p cyber-nox --no-default-features --locked
cargo test --workspace --release --locked
rustfmt --edition 2024 --check rs/artifact.rs
git diff --check
```

All checks passed, with no compiler warnings. The full workspace suite passed
178 tests, including nine new artifact tests; zero failures or ignored tests.
The new module compiles with the no_std/default-library configuration and uses
the existing alloc dependency surface. No Cargo dependencies/versions changed.

Coverage includes independent expected leaf order; construction-history invariance;
topology/sharing/identity round trips; exact-limit success and one-below failures;
every truncation of the pair fixture; bad version/length/count/trailing bytes;
noncanonical atoms and all four limbs in header/entry/child particles; hash
tampering; duplicates/missing/forward/unreachable/reordered entries; shallow-first
shared paths whose longest depth exceeds the cap; and partial arena allocation
failure preserving preexisting nodes.

A 100-level repeated-child DAG round-trips as 101 nodes and fewer than10000
bytes. No leaves are expanded and no recursive host traversal is used. This is
a codec stress case, not compiler-scale execution or a proof measurement.

Independent read-only review found no blocking codec defect and prompted the
all-limb and encoder-depth assertions. Determinism, type/shape/range checks,
bounded traversal, error persistence and readability were reviewed. New source
files remain below500 lines. Trident's SH0.3 work consumes this codec for compiler
job/result goldens; Joy structured execution and Zheng relations remain separate
deliveries. No six-platform release or native self-hosting claim is made here.

Source/dependency and validation log identities are recorded in
[the receipt](native-artifacts-2026-09-23.json).
