# Bounded finalizer cache review

This is local development evidence against nox revision
`c9f7486a74fe81bfc194b598da40f6343ecb2ef1`. The changed source hashes and
verification commands are recorded in `validation.json`. The contract was
written in `specs/sequential.md` before the implementation.

## Profile and selected scope

`baseline-run.json` records the exact installed Joy command, source receipt,
binary/compiler/job hashes, and observed budget exhaustion. With the sampler
attached, that command took 63.687 seconds; it did not finish compilation.
`baseline-command.json` records the sampled process. The command
`/usr/bin/sample <pid> 8 1 -file audit/sequential-untraced/baseline-sample.txt`
produced `baseline-sample.txt` on that revision. Sampling covers the beginning
of the run, including loading, rather than the complete execution.

The sampled evaluator had 4,905 observations. Of those, 3,211 were beneath
`Reduction::pair` and its Hemera hashing, and 40 were in the nebu field inverse
used by witness arithmetic. The profile therefore selected repeated successful
finalizers for caching. A witness-only optimization would address a small
fraction of this sample. Hemera hashing remains authoritative on cache misses.

Whole-subtree memoization was rejected for this change because it would skip
guest continuations and cancellation checkpoints. The implemented cache wraps
only the existing pure finalizers; its key consists of the tag and evaluated
operand Orders. It reserves the fixed buffer specified by
`finalizer_cache_storage_bytes`, records only successful results, and belongs
to one evaluator invocation. No host compiler, jet, provider or guest-language
recognition was introduced.

## Independent correctness review

Can a hit bypass arena admission? Every successful finalizer allocates immutable
hash-consed nodes. Within the same exclusive arena borrow, repeating that
finalizer finds the same nodes already present, including at the allocation
limit. Misses and failures still call the original finalizer. Tests compare
every arena entry after successes, failures and cancellation.

Can a hit change budget reservation or failure precedence? Both children finish
before lookup, and dispatch charging plus reservation/refund calculations stay
in the shared evaluator. A hit uses the current remaining budget. Type errors,
inverse-zero errors, malformed input, unavailable storage and reached services
are compared against the original executor at budget boundaries.

Can a hit hide a frame excess or deadline? The shared Enter/Return loop and frame
buffer are unchanged. Finalizers have no internal cancellation checkpoints.
Tests compare exact callback counts and partial arenas at each tested
cancellation threshold and frame allowance. The additional fixed cache
allocation is fallible and explicitly exposed in the API memory report.

Can stale Orders or collisions change results? Cache ownership ends with the
Reduction borrow. Tests repeat executions in a shared arena, then use fresh
arenas with overlapping Order numbers. Pressure beyond cache capacity forces
collisions, and every lookup checks the full tag and operand key.

Does tracing change? The original generic entrypoints instantiate the shared
engine with caching disabled, allocate no cache, and continue to call every
original finalizer. The recursive differential suite compares all postorder
trace columns, outcomes and arena entries against the sequential executor;
the cached execution is an additional independent comparison.

## Mechanical lint cleanup

The all-target strict Clippy command initially found existing warnings in the
owned branch dispatch and three test expressions outside the initial scope.
The owner explicitly authorized the latter cleanup. The hash test now searches
its double-ended iterator from the back; the two field tests use `div_ceil(2)`
for the existing odd modulus. These test-only changes are a separate logical
change in `rs/patterns/hash.rs`, `rs/jets/formulas.rs` and
`rs/jets/fri_fold.rs`; they do not alter runtime semantics.

## Verification and measured compiler diagnostic

`validation.json` records the exact commands and source hashes for the final
checks: the default suite passed 203 tests, the parallel-feature suite passed
202 tests, and all-target Clippy passed with warnings denied. The default suite
includes the existing recursive differential corpus plus cached comparisons;
the parallel suite independently covers cached behavior, errors and limits.

`runtime-comparison.json` records the probe commands, source revisions and
binary/input hashes for `runtime-before.json` and `runtime-cached.json`.
The exact same compiler artifact and job exhausted their budget in 57,437 ms
uncached and 16,090 ms cached, a 3.57x improvement in this run. Both used
99,999,945 charged reductions with 55 remaining, allocated 2,069,519 arena
nodes, peaked at 2,087 active frames and invoked the cancellation predicate
89,547,033 times. Loading is excluded from those evaluator times. This is one
local paired observation, not a statistical throughput claim.

## Limits

Cache hit rates depend on the executed workload. The measured probe bypasses
Joy admission and returns a budget halt; it establishes runtime acceleration
with matching resource observations, not a usable next-generation compiler or
full self-hosting. The full compiler's installed acceptance is recorded by the
owning cross-repository run. This cache preserves the charged guest work and
does not raise the compiler's reduction budget.
