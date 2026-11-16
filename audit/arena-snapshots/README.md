# Version-2 observer arena snapshots

This change adds an opt-in native observer profile which exports the complete
resident arena after each successful collection. It leaves version-1 events,
the version-1 observer API and encoding intact. Reset records bind the next
transition sequence and exact snapshot count; snapshots consume the existing
capture caps. The normative contract is
[sequential-compaction.md](../../specs/sequential-compaction.md#version-2-arena-snapshots).
These events are host observations. They supply no execution proof or SH7/SH8
closure.

The base is nox `172811b7746cdcd6ab198a3ed976dc6c19f55d5b`. Each command receipt
records that base plus exact changed-file hashes, clean pinned sibling revisions,
resolved source paths, toolchain executable hashes and versions, PATH and
Rust/Cargo environment, raw output hashes and sampled process-tree RSS. The final
source map is identical in `check-command.json`, `workspace-command.json`,
`all-features-command.json` and `root-review.json`. No sibling source changed.

## Validation

All commands ran with explicit rustup Rust 1.89.0, Cargo
`c24e1064277fe51ab72011e2612e556ac56addf7` and rustc
`29483883eed69d5fb4db01964cdf2af4d86e9cb2`, in the isolated observer worktree.
`run.py` retains the full arguments and environment, samples process-tree RSS
every 250 ms and terminates its own process group above 2 GiB or 128 MiB logs.
Those sampled memory values are observations, not exact peak-memory bounds.

| Command | Result | Sampled peak RSS, KiB |
|---|---|---:|
| `python3 audit/arena-snapshots/run.py focused 1` | 18 passed, zero warnings | 513536 |
| `python3 audit/arena-snapshots/run.py focused 2` | 21 passed, zero warnings | 336112 |
| `python3 audit/arena-snapshots/run.py check` | all-target release check passed, zero warnings | 471984 |
| `python3 audit/arena-snapshots/run.py workspace` | 244 passed, 2 ignored, zero warnings | 401664 |
| `python3 audit/arena-snapshots/run.py all-features` | 246 passed, 2 ignored, zero warnings | 419408 |

Focused runs preceded the final strengthening which checks every interning-index
entry after interrupted capture. Both full suites include that final check.
The two ignored tests are existing long-running diagnostics. Explicit-file
Rust 1.89 formatting, runner syntax and `git diff --check` also passed; see
`format-command.json`. Root independently reviewed production, specification,
positive and failure cases with no findings; `root-review.json` binds that review
to the final source hashes.

## Preserved version-1 stream

`golden-v1-command.json` records a separate probe built against the unchanged
base revision. Its exact source, manifest, lockfile and raw output are retained
alongside it. A 40-iteration native loop, with a resident allowance of 100,
completed two collections and emitted 1383 version-1 events, 341960 encoded bytes
and 5183 capture work units. Its evaluator/collection checkpoint counts were
1211/6; total allocations were 171. The regression test checks all those counters
and the Hemera digest of the entire ordered encoded stream against the probe.
These fixture numbers are compatibility observations, not compiler measurements.

The snapshot suite also compares version-2 execution against version1 and
unobserved execution, including physical statistics, outcomes, checkpoints,
fresh-node counts and logical transitions. It independently reconstructs each
snapshot's topological noun definitions and Cost metadata. Separate cases cover
exact caps, each one-below cap, event/byte/work/cancellation interruption inside
a snapshot, atomic sink failure at reset and every snapshot node, failed
collection and cancellation after collection commit. A fixture confirms that
original parent inputs can disappear from a later snapshot while their
continuations remain open. Retaining checked activation facts and rejecting
stale epoch handles remain responsibilities of the eventual proof verifier.

## Version-1 golden re-pinned under hemera 0.4 (2026-10-16)

hemera 0.4 (profile 15, `release/0.4` 7237a23) changes every particle
digest, so the version-1 golden of `rs/sequential/observe/tests/snapshots.rs`
was re-captured on nox `feat/hemera-0.4` with `cargo test --release
-p cyber-nox --lib version_one_gc_stream`:

| counter | base (hemera 0.3) | hemera 0.4 |
|---|---|---|
| events / encoded bytes / capture work | 1383 / 341960 / 5183 | unchanged |
| evaluator / collection checkpoints | 1211 / 6 | unchanged |
| total allocations, pinned, resident, peak, reclaimed, collections, scratch | 171, 32, 43, 100, 128, 2, 1424 | unchanged |
| collection_work | 133415 | 133417 |
| Hemera digest of the ordered encoded stream | `f7e00851…631b61` | `eca8db90…82a4` (bytes in the test) |

The fixture contains no hash pattern, so the event count and the stream
length are the same; the stream bytes and its digest change because the
particles inside the events are hemera digests. The collection meter moves
by two units: the dedup index rebuilt during collection probes by digest,
and the new digests take two more probe steps. The base probe's raw output
under hemera 0.3 is kept unchanged in `golden-v1-command.json`.
