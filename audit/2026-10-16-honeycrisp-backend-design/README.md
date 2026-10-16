# Honeycrisp backend delivery — 2026-10-16

The missing symbols are an unpublished upstream addition. Nox already calls the
correct API; prefer an acpu release, without adding another NTT/interpolation
implementation to Nox. No production source, manifest, lock or ref changed in
this investigation. [Plan](../../.claude/plans/honeycrisp-backend.md).

## Identity and cause

Nox base is `434d48ee7e94fdde0f4e7bb71012afb80db21a7e`; canonical Honeycrisp is
`70dfb1f55e4cb2570d5a78a497be5828a203087b`. Published `acpu 0.2.0` is VCS
`96fa4f4509d7a966cb22a4bb9e12ac753cff43ae`, archive checksum
`81089272e2a248781b1e2c38ad7247a98bea63a602c0a445dc63e955b2035375`.
[Identity verification](identities.json) compares the complete cached extraction
against that archive and matches the Nox lock and fresh official
[sparse-index response](registry-index.jsonl).

The published `field/mod.rs` exports scalar/batch Goldilocks arithmetic,
Poseidon2 and Merkle functions. It has neither missing function. Honeycrisp
[568e41f](https://github.com/cyberia-to/honeycrisp/commit/568e41f433558a01a7a70d4febcc91cb1852ee0f)
added them on 2026-05-18; Nox commit
`5ae2e6655919be03d099e6d942a857dee40e1ba1` wired the callers that day. Honeycrisp
still declares 0.2.0, and the fresh index contains only that original version.
These APIs were added, not renamed. The observed failure is delivery lag.

## Contract checked

| Boundary | Established behavior and exact source |
|---|---|
| NTT | `acpu/src/field/ntt.rs:60`: in-place radix-2 DIT, bit reversal, supplied root, positive power-of-two length. Inverse root gives inverse-direction sums without dividing by length. Nox `rs/jets/backends/honeycrisp.rs:69–103` uses the same convention and `log2(length) * length` budget as its CPU jet. |
| Multilinear | `acpu/src/field/poly.rs:29`: half-split interpolation; first coordinate selects the most-significant hypercube bit. Nox `honeycrisp.rs:132–180` flattens left-to-right, supplies exactly the declared coordinate count and charges the evaluation count. This is not univariate Horner evaluation. |
| Representation | Upstream arithmetic can return redundant residues: the probe observes raw `p` and `p+1`. Nox wraps these in `Goldilocks`, whose `as_u64` and identity hashing canonicalize them. The module's broad canonical-form wording should be clarified upstream. |
| Hardware | These functions compose the existing aarch64 integer `mul/umulh` primitives. No AMX or Metal dispatch occurs in either function. Nox's AMX/Metal prose is stale; acceleration/speedup was not measured here. |
| Features | Nox `honeycrisp = ["std", "dep:acpu"]`; Nox `std` forwards `hemera/std`. acpu uses std unconditionally and declares no feature surface. Published unimem 0.2.0 also has no opt-out feature; current source unimem has default `std`. Keep this optional Apple-Silicon backend separate from core no_std delivery. |

## Executed evidence

[check.nu](check.nu) records argv, pinned rustup 1.95.0, warnings denied,
external cache, raw streams and exact before/after source closures. The temporary
diagnostic manifests and locks are retained as `*.toml.txt`; their acpu patches
never enter production manifests. Source closure is `fbe4090a…`
([full identity](sources.json)); normalized-package closure is `9a2b844e…`
([full identity](sources-package.json)). [verify.nu](verify.nu) rechecks both
closures and every receipt stream; its successful result and clean production
status are retained in [handoff.json](handoff.json). Preparation-only errors are
separated in [preparation.md](preparation.md).

| Receipt | Actual result |
|---|---|
| [Registry feature](registry-feature.json) | `cargo check -p cyber-nox --features honeycrisp --locked --offline`: red, exactly the missing `ntt_forward` and `multilinear_eval` symbols. |
| [Source debug](source-debug.json), [source release](source-release.json) | `cargo test --manifest-path <source-diagnostic>/Cargo.toml [--release] --locked --offline`: four tests pass in each profile. The acpu patch selects canonical source and its local unimem. |
| [Packaging](acpu-package.json) | `cargo package -p acpu --locked --offline` on clean Honeycrisp: 94 files; actual normalized verification passes using registry unimem 0.2.0. |
| [Package consumer](package-consumer.json) | `cargo test --manifest-path <package-diagnostic>/Cargo.toml --locked --offline`: the same four tests pass with normalized acpu plus registry unimem, preserving this distinct dependency epoch. |

The authored [probe](probe.rs) checks 48 NTT configurations against an independent
u128 direct DFT, 48 multilinear configurations against independent hypercube
basis sums, explicit redundant-residue cases and 40 Nox CPU/Honeycrisp comparisons
of result values/identity, budget, trace row and emitted trace. Sizes and budgets
are finite; repeated zero-dimensional configurations are counted as configured
cases. This is a bounded contract diagnostic, not conformance certification.

Every normalized acpu Rust source equals its owning bytes. Local archive SHA256:
`661b7735d62550242e41d4560151d9e6dd3831183f06225bf622a87ed148fc9e`.
Its version is still occupied 0.2.0; the archive is only a diagnostic artifact.

## Preferred release path and remaining gates

Recommend an acpu-only `0.2.1` release, subject to the upstream gates below.
The [complete library delta inventory](api-inspection.json) adds four files and
only adds module/re-export declarations in the two existing edited files;
the acpu manifest and existing implementation bodies are unchanged. Additive
APIs fit Cargo's compatible `0.2.x` convention; this is a source/API assessment,
not a completed whole-workspace compatibility verdict.
[Cargo SemVer guidance](https://doc.rust-lang.org/cargo/reference/semver.html).

The smallest version-only proposal sets `acpu/Cargo.toml`'s package version to
`0.2.1` explicitly instead of inheriting the workspace version, updates the
generated root lock and changelog, and keeps existing internal dependency
constraints/features unchanged. This package-specific version exception needs
owner review. Raising the shared workspace version would unnecessarily couple
unimem/rane/aruminium releases to this repair. No such edit was made here.

After owner merge/tag/publication, Nox's existing `acpu = "0.2"` requirement
already accepts 0.2.1: update its generated lock to the new registry checksum,
without changing feature/std wiring. Re-run the real unpatched Nox honeycrisp
tests/strict checks, default/no_std controls and package consumer checks. Keep
that lock update separate from Nox's own version bump. No source patch or
published-0.2.0 substitution belongs in the release path.

Before a version-only PR, Honeycrisp's root `CLAUDE.md:39–45` requires workspace
fmt, Clippy, release build and tests; `acpu/CLAUDE.md:34–40` additionally requires
the real AMX matmul example. Use `cargo fmt --all -- --check`, strict
`cargo clippy --workspace -- -D warnings`, `cargo build --release --workspace`,
`cargo test --workspace`, and `cargo run -p acpu --example matmul` on Apple Silicon.
These broad upstream gates and API/performance baselines were not rerun here.
The new sparse APIs ship in the same package and need upstream test coverage;
these four diagnostic tests do not validate that surface. Packaging must be
repeated on the actual bumped, committed source with a registry-backed consumer.
Agents prepare evidence; only the owner merges a bump, tags and publishes.

Separate admission debt remains: acpu's current multilinear preconditions are
partly debug-only; unmerged branch `launch/131-field-power-of-two-release`
(`904dd336…`) addresses length checks but not exact point-count validation.
Nox's CPU and Honeycrisp wrappers both have unchecked dimension shifts and
recursive flattening. This probe uses valid, bounded inputs and does not certify
malformed-input behavior or the broader pure-formula jet contract. None of this
justifies duplicating the existing upstream algorithms.
