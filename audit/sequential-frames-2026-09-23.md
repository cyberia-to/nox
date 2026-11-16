# Bounded sequential evaluation — 2026-09-23

`sequential::reduce` removes host recursion from pure L1 execution. The same
compact loop body executes4097 and5000 iterations, returning4097/5000 and
charging61460/75005 reductions. VecTrace and NoTrace agree; rerunning with the
observed exact frame cap succeeds. Root/leaf boundaries and host cancellation
fail explicitly. Reached call/look services are rejected, while quoted data is
unrestricted. Pending parents still consume bounded heap frames.

4185 deterministic small-program configurations compare recursive and heap
execution: semantic outcomes, budgets, every trace column and every allocated
node. Coverage includes all pure tags, weighted costs, wrong types, invalid
word widths, inverse zero, malformed formulas, selected branches, dynamic
compose, partial allocation and insufficient budget. The old recursive path
is an oracle only without the parallel feature.

Default workspace:192 tests passed. Parallel-feature library:191 tests passed;
its forced-sequential fixtures and long loops pass independently of the legacy
parallel path. Differential cases are intentionally excluded in that feature
build because it is not the same sequential oracle. All-target and no_std
checks passed. [Commands, hashes and assertions](sequential-frames-2026-09-23.json).

The read-only code review found no correctness blocker. A cooperative resource
guard was then added and tested for cancellation before work and between
continuations. It cannot preempt individual allocator/tracer/pattern operations.

This does not implement source-language loops/calls, compiler jobs or extend
Zheng's proof relation. Tracer memory, host supervision and worker admission
remain separate policies. Versions and master stay unchanged.
