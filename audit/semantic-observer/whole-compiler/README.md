# Whole compiler compatibility with NoObserver

One actual C2(S1) → C3 execution passed through Joy's ordinary NoObserver
compacting path on 2026-09-28. The input is the actual C2 from Trident native CI
run `36359020560`, artifact `10948746458`, macOS arm64 repetition 1. Its 9,691,488
bytes and the returned C3 are exactly equal, with SHA-256
`76a07c08265bd2ef525164472b6b53ac3f0e6cbbedce3250c4202f40ffba34c8` and particle
`2eea2ac5f611012877b4e7291a3a6f534aee7281bb358a0b8e5fabe2ac1f9fbe`.
This local compatibility result supplements the earlier small CLI case. Observer
capture remains disabled; SH6 matrix acceptance and execution proofs remain
separate gates.

The installed Joy SHA-256 is
`853806f2bb0aba35c2872820597192216a20b05ee674c4096a3a525648e518eb`.
Its inputs are nox `71b5860219f0e0810200946e4445e5fdf54269dd`, Joy
`ec83bd8d85b20a8bd20d2d14b0f25aab0f75e9fe` and Trident
`c17bd0371c11746f46e20222c48cae2ab08be79d`; `launch.json.gz` records all nine
exact sibling revisions and clean Git states before and after the run.
The existing [installation evidence](../joy-integration/README.md) records the
clean build and downstream tests. Binary and compiler start/end hashes match.

The freshly generated inventory is exactly the frozen S1 inventory, SHA-256
`d35d263c7f9f27cbe7ea760a34393105ae8140dc6ab84d484151b6fe04960571`:
94 modules, 484 functions and 370,544 source bytes. Every source equals both
its clean Trident Git blob and the frozen S1 source. The inventory helper was
built with `--release --locked --offline` in `target-observer-joy`, with zero
Rust warnings. The supplied C2 executes the complete source package.

The probe argv, timestamps, stdout, stderr and exit statuses are retained in
`launch.json.gz` and `c2-to-c3.json.gz`. The one probe returned exit 0; no retry
or limit change occurred. It used `--emit program` and these fixed bounds:

| Resource | Bound |
|---|---:|
| Reductions | 20,000,000,000 |
| Cumulative allocations | 1,000,000,000 |
| Resident nodes | 3,145,728 |
| Collection work | 10,000,000,000 |
| Evaluator frames | 65,536 |
| Host deadline, milliseconds | 7,200,000 |
| Validation visits | 16,777,216 |

`verification.json.gz` records the unchanged Trident
`check-selfhost-fixed-point.py::step()` guard applied to this single producer.
It checks the saved inventory and source copies, then canonically repacks JOB1
with the pinned Joy and compares actual JOB1 bytes. It also validates producer
commands, admission, publication, compiler identity and result bindings. This
uses the existing per-producer guard, without inventing a second compilation
under the new runtime or a new two-stage fixed-point receipt.

`verify.py` independently binds the original GitHub ZIP's size and API digest,
its platform receipt, file manifest, nested C2 producer and C2 bytes. The CLI
guide's `producer_receipt_sha256` names the platform receipt; the nested
`c2-step.json` has its own separately checked identity. Raw receipts keep their
original paths and contents. The original CI ZIP remains at
`measurements/ci-split-readonly/artifact-10948746458.zip`; the Trident platform
store separately retains its exact raw tree and ZIP identity. This archive
includes the relevant raw receipts and API metadata.

All fields of the actual execution report equal the frozen S1 C2 → C3 report
after excluding only `elapsed_micros`, including every compiler-job field,
particle, physical report and checkpoint count:

| Observation | Both executions |
|---|---:|
| Charged reductions | 9,777,538,159 |
| Allocated nodes | 162,296,944 |
| Collections | 56 |
| Collection work | 1,370,915,937 |
| Evaluator checkpoints | 8,427,356,821 |
| Collection checkpoints | 233,603 |
| Peak frames | 9,254 |
| Peak resident nodes | 3,145,728 |

JOB1, manifest, admitted package and emitted artifact also match exactly.
The comparison baseline is the retained local S1 receipt
`measurements/lexer-v9-c2-to-c3.json`, with Joy SHA-256
`506f665b0567cf8d7d669f152153b72dbbbd4520e926a4f47955d2f0bef487b8`.
Its host deadline was 3,600,000 ms; the current explicit deadline is 7,200,000 ms.
Every other host bound and every JOB1 limit matches. Reported execution time
is 1,180,982,939 microseconds for this run and 1,251,723,053 for the historical
baseline. These are individual local observations, without a speedup claim.

`driver-commands.json` records the exact driver shell commands, working
directories and exit statuses. The following is their equivalent spelling
relative to `/Users/master/cyber/.worktrees/selfhost-0.4-full-bootstrap`:

```sh
python3 measurements/observer-whole-compiler/run.py > measurements/observer-whole-compiler/launcher.stdout 2> measurements/observer-whole-compiler/launcher.stderr
python3 measurements/observer-whole-compiler/verify.py > measurements/observer-whole-compiler/verification.stdout 2> measurements/observer-whole-compiler/verification.stderr
python3 measurements/observer-whole-compiler/archive.py
```

`run.py` and `verify.py` are retained here for review. Their original copies,
the archival script, complete producer directory, exact compiler/JOB1/C3 bytes,
source snapshots, raw logs and reference receipts are in `raw-evidence.tar.gz`.
`files.json` binds every member's raw size/SHA-256 and the compressed archive.
All 217 file members, totaling 29,431,409 bytes, were compared byte-for-byte to
their originals after compression. Only tar headers are normalized; file
contents, including empty logs and trailing whitespace, are unchanged. The
archive is 12,769,325 bytes. Host `ps` samples are retained as host telemetry;
they do not establish guest progress or execution resource claims.
