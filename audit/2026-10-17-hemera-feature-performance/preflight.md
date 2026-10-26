# Matched Hemera feature performance — prepared, timing held

The source/tool/build preflight passes. No benchmark has executed in this
investigation. The accepted finite schedule remains pending root review of the
freeze and an explicitly coordinated quiet window. Active unrelated compute
currently prevents that window. The historical initial 5/9 and repeat 6/9 red
threshold crossings remain red/unverified.

The accepted [plan](../../.claude/plans/hemera-feature-performance.md) was committed
as `edba749`; owning compiler inputs remain fixed at
`64d091e5ad3f277cce77b88ae0de6eea4c49b4df`. No owning production, dependency,
lockfile, version or benchmark source was edited. All commands below take the
owning repository argument. Their complete argv, paths, hashes and streams are
retained beside this report.

## Inputs and builds

`nu prepare-sources.nu REPO CACHE` reproduces the exact historical capsules,
accepted patch and current committed exports in [sources.json](sources.json).
A uses Nox `0b1407a4…`, Hemera `d5f0a095…`, Strata `494df7a0…`, Lens
`8b26cef6…`; B differs only by the accepted two-file patch. C/D use Nox
`64d091e5…`, Hemera `1c385e54…`, Strata `56aedb2d…` and the same Lens revision.
C's exported manifest reverses only the two feature edits; D retains the shipped
policy. P has A's exact bodies and modes at a different same-depth cache path.

`nu prepare-registry.nu REPO CACHE CARGO_CACHE` verifies all 72 locked package
archives against lock checksums, extracts every member and freezes the directory
source checksums. [registry.json](registry.json) records the sole configuration
change from the original registry setup: its cache-local path relocation.

[build.nu](build.nu) and [finish-builds.nu](finish-builds.nu) build each cell
exactly once using the original locked/offline benchmark command with `--no-run`,
`-vv` and Cargo JSON output. `env -i` applies the recorded allowlist; rustup
1.95.0, absolute `RUSTC`/`RUSTDOC`, two jobs, no incremental compilation and empty
Rust flags are identical across cells. Target identity is `aarch64-apple-darwin`.
[tools.json](tools.json) records executable/sysroot hashes, compiler/linker/SDK,
OS and hardware observations. Full Xcode is unavailable; the recorded command
line tools and SDK are available. The historical host is not reconstructed.

[builds.json](builds.json) retains all five successful build receipts and binary
hashes. Every binary is 3,044,656 bytes. Feature trees and actual rustc cfg/command
lines show inherited Hemera `default,std` in A/C/P and their absence in B/D.
No timed invocation is allowed to compile or change its frozen binary.

## Source and code bridges

`nu summarize-preflight.nu REPO` at the source revisions above produces
[preflight-summary.json](preflight-summary.json). B/D changes 45 Nox Rust files
and two Strata Rust files. Hemera and Lens compiler-source bytes match between
eras. The benchmark's [complete diff](benchmark-era.patch) is formatting:
pinned rustfmt on historical bytes produces the exact current bytes, recorded
in [benchmark-format.json](benchmark-format.json). Strata's `algebra_impl.rs`
and `proptests.rs` differ materially, including the `Dot` implementation. B/D
therefore remains a multi-source comparison; this investigation repairs none
of those source changes.

`nu inspect.nu REPO CACHE` compares all 17 Mach-O sections and all 3,949
`LC_FUNCTION_STARTS` spans per binary. [code.json](code.json) binds the exact
raw sections, complete disassembly and explicit normalized view to each binary.
Span lengths include any padding up to the next start; semantic function sizes
and complete reachable call boundaries remain unknown. Cargo's raw rustc/link
driver commands are retained; child linker argv was not instrumented.

| Contrast | Label-selected spans | Unique name matches | Exact raw span matches | Moved spans |
| --- | ---: | ---: | ---: | ---: |
| A/B, historical feature policy | 49 | 47 | 35 | 5 |
| C/D, current feature policy | 49 | 47 | 16 | 29 |
| A/P, identical-source path control | 49 | 47 | 44 | 2 |
| B/D, source-era bridge | 49 | 47 | 22 | 26 |

Each contrast changes raw `__text`, constants and unwind-related sections,
including A/P. Cache-path/layout effects therefore exist in this build setup.
Whole-file inequality or selected span equality cannot isolate a feature effect,
prove complete equivalence or establish the cause of the old performance reds.

## Portable replay and timing boundary

[inputs.tar.gz](inputs.tar.gz), indexed by [inputs.json](inputs.json), retains
the exact source archives, treatment bodies, registry archives/checksums, five
binaries, every raw section and helper binaries. Its SHA256 is
`da606913e228eacd0687d3691529ee231d7e3b72327e07020fa2facbf5e75231`.
The read-only verifier extracts its own temporary directory and checks the
archived bytes without executing archived helper binaries or historical Ruby:

```sh
nu audit/2026-10-17-hemera-feature-performance/verify.nu REPO
```

`--live` additionally checks the original cache and current tool executables.
[preflight.json](preflight.json) records a successful portable-plus-live replay;
the final freeze also records the exact verifier inputs for its final replay.
The named thumb gate passes after the pinned-path correction. Its initial
missing-core red, the already-installed target observation, all preparation
failures and successful rerun remain in [preparation.md](preparation.md).
Broad hygiene tests were bridged by exact compiler-input bytes, not rerun.

[timing.nu](timing.nu) requires a separate root authorization bound to the exact
freeze and approving a quiet window. It permits exactly
`A1 B1 B2 A2 C1 D1 D2 C2`, a 60-second idle interval, then
`D3 C3 C4 D4 B3 A3 A4 B4`. Every run preserves all nine complete Criterion
directories, raw streams, median 95% intervals and a fixed five-second sampler.
Missing or stale cases, rebuilds or source/tool/binary drift stop the schedule
without replacement. No CPU affinity or frequency control is claimed.

After that schedule, `verify.nu REPO --timed` computes every individual
comparison from `estimates.json`: 36 per source era and 72 same-binary temporal
case comparisons. The original rule is unchanged: candidate/parent median
greater than 1.05, or candidate lower median bound above parent upper bound.
Independent telemetry review is still required to assess host interference.
A green future contrast cannot erase the old reds or identify their cause.
