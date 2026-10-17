# Matched Hemera feature-performance investigation

Status: accepted by root on 2026-10-17; tool/source freeze requires review before timing. Owner:
`/root/nox_perf_finish`; checkout `nox/.worktrees/no-std-dependencies/nox`,
branch `research/hemera-feature-performance`, base
`64d091e5ad3f277cce77b88ae0de6eea4c49b4df`. Preserve the wrapper's sibling pins.
Scope: one finite investigation, retained in
`audit/2026-10-17-hemera-feature-performance/`; production, dependencies, locks,
benchmark source, historical evidence, versions and release actions stay unchanged.
Source exports/targets live in `~/.cache/cyber/nox/hemera-feature-performance/`.

## Question and fixed evidence

Does the Hemera default-feature opt-out cause a repeatable slowdown under matched
conditions, and does that effect survive at current committed sources? Distinguish
feature selection, emitted code/layout, source-era changes and host/order effects.
The original [audit](../../audit/2026-10-13-hemera-no-std/README.md) remains red:
records 49/83 crossed 5/9 thresholds; the predeclared repeat 102/103 crossed 6/9.
The successful portable Nu replay verifies those observations, not their cause.
Both old pairs ran parent before candidate. Old load/frequency/thermal telemetry
and initial binary hashes are absent; later equal sizes/different hashes do not
establish equivalent executable code. Historical Ruby stays inert evidence.

Preserve all nine cases and their original Criterion input/work counts:
`pattern/add`, `pattern/mul`, `pattern/inv`, `hash/pattern/hash`,
`hash/pattern/hash_chain_64`, `pattern/look_null`, `jet/merkle_verify`,
`encode/tree_64`, `encode/particle_atom`. Every comparison uses the original rule:
red if candidate median / parent median > 1.05 OR candidate median 95% lower bound
exceeds parent median 95% upper bound. Use `estimates.json` median fields, never
the printed slope interval or Criterion's implicit previous-run `change:` line.

## Four source cells; freeze before execution

| Cell | Nox and siblings | Feature treatment |
| --- | --- | --- |
| A | Exact historical source capsules | Original inherited Hemera defaults |
| B | Same historical capsules | Exact accepted two-file patch from old audit |
| C | Current pinned source exports | Audit-only inverse of the two manifest feature edits |
| D | Same current exports | Unmodified shipped feature policy |

A's Nox is `0b1407a4b855e4961ede07e1b7ea199cb8018c30`, Hemera
`d5f0a09539360934fb32db9b2e15a91995607db6`, Strata
`494df7a0973ae10a6cc1c2022e644d70449a29d6`, Lens
`8b26cef6587bee6a686f2cf163f6a8496931ee6a`. B uses old patch SHA256
`387f9ccee4480bbfeb957ee9a2cbb7d959ac808db5635a3f296ee51b001f3899`.
C/D use the base above, Hemera `1c385e54e18885075f0dd12a0407ad0234e1bebf`,
Strata `56aedb2d12b3126c601eb333419136d403614dbb`, and unchanged Lens.
C changes only `std = []` and removal of Hemera `default-features = false` in
its exported `rs/Cargo.toml`; it is an experimental control, never a source fix.

Verify original capsule/index hashes with the existing Nu verifier; reconstruct
full body/mode inventories and exact B bridge. Export C/D by committed Git object,
freeze complete compiler inputs, locks, registry checksums and source/config diffs.
Retain each era's own benchmark bytes: the current hygiene commit formatted that
file; compare its full diff explicitly. A/B and C/D are matched causal contrasts;
B/D is a multi-source bridge and cannot isolate any one later source change.
No current sibling may silently replace an archived historical sibling. Missing
archives/toolchain/locked packages are an explicit prerequisite stop. Root permits
installation of the exact `thumbv7em-none-eabi` component for toolchain 1.95.0 as
environment preparation; retain any missing-target red and a distinct rerun epoch.
No resolver, source or lock repair is part of this investigation.

## Build and executable inspection, before the quiet window

Pin rustup Cargo/Rust 1.95.0, target `aarch64-apple-darwin`, two build jobs,
incremental off, empty RUSTFLAGS/RUSTDOCFLAGS, offline checksum-verified registry,
separate actual target per cell. Capture executable/tool hashes, `rustc -Vv`,
Cargo, linker/Xcode/SDK, OS and hardware identities. A clean allowlisted PATH
replaces the historical agent PATH; record this and cache/source path relocation
as environment bridges. Do not claim reconstruction of the unrecorded old host.
Pin `RUSTC` and `RUSTDOC` to the absolute 1.95.0 toolchain executables for every
cell build and timing invocation; record relevant inherited tool overrides.

Build each reduction binary once with the locked original graph and
`rustup run 1.95.0 cargo bench --manifest-path CELL/nox/Cargo.toml -p cyber-nox
--bench reduction --locked --offline --no-run -vv --message-format=json`.
Resolve rustup to its recorded executable explicitly. This wrapper replaces the
historical `cargo +1.95.0` dispatch while preserving the benchmark subcommand and
flags; actual toolchain Cargo does not accept rustup's `+toolchain` shorthand.
Retain rustc/link commands, inverse Hemera feature trees and all cfg selections.
Freeze each binary hash before timing; successful timed commands must do no build.
Compare exact Mach-O sections and symbol sizes/disassembly for benchmark closures,
Nox reduction/encoding, reachable Hemera/NeBu kernels and relevant static data.
Keep raw output; any address/symbol normalization is a second, explicit view.
Whole-file inequality, symbol renaming or matching selected functions alone
cannot prove a runtime cause or complete equivalence.

One additional A-identical export at a different same-depth cache path is built
once for an unmeasured build-path control. Compare its raw sections and code with
A before attributing A/B differences to features. No new optimization flags,
LTO/target-cpu changes, source tuning or binary patching. Record unknown/stripped
symbol boundaries rather than fabricating a complete call-graph proof.

## Predeclared timing protocol

Coordinate one quiet window with root and active agents after all compilation.
Record power source/mode, process CPU/load and available thermal status before and
after each invocation; use one fixed lightweight sampler throughout all cells.
Unavailable telemetry stays unavailable. Do not claim CPU affinity or frequency
control. No agent builds/tests/benchmarks concurrently; ordinary system activity
is recorded. Unexpected sustained competing compute or a tool/source drift makes
that epoch indeterminate, retained without replacement under this plan.

Every invocation retains the original benchmark subcommand/flags, with recorded
path relocation and the explicit rustup wrapper above:
`rustup run 1.95.0 cargo bench --manifest-path CELL/nox/Cargo.toml -p cyber-nox
--bench reduction --locked --offline -- --warm-up-time 1 --measurement-time 2
--sample-size 20 --noplot`. No filter, new baseline flag or changed duration.
Archive all nine complete Criterion directories and stdout/stderr immediately
after each run so later runs cannot overwrite evidence. Freeze runner, schedule,
source/tool/environment manifests before the first timed invocation.

Execute exactly two blocks, separated by a fixed 60-second idle interval:

1. `A1 B1 B2 A2 C1 D1 D2 C2`
2. `D3 C3 C4 D4 B3 A3 A4 B4`

Thus 16 complete nine-case runs, four samples per cell, balanced order and era.
Within each era report all four matched `Bi/Ai` or `Di/Ci` contrasts using the
unchanged threshold. Report every same-binary within-block comparison (`X2/X1`,
`X4/X3`) with direction, median, interval and threshold result as a temporal
control. These are process runs containing Criterion samples, not independent
machines; do not pool iterations to inflate replication. Descriptive aggregates
may accompany all individual comparisons, never replace a failed pair.

## Decision and stopping rule

End after the scheduled runs, including all reds; no repeat-until-green or
result-dependent configuration selection. A current non-regression statement
requires all nine cases below the original threshold in all four C/D contrasts,
with intact inputs and usable temporal controls. Conflicting pairs, threshold
crossings in same-binary controls, missing prerequisites or host drift leave the
performance gate red/unverified. A reproducible slower feature contrast with
localized code/dispatch differences motivates a separately reviewed repair;
matching code plus variable controls establishes only the measured variation,
not the historical cause. A green current contrast never erases the old reds.

Deliver compact findings, source/feature/code bridges, exact commands and all raw
samples; a read-only Nu verifier checks inventories, binaries, schedule, median
arithmetic and old evidence identities. Independent review precedes root's
ordinary evidence commit. This unit closes only the investigation it can support;
publication, API/product gates and Kadek dependency delivery retain their own
acceptance conditions. Further experiments require a new predeclared plan.
