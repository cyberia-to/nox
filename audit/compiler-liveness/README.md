# Compiler allocation liveness census

Status: measured. This diagnostic establishes a reachable frontier at one
allocation failure; full compiler C2 production and self-hosting acceptance
remain open. It implements observation only, with production `R/nox` untouched.
Here `R` is `/Users/master/cyber/.worktrees/selfhost-0.4-full-bootstrap`.

## Measured result

The saved packed stage-four prefix reached `Error(Unavailable)` with
25,165,824 lifetime arena entries. Only 177,780 entries were reachable,
including every one of the 159,037 entries present before evaluation.
The remaining 24,988,044 entries (99.2936%) were unreachable at this frontier.
The live set beyond the conservative pinned prefix was 18,743 entries.

This is evidence of lifetime allocation pressure for this failure. It does not
establish the maximum live set during the run, the live set at later compiler
stages, or that a collector alone will complete the compiler. Garbage collection
has not been implemented. A separately bounded execution policy would still
need cumulative allocation and collection-work quotas, stable pinned Orders,
correct continuation remapping, cache invalidation, and valid arena state on
cancellation. Existing lifetime-allocation APIs retain their behavior.

| observation | value |
|---|---:|
| allocation count | 25,165,824 |
| reachable entries | 177,780 |
| unreachable entries | 24,988,044 |
| preloaded entries pinned | 159,037 |
| pending frames before error propagation | 832 |
| pending frame slice bytes | 146,432 |
| retry operand Orders | 25,165,823; 84,767 |
| marker bitset bytes | 3,145,728 |
| marker root snapshot bytes | 24 |
| marker traversal stack bytes | 0 |
| marker deadline polls | 6,144 |
| marker elapsed microseconds | 47,667 |

The retry operands come from a fixed pre-step snapshot because the failed
constructor's frame has already been popped when the error Action appears.
The remaining continuation frames retain only values consumed after a child
returns. Historical trace row references and the finalizer cache are excluded.
The marker checks every allocated pair has both children earlier in Order,
then propagates marks during a descending pass. Invalid roots or pair ordering
reject the census. The contract is [compiler-liveness-census.md](../../specs/compiler-liveness-census.md).

## Exact workload and accounting

The ignored test loads the same saved `packed-prefix.dag` and
`packed-closure-files-k_n8ecq7/job-visits.dag` as the baseline, wraps the subject
with stage `4`, and extracts the ART1 formula by the same path. The artifact and
input SHA-256 digests are in [run-inputs.json](run-inputs.json); baseline guest
program and subject particles remain in [baseline-prefix-bodies.json](baseline-prefix-bodies.json).
The test asserts the matching 159,037 preloaded entries.

Execution uses a 10,000,000,000 reduction budget, 25,165,824 logical entries,
33,554,432 physical entries, 65,536 maximum frames, the bounded cached NoTrace
API, and a cooperative 1,200-second deadline. The marker has its own deadline
polls, which do not increment evaluator checkpoints. Marker storage grows only
with the bounded existing arena and does not recurse or allocate a traversal
stack. Its scan performs one visit per allocated entry, with a deadline poll
at each 4,096-entry boundary. The marker observes the existing frame buffer.

The baseline and instrumented execution both returned `Error(Unavailable)`,
with exactly 1,091,810,645 evaluator checkpoints, 3,591 peak frames and
25,165,824 allocated entries. The instrumented evaluation took 212,046,510
microseconds, including the marker. The baseline took 224,830 milliseconds.
These are separate wall-clock observations with diagnostic and ambient load;
they are not a throughput comparison. Charged reductions remain null for this
failed execution, rather than being inferred from a propagated child outcome.

The existing reserved arena occupied 3,355,443,216 bytes, the reserved frame
buffer 11,534,336 bytes, and the per-run cache 1,048,576 bytes. The diagnostic
adds the bitset and fixed snapshot above; allocator metadata, the small receipt
string/path, input decode buffers and the thread stack are separate. The
runner requests a 268,435,456-byte stack and bounds each regular artifact read
at 16,777,216 bytes plus one byte to detect growth. Those are capacities, not
measurements of resident memory.

## Revision and reproduction

Run from `R/nox-live-probe`, an isolated worktree on
`chore/0.4-compiler-liveness-probe`, based on nox
`13b4c2ee71b3f3e205ddc9ad30d1476b86a7ccc5`. The measured patch is archived as
[implementation.patch](implementation.patch), SHA-256
`6e81375277d961cb13fbe07de53f69f2a6d32b9da8195a13c5a5f385e37d6de4`.
The final source differs only by Clippy's equivalent replacement of
`stack.len() * size_of::<Frame>()` with `size_of_val(stack)` when reporting
slice bytes. The measured binary/source hashes, sibling revisions, toolchain,
platform and working-tree state are in [run-inputs.json](run-inputs.json).
Production nox was clean at capture and was never modified by this diagnostic.
The saved guest artifacts, rather than subsequently edited compiler source,
are the inputs to this measurement.

[commands.sh](commands.sh) records the exact commands and environment paths.
Choose a fresh output path to rerun: the ignored test refuses to overwrite
existing census or execution receipts. Raw results are
[stage4-census.json](stage4-census.json),
[stage4-census.execution.json](stage4-census.execution.json), and
[stage4-run.log](stage4-run.log).

The three marker tests cover all continuation phases, dead trace fields,
failed-operation operands, 63 generated DAGs against an independent graph
walk, invalid roots, malformed pair ordering even in unreachable entries,
and deadline rejection. Default nox tests passed: 206 passed and one explicit
multi-gigabyte diagnostic ignored. The full diagnostic passed separately.
Scoped nox all-target Clippy passes with warnings denied. Workspace Clippy
also checked the CLI and found its pre-existing `cli/main.rs:247`
`clippy::needless_borrow`; that unrelated source remains untouched. Initial
and final validation logs are retained, including the diagnostic's corrected
`manual_slice_size_calculation` lint.
