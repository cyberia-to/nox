# Release hygiene — 2026-10-16

The default/core quality gates are restored on base
`7fe9ad368dc93ef136c196c2366b39f1c67e1cd7`. Final compiled closure:
`f61c0ba0b475a463db1bc627a592b4f3c16c8dc88d8d3a723aba3766409fd1df`
([files and sibling revisions](sources-reviewed.json)). This is a local source
repair, not release readiness or registry delivery.

The [plan](../../.claude/plans/release-hygiene.md) limits this unit to
mechanical formatting, one redundant CLI borrow, rustdoc markup and three
equivalent test expressions. A separately formatted pristine export isolates
the [precise non-format patch](nonformat-reviewed.patch); the
[comparison](format-bridge-reviewed.json) records every source hash.
The original formatter check changed 44 files; markup in `trace.rs` adds the
45th changed Rust file. Manifests, lockfile, dependency versions, runtime
algorithms, expected test values and benchmark logic retain their original bytes
apart from formatter changes where applicable. The fixed odd-modulus division
has a [compiled numeric proof](div-ceil-proof.json); the pure trace predicate
selects the same last matching row using `rfind`.

## Exact final checks

All commands use pinned rustup 1.95.0, `RUSTFLAGS=-Dwarnings`,
`RUSTDOCFLAGS=-Dwarnings`, an external cache and unchanged before/after compiled
closure. [check.nu](check.nu) takes a repository argument; each linked receipt
contains complete argv, tool paths, source identity and raw stream hashes.
Final receipts are named `*-reviewed`; earlier `*-final` filenames belong to the
intermediate source epoch described below. [verify.nu](verify.nu) rechecks source,
receipt streams and the precise patch; its successful [handoff](handoff.json)
also records branch, status and worktree placement.

| Receipt | Command and result |
|---|---|
| [fmt](fmt-reviewed.json) | `cargo fmt --all -- --check`: pass. |
| [Clippy](clippy-reviewed.json) | `cargo clippy --workspace --all-targets --locked --offline -- -D warnings`: pass. |
| [Docs](docs-reviewed.json) | `cargo doc -p cyber-nox --no-deps --locked --offline`: pass. |
| [Debug](debug-reviewed.json) | `cargo test --workspace --locked --offline`: 161 library tests pass; CLI/doc targets have no tests. |
| [Release](release-reviewed.json) | `cargo test -p cyber-nox --lib --release --locked --offline`: 161 pass. |
| [Thumb](thumb-reviewed.json) | `cargo build -p cyber-nox --no-default-features --target thumbv7em-none-eabi --locked --offline`: pass, actual code generation. |
| [Package](package-reviewed.json) | `cargo package -p cyber-nox --locked --offline --allow-dirty`: 53 files, verification passes. |
| [Package thumb](package-thumb-reviewed.json) | `cargo build --manifest-path <normalized-package>/Cargo.toml --no-default-features --target thumbv7em-none-eabi --locked --offline`: pass. |

[Package identities](package-identity.json) bind every packaged Rust source to
the owning bytes. Archive SHA256 is
`53485d0256180839cec29fae34d49b18b8e59c4d93f66c706ac8d1602e5f658d`.
The normalized manifest and lock hashes equal the parent diagnostic package;
registry Hemera 0.3.1/NeBu 0.1.1 remain distinct from local sibling inputs.
This still declares the occupied version 0.2.0 and records `dirty: true`.
No tag, version change, publication or dependency substitution occurred.

## Preserved failures and scope

Source epochs are explicit: initial `befee149…` retained the three test-only
Clippy errors in [clippy.json](clippy.json); the approved `next_back` epoch
`8e9e804a…` exposed `filter_next` in [clippy-final.json](clippy-final.json).
Its other gates passed. The final `f61c0ba0…` epoch changes that same expression
to `rfind`. Earlier manifests, patches and raw logs remain intact. Original
clean-parent formatting/Clippy/rustdoc failures are in `parent/`; the complete
formatter output is losslessly compressed. [Preparation notes](preparation.md)
separate audit-runner errors from executed Cargo results.

These independent prerequisites stay open; the receipts below were inspected
and retained from the clean parent, not rerun or relabeled as final-source gates:

- [Honeycrisp](open-prerequisites/honeycrisp.json): acpu 0.2.0 lacks
  `ntt_forward` and `multilinear_eval`; no fallback was introduced.
- [Trident](open-prerequisites/trident-current.json) and
  [Soft3 conformance](open-prerequisites/conformance-current.json): current
  first version-resolution failures remain Nox `^0.3.0` and Hemera `^0.4`.
- [Historical release audit](../2026-10-13-hemera-no-std/README.md): the two
  paired performance reds remain unestablished. No performance measurement or
  optional-feature matrix was rerun by this hygiene unit.
- [Exact-tag baseline](../../.claude/tasks/exact-tag-api-baseline/research.md):
  immutable tag/lock mismatch and API-baseline policy remain unresolved.
- Existing oversized modules are enumerated by the format comparison; this
  authorized mechanical unit does not claim to close that structural debt.

The owner-only bump merge, tag and crates.io publication remain necessary
before Kadek can consume a new registry checksum. The reused checkout is kept
for root review/integration; no agent commit has been made at this handoff.
