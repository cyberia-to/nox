# Preserve nox's no_std dependency boundary

Stage: revised PLAN v2, awaiting fresh different-vendor review. Parent:
`0b1407a4b855e4961ede07e1b7ea199cb8018c30`. Lineage and measured failures are
in [research.md](research.md); exact feature/ring analysis is in
[feature-audit.md](feature-audit.md). [review-v1.md](review-v1.md) disposes the
first review. No production change before fresh approval.

## Exact change and durable guard

In `rs/Cargo.toml` only these two values change:

```diff
-std = []
+std = ["hemera/std"]
@@
-hemera = { package = "cyber-hemera", version = "0.3", path = "../../hemera/rs" }
+hemera = { package = "cyber-hemera", version = "0.3", path = "../../hemera/rs", default-features = false }
```

Add a project-specific `no_std dependency gate` section to `CLAUDE.md`, naming
this executable every-commit gate, requiring an installed thumb target and the
repository's normal committed sibling layout:

```sh
cargo build -p cyber-nox --no-default-features --target thumbv7em-none-eabi --locked
```

Run from the repository root. CI remains unchanged; the named repository gate
is durable and code-generates the real library. Local evidence additionally uses
`--offline` and an isolated checksum-verified source cache. Do not install a target
silently when it is absent; record a missing prerequisite as a red gate. No new
script or runtime dependency is necessary.

With nox defaults disabled, the nox edge no longer enables Hemera/std. Explicit
nox/std forwards it; parallel/brakedown/wgpu/honeycrisp already include nox/std.
Other dependency edges can still independently enable Hemera/std. The two std-only
Hemera APIs and every architecture cfg site have been audited; source selections
for arithmetic are independent of std. All paths, versions, optionality, type
identities, Rust source, locks, CLI manifest, algorithms and canonical specs stay
byte-identical. Only the manifest and named gate declaration are production scope;
task/audit evidence is separate. No loose audit file is named Cargo.toml.

## Steps → verification

1. Freeze parent, sibling exports, registry checksums, toolchain and tag → verify
   complete Git body/mode/link inventories. Retain all failed harness attempts.
   Measure each introduced command at the parent before changing production,
   including workspace debug/release, owning std/parallel/brakedown, docs, pinned
   nightly doctests, format, Clippy, reduction benchmark, CLI, H264 and package
   variants. Existing results remain attributed to their actual revision/command.
2. Apply the exact manifest and gate declaration after approval → verify only those
   intended bytes differ. Candidate exports preserve all other frozen input bytes.
   Parent/candidate/package/nightly/compiler JSON builds use distinct actual
   compilation roots. Pin toolchain-bin PATH as well as RUSTC/RUSTDOC, avoiding
   mixed Homebrew/rustup Clippy drivers. Two jobs; incremental disabled.
3. Build actual no_std library → the named thumb build, host no-default-features
   check, and normalized-package thumb build must pass. The parent thumb failure
   is specifically Hemera `extern crate std`. Inspect default feature metadata
   and explicit std inverse tree: default must exclude Hemera/std; std must include
   it. Do not suppress a dependency, target or real kernel to obtain green.
4. Preserve behavior → repeat exact parent workspace debug/release and owning
   std/parallel/brakedown tests, CLI build/test, wgpu check, honeycrisp check, and
   the four actual Kadek H264 literal/error/resource tests in debug/release.
   Honeycrisp currently resolves acpu but fails on its absent `ntt_forward` and
   `multilinear_eval` functions; preserve that inherited result, not an obsolete
   cache-unavailable claim. These controls do not represent the whole Kadek suite.
5. Run the named `reduction` benchmark in parent/candidate under the same compiler,
   flags, two jobs, no incremental, target architecture and Criterion settings:
   `cargo bench -p cyber-nox --bench reduction --locked --offline --
   --warm-up-time 1 --measurement-time 2 --sample-size 20 --noplot`.
   Preserve all nine estimates/samples, compare per-case confidence intervals and
   relative point estimates, record environment/concurrent-load limits. A measured
   regression or indeterminate noisy result is an explicit red/unverified gate,
   not waived because algorithms are unchanged. Investigate a >5% median slowdown
   or disjoint slower confidence interval with one paired repeat before attribution;
   do not promise a speedup or alter Rust to tune this manifest fix.
6. Check distribution without releasing → `cargo package -p cyber-nox --no-verify
   --locked --offline --allow-dirty`, inspect the normalized manifest and generated
   package lock. External paths are removed by Cargo, not production edits. Package
   Rust must equal owning source. Test the extracted library on host and build it
   for thumb. Repeat the same H264 literals using registry NeBu 0.1.1 matching the
   package in a separate nonmember harness/target. Measure its parent first: any
   already-present literal/overflow error belongs to that registry dependency
   epoch, not this feature patch. Do not normalize production NeBu to repair it.
7. Repeat format, workspace/all-target Clippy `-D warnings`, owning docs with
   `RUSTDOCFLAGS=-D warnings`, and actual nightly-2025-11-26 doctests. Compare exact
   first errors and error sets with parent. Preserve unrelated format, CLI lint,
   docs, optional-backend and package-epoch reds; no unrelated source edits.
8. Owning API → compare current parent/candidate default surface through separately
   emitted, hashed JSON from separate actual rustdoc targets; expected surface
   floor is patch. Attempt the exact-tag v0.2.0 locked baseline; its committed lock
   presently requires an update (record 64), so this required gate is unmet. An
   alternate generated-lock tag comparison, if retained, is diagnostic only and
   never substitutes for that red. Automatic runs 60/61 returned pass but reused a
   compiler target, so are not accepted as isolated proof. No 26-crate API rerun.
9. Close the ring honestly → keep each named reverse-consumer feature audit,
   isolated Trident prerequisite, all unexecuted product/build/test/spec/Kelvin
   obligations explicit. No sibling normalization, product bypass, fleet effects
   or release action. Freeze final source, exact commands/raw streams, benchmark
   samples, known reds and feature/API limits for fresh CODE review and portable
   Nu replay. Root owns explicit staging/ordinary merge.

Targets stay below 20 GiB, staging below 30 GiB. Reuse source-identical parent
results when appropriate, without claiming a different command or environment was
run. Registry archives and available toolchains are fixed; no installation is part
of this plan. Nu verifies the portable audit; historical Ruby capture scripts remain
evidence and do not become repository tooling dependencies.

## Five likely failures

1. Lost inherited Hemera/std affects APIs or backend performance → enumerate its
   three std API/linkage guards and fourteen architecture guards; audit every named
   consumer's feature inheritance; execute paired reduction and std feature checks.
2. no_std is only a host illusion → real thumb code generation in sibling and
   normalized-package epochs plus the named durable repository gate.
3. Delivery accidentally changes identity or hides older NeBu behavior → keep all
   production paths and pins, compare package metadata, measure registry H264
   separately, preserve any pre-existing overflow/literal errors.
4. Environment/cache drift disguises a red → checksum-pinned inputs, source
   before/after proof, toolchain-bin PATH, separate parent/candidate/rustdoc roots;
   all new gates have exact parent baselines, including harness failures.
5. Local success becomes a false delivery/phase claim → ring 1 and `decision`,
   product/Kelvin and exact-tag reds explicit. Kadek still resolves unchanged
   registry nox 0.2.0. Owner-only publication/version delivery and separate Kadek
   alloc/math/public selector work remain open.

## Acceptance and remaining obligations

Accept the scoped source repair only with real no_std passes, no unexplained new
failure, required paired evidence and fresh CODE approval. It is not a clean ring-1
candidate. Existing owner authorization covers ordinary implementation/review/merge;
`decision` and any required `wip:` label record the retained reds rather than creating
another permission request. No version, tag, registry publication or release is
performed. Future release floor includes the graph and inherited dependency-feature
contract, beyond cargo-semver-checks' default-surface result.
