# Hemera feature propagation

Stage: active research. Serves the owner's Kadek phase-1 foundation closure,
through nox's declared `no_std + alloc` mode and Kadek foundation row 7's real
scalar kernels. Discovered from Kadek private FLAC header audit, PR 60, merged
as `b939c75e553bc7d8c645216555e8d80d20f3d950`. Public FLAC selection follows.

Owning nox parent is origin/master `0b1407a4b855e4961ede07e1b7ea199cb8018c30`.
Worktree: `nox-no-std-dependencies`, branch `fix/no-std-dependencies`. No production
change has been made. `rs/lib.rs:15` already selects no_std when `std` is disabled.
But `rs/Cargo.toml` enables Hemera defaults unconditionally; Hemera's default
includes `std`. Cargo features are additive: Kadek's existing
`nox default-features=false` cannot remove a feature enabled by this dependency.

## Exact inputs and observations

Research commands and full streams are frozen under the research evidence's
`records/NN-*/`. Two wrapper versions are retained; the later one selects a private
registry and explicit Rust 1.95.0. Both use two jobs and no incremental compilation.

| Input | Exact revision |
| --- | --- |
| nox origin/master | `0b1407a4b855e4961ede07e1b7ea199cb8018c30` |
| Hemera origin/main | `d5f0a09539360934fb32db9b2e15a91995607db6` |
| Strata origin/main | `494df7a0973ae10a6cc1c2022e644d70449a29d6` |
| Lens origin/master | `8b26cef6587bee6a686f2cf163f6a8496931ee6a` |
| Kadek control source | `b939c75e553bc7d8c645216555e8d80d20f3d950` |

Records 02–04/08 retain live `git ls-remote --symref origin HEAD` results.
`committed-inputs.json` verifies every mode, blob and body/link target of isolated
Git exports: 143/100/244/49 entries respectively. These exports, not original
sibling working trees, are Cargo inputs. Registry material is extracted from 186
locally cached crate archives whose SHA-256 matches the exact lock/index entry;
all unpacked file hashes and directory-source configuration are retained. No
upstream code, package checksum, compiler or dependency setting was edited.

- Records 09/10 first failed during resolution: the old shared directory cache
  lacked optional `acpu`. These are prerequisite failures, retained unchanged.
- Record 11 runs exact-parent `cargo check -p cyber-nox --no-default-features
  --target thumbv7em-none-eabi --locked --offline` with the isolated manifest.
  It fails with `error[E0463]: can't find crate for std` at the committed Hemera
  `rs/src/lib.rs:55`, before nox builds. Record 17's inverse feature tree names
  nox → Hemera/default → Hemera/std directly.
- Record 13 runs `cargo test -p cyber-nox --lib --locked --offline` against these
  same committed siblings: 161 pass, zero failures or ignored cases.
- Records 15/16 run the actual Kadek H264 formula and scalar adapter against this
  nox and the same path NeBu. Four tests pass: three unchanged literal/error tests
  extracted from the committed integration test plus the unchanged resource test.
  Record 15 creates the external nonmember harness lock offline; 16 repeats it
  locked. This is a bounded kernel control, not the full Kadek workspace or its
  independent FFmpeg vector suite. Harness source and extraction provenance are
  explicit, with no mock nox implementation.

## Packaging and delivery are distinct

Record 12 runs `cargo package -p cyber-nox --no-verify --locked --offline
--allow-dirty` in the isolated exact-parent export. Cargo creates a local archive
with 52 files. Its normalized manifest already removes all external path keys
while retaining the published version requirements. Its generated lock resolves
registry NeBu 0.1.1, Hemera 0.3.1, lens-core 0.1.2 and lens-brakedown 0.1.1.
Record 14 checks that extracted archive for thumb and reproduces the same first
Hemera `std` error. Thus the packaging route is real; a successful local archive
after the proposed fix will be evidence, not a publication or release candidate.

The registry index acquired in record 01 lists nox only through 0.2.0. Every
published version enables Hemera defaults. The current nox origin's 48 Rust files
match the published 0.2.0 source in 46 files; the two remaining jet diffs contain
comments only. This does not make current siblings identical to registry sources:
Hemera has implementation changes, and current NeBu includes a Dot overflow fix
absent from published 0.1.1. The existing path identity must remain intact.

A direct Git dependency on nox remains unavailable because its four external
sibling paths are not within the fetched repository. Normalizing those paths in
production was considered and rejected: CLI and real soft3, zheng, mudra and bbg
consumers share path NeBu values with `Reduction::atom(Goldilocks)`. Switching only
nox to registry NeBu creates a new type-identity break. Committed consumer manifests
and concrete exchange excerpts are retained. Trident independently already requests
nox 0.3.0 while this origin is 0.2.0; that is not a reason to introduce another break.

The minimal upstream repair is therefore feature propagation only. Delivery to
Kadek waits for an owner-authorized version/release/publication action; no such
action is part of this unit. An upstream source merge alone does not fix Kadek's
locked registry dependency. A later Kadek alloc/math unit can be reviewed separately
while keeping its thumb gate explicitly blocked on this dependency delivery.

## Limits and follow-up

Kadek-ref itself lacks `#![no_std]`; its VLC tables and transforms use Vec, and
transforms call std-only sin/cos/sqrt methods. No kernel may be hidden to make the
gate pass. Explicit alloc imports and a reviewed portable math boundary are the
separate next owning change; libm precision must be assessed against existing
contracts and literals. This research does not change float algorithms or claim
full runtime conformance. Physical-memory/pinning policy remains untouched.

Nox's committed CLAUDE requires clean tests/type checks/lints and preserves manifest
discussion; root has accepted this narrow feature-only scope for formal PLAN.
The current dev doctrine requires fresh different-vendor PLAN and CODE reviews,
the last-tag API gate and ring-aware evidence. Nox is the layer-1 machine; registry
`part_of`/dependent obligations are retained. No release-train component version,
pin, tag or frozen default branch is modified. Product/consumer gates that cannot
run from exact inputs must remain explicitly unverified, never relabeled green.

## PLAN-review correction evidence

The first review requested feature reachability, durable no_std coverage, benchmark
and ring/baseline precision. See [review-v1.md](review-v1.md) and
[feature-audit.md](feature-audit.md). Their exact commands and complete streams are
new records 19–67; v1's frozen source/records are unchanged.

The corrected parent matrix measures workspace debug/release, std, parallel,
brakedown, pinned-nightly doctests, CLI build/test and wgpu check successfully. The
nine-case reduction benchmark completes with all estimates/samples retained. Both
source and packaged-source H264 controls pass four tests in debug and release;
normalized-package library tests pass 161. These are parent controls, not a fixed
no_std candidate. The real thumb build still fails at Hemera's `extern crate std`.

Inherited first errors: format check reports existing formatting differences;
correct-driver Clippy reports needless_borrow at cli/main.rs:247; strict docs
report unresolved `11` at rs/call.rs:47; honeycrisp reports missing
acpu::field::ntt_forward at rs/jets/backends/honeycrisp.rs:85. Trident's actual
locked metadata fails first on unavailable cached clap 4.5.57. Tag v0.2.0's real
locked rustdoc build fails because its committed Cargo.lock requires update.
Each failed gate retains its exact full output, not just this summary. No fix to
these unrelated sources/locks is proposed.

API automatic parent/tag diagnostics return 229 pass/31 skip but share an actual
compiler target, violating this task's isolation rule; they are not accepted proof.
Separate current/parent rustdoc JSON builds exist with distinct targets and hashes.
The tag locked failure remains the required gate result. A release API floor or
complete product/phase acceptance is not claimed.
