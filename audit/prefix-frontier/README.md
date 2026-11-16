# Full-source prefix frontier

Admission through the first 21 dependency-ordered modules completed in
3,146,170,244 reductions.
The all-module-bodies checkpoint exhausted the fixed 10-billion reduction
profile with raw `Halt(38)` and no returned prefix words. For this frozen prefix,
the measured frontier is after that early module checkpoint and before all body
checking completes. This supports body checking as the old full C1 run's
bottleneck; the full C1 artifact is separate, so this is not a proof of equal
gas costs or identical execution traces between the two artifacts.

This audit locates the frontier reached by the saved compiler prefix artifact on
the exact full compiler JOB1 used by the failed C1(S) run. It is a pure nox
component diagnostic. It does not repeat Joy admission, emit a compiler artifact,
or establish C2 usability or full self-hosting.

Production code is pinned to `1eaa8a494f994e4da6b20509469c1e62624f3f2d`.
`test-helper.patch.gz` adds only a separate ignored measurement test and its module
registration. The existing discovery golden test is unchanged. `run-inputs.json`
records all input, helper source, test binary and toolchain identities. Each run
has a `stageN-command.json`, archived `stageN.log.gz`, and a `stageN.json`
result receipt.
Successful returns also have a canonical `stageN.dag`.

The artifact SHA-256 is
`12d6420e22d48a5ce6629ee8916f0af78e463be0ff6e7da3612ac5ae082b5c7d`.
The JOB1 SHA-256 is
`77ed0d6e18e956952717cfd357e9ebd07626e0fe00b3f9a61cdb63adab48705f`.
The input inventory is 94 modules and 369,707 source bytes, as recorded by the
parent's exact full-run receipt summarized in `full-run-baseline.json`.

All runs use 10,000,000,000 guest reductions, 1,000,000,000 cumulative fresh
allocations, 3,145,728 resident nodes, 4,194,304 physical slots, 65,536 frames,
10,000,000,000 collection work, and a cooperative 3,600-second execution deadline.
The helper decodes bounded regular files, wraps `[JOB1 stop-stage]`, extracts the
saved ART1 formula, and executes the production compacting pure VM. It supplies
no host language compiler stage. Stage 120 stops after dependency-ordered module
position 20; stage 4 stops after all module bodies.

Five-word successful prefix results have stage, index, error code, error start
and error end. For a successful per-module stop, index is the graph owner ID;
for a module error, it is the JOB1 index. They must not be interpreted as the
same namespace. The precise fixture source used for that interpretation is
retained as `saved-native_selfhost_prefix.tri` and included by identity in
`run-inputs.json`.

The stage-120 command returned `[120, 24, 0, 0, 0]`. Admission, discovery,
ordering and the first 21 dependency-ordered modules completed within the
unchanged budget. Its exact charged reductions were 3,146,170,244, with
6,853,829,756 remaining. VM elapsed time was 386,763,312 microseconds. It used
49,585,791 cumulative allocations, reclaimed 47,402,463 entries over 16
collections, and ended at 2,183,328 resident entries. Peak resident count was
3,145,728; peak frames were 5,098. Evaluator checkpoints were 2,694,541,915 and
collection checkpoints were 66,318. Full counters are in `stage120.json`.

Source analysis by the compiler investigation agent maps graph owner 24 to
`std.compiler.nox.emit`, JOB1 index 34. This is an inferred name mapping from
the frozen source import order, not an additional guest result. The observed
result establishes only the explicit stage-120 prefix. The mapping excerpt and
the original analysis script/result hashes are retained in
`source-inferred-positions.json`.

The stage-4 command returned `Halt(38)` after 1,212,909,016 microseconds of VM
execution. No five-word result or output DAG was produced. Remaining and charged
reductions are null; 38 is retained only as the raw propagated child Halt budget.
It used 122,047,449 cumulative allocations and reclaimed 119,505,312 entries over
41 collections. Final resident count was 2,542,137, peak resident count was
3,145,728, and peak frames were 5,098. Charged collection work was 1,001,323,059;
scratch was 29,360,128 bytes. Evaluator checkpoints were 8,243,400,319 and
collection checkpoints were 170,449. Full counters are in `stage4.json`.

The separate full C1 run in `full-run-baseline.json` also exhausted its budget;
its 8,243,399,959 evaluator checkpoints differ from this prefix by 360. This
close count is a measured comparison, not an assertion that its child Halt
budget or charged gas is known. Both report guest-budget termination; collection
completed within the other resource limits. This audit makes no claim about the later
optimized compiler snapshot and launches no additional long prefix runs.

Only a successful VM return has a root remaining budget and an exact charged
reduction count. A propagated Halt budget is a child budget, so failure cost
fields remain null. Checkpoint counts are measured VM callbacks, not gas.
The measurement test records expected guest failures without treating them as
successful prefix completion; its test status alone is not an acceptance gate.

The ordinary helper regression and full nox test suites pass with 221 default
and 220 parallel-feature tests, each with two intentionally ignored diagnostics
(`tests.log.gz`, `tests-parallel.log.gz`). Strict scoped all-target Clippy passes
(`clippy.log.gz`, `clippy-parallel.log.gz`). `checks-commands.json` records the full
test and parallel Clippy commands. They use
`CARGO_TARGET_DIR=../target-nox-frontier` and
`--release --locked --offline -p cyber-nox` from the nox worktree. Ordinary
tests ran during stage 120; parallel tests and Clippy ran during stage 4.
Elapsed times are therefore diagnostic observations, not controlled throughput
comparisons.

The new helper passes a scoped rustfmt check (`fmt-helper.log.gz`). The additional
whole-workspace `cargo fmt --all -- --check` fails on pre-existing untouched
formatting, beginning at `cli/main.rs:16`; the complete output is retained in
`fmt-all.log.gz`. No formatter changes were applied to those files.

Review after measurement hardened only the reusable test helper. New receipts
label the embedded historical revision as `measurement_reference_revision` and
require an external source/binary manifest for the executing runtime's identity.
Both DAG and JSON writes use exclusive creation, preventing a file created
during execution from being overwritten. Focused regressions cover the metadata
label and preservation of an existing output. The measured stage receipts and
the decompressed `test-helper.patch.gz` contents are unchanged.
`measured-frontier.rs` retains the exact source
used for measurement; `measured-binary.json` identifies a retained binary copy.
`post-measure-helper.patch.gz` records the subsequent helper-only changes. No long
prefix was rerun after these changes, and no production runtime source changed.

Archived logs and patches use deterministic gzip with timestamp zero and an
empty filename. `archived-evidence.json` records their exact original byte counts
and SHA-256 digests, plus the compressed identities. No archived text was
normalized. Command receipts keep their original output path and add the archive
path. `archive-verification.json` records verification of every archive and
byte-for-byte reconstruction of both helper versions by applying the patches in
an isolated temporary checkout. For example, inspect a final test log with:

```sh
gzip -dc audit/prefix-frontier/tests-final.log.gz
```

To reconstruct the measured helper in a clean checkout of the pinned production
revision, apply the first patch. The second patch reconstructs the final reviewed
helper from the measured version:

```sh
gzip -dc audit/prefix-frontier/test-helper.patch.gz | git apply
gzip -dc audit/prefix-frontier/post-measure-helper.patch.gz | git apply
```

Final helper checks pass with 223 default and 222 parallel-feature tests, each
with two ignored diagnostics. Both strict scoped all-target Clippy checks and
the helper rustfmt check pass. Their exact commands and logs use the
`post_measure_checks` entries in `checks-commands.json` and `*-final.log.gz` files.
