# Bounded semantic observer

Local implementation based on nox `f8047c22cc6075d5171c0fbe520174e78e94ccb6`,
on `feat/0.4-semantic-observer`. Test execution used the uncommitted source
identified by every Rust source hash in [receipt.json](receipt.json). The
receipt preserves actual commands, dependency revisions, raw log hashes and
losslessly compressed logs. This evidence covers witness capture; it supplies
no Zheng proof, Joy proof dispatch, semantic-preservation result or SH7 closure.

The implementation adds an opt-in observer around existing compacting steps.
Events contain owned particles, node definitions and phase-specific live stack
changes. Capture uses constant temporary storage. Event, encoded-byte and work
caps are separate from guest/runtime limits; byte accounting measures canonical
encoding, not sink allocation or RSS. Legacy wrappers select NoObserver and
retain Frame layout, existing limits and physical counters. See the
[owner contract](../../specs/sequential-compaction.md#logical-observation).

Commands ran from this worktree with `CARGO_TARGET_DIR=../target-observer`:

```sh
cargo test --workspace --release --locked --offline
cargo test --workspace --features parallel --release --locked --offline
cargo check --workspace --all-targets --locked --offline
cargo check -p cyber-nox --no-default-features --locked --offline
```

Default workspace: 234 passed, 2 ignored. Parallel-feature workspace: 233 passed,
2 ignored. Both complete test runs and both checks emitted zero Rust warnings.
The eleven new tests cover computed continuation and differing result topology
at budgets 7/8/9; four repeated hash finalizers at budgets 106/107/108; repeated
GC with complete logical-transition comparison against a large arena; all 336
pure-pattern/value/budget vectors; exact capture caps and one-below rejection;
every sink rejection boundary; cancellation before Completed and during initial
export; existing host errors; malformed exported references; and the 456-byte
maximum public event encoding.

An initial focused run failed a test-only assertion that the final loop result
Order must move. Its result is an atom pinned at entry. The corrected test
checks actual relocation of surviving fresh allocations while comparing result
particles and the complete logical stream. The red log remains under `logs/`.
The corrected focused run preceded addition of the all-pattern differential
test; both later full gates include all eleven tests.

[benchmark.json](benchmark.json) retains paired commands and nine alternating
samples per variant for the identical [benchmark.rs](benchmark.rs) consumer.
Each sample performs thirty ordinary compacting executions of a 3,000-iteration
loop, with identical budget/resource limits. The baseline reads the untouched
`f8047c22` worktree; the candidate reads the captured implementation. Median
elapsed time was 2,568,607,209 ns baseline and 2,579,099,625 ns candidate, ratio
1.0041. This short single-host measurement showed no material default-path
regression; it establishes neither whole-compiler throughput nor proof cost.
Both benchmark builds emitted zero warnings. Binary identities and temporary
consumer manifests are retained in the benchmark receipt.

Read-only review checked live roots against `compacting/roots.rs`, one-pop/
one-push behavior against dispatch, cache-hit event coverage, cancellation and
the public encoder's worst case. Cached hashes and Cost metadata still require
independent authentication/derivation by a future verifier. Physical GC work
and allocation counters remain host telemetry.
