# Bounded sequential compiler execution

Status: 0.4 implementation contract. `sequential::reduce` executes native pure
L1 with a bounded heap stack and no Rust recursion. It accepts a Reduction,
object, formula, budget, Limits { max_frames }, and a Tracer. It returns
Execution { outcome, peak_frames } or a profile/resource Error. The original
recursive APIs remain unchanged.

`max_frames` counts active invocations including the root. Zero rejects every
execution. Pending parents occupy explicit frames until their postorder rows
are emitted; continuation reuse is not constant-space tail-call elimination.
Frame storage is reserved once with fallible allocation before execution.
`frame_storage_bytes` reports the requested buffer size, excluding the small
Vec/engine metadata, allocator overhead and arena. Heap allocation or frame
excess aborts with no successful outcome; partial traces are not proofs.

`reduce_controlled` additionally accepts a host cancellation predicate, checked
before buffer allocation and between bounded Enter/Return steps. True aborts
with Cancelled, including while unwinding pending frames. The predicate receives
no VM values and must only implement resource supervision, never guest work or
witnesses. It is cooperative; it cannot preempt an allocator, tracer callback
or an individual pattern finalizer. The plain entrypoint never cancels.

Tags0..15 retain sequential recursive semantics, dispatch costs, binary
left-before-right evaluation, after-both-children type checks, bound reservation
and success refunds, chosen-arm evaluation, native dynamic compose and exact
postorder trace columns. Error paths keep existing partial multirow behavior.
Bounds are upper bounds; a branch may run below its worst arm's bound. Failed
child outcomes propagate without fabricated reservation refunds.

Reached tag16/17 returns UnsupportedService before dispatch charging or child
execution. Those numbers are allowed inside quoted data. This entrypoint has
no registry/provider argument, never invokes jets or the parallel evaluator,
and remains sequential when the optional parallel feature is enabled.
Unknown numeric tags retain ordinary nox Malformed behavior after default cost;
malformed formula/tag dispatch precedes the budget guard, as in reduce.

The same arena node allowance covers loading and execution. Tracers own their
storage policy: NoTrace retains no rows; VecTrace is opt-in. Successful cost is
initial minus remaining budget. Error outcomes have no precise remaining
budget; do not derive failed cost from row count. This executor does not claim
Zheng production coverage for dynamic applications or compiler workloads.

Acceptance: differential small pure formulas compare every trace column,
result/budget and node allocation order against the recursive sequential path;
failures include malformed/type/word/inverse/budget/allocation cases. A compact
runtime loop exceeds4097 iterations without formula expansion. Exact frame
bounds and unsupported reached services fail explicitly. Optional parallel
builds must pass the new entrypoint's deterministic fixtures independently.

## Cached trace-free finalizers

`reduce_cached` and `reduce_cached_controlled` provide pure L1 execution without
a tracer argument. They share dispatch, budget partitioning, continuations and
pattern finalizers with the existing entrypoints. Their execution-local cache
keys successful finalizers by tag and evaluated operand Orders. Tags 3 and 5..15
use this cache; quote, axis, branch and dynamic compose retain their ordinary
steps. A hit returns the saved Order and the current post-charge budget.
Every child is evaluated and every dispatch cost is charged normally.

Each invocation reserves a fixed 65,536-entry direct-map cache, with 16 bytes per
entry. `finalizer_cache_storage_bytes` reports this 1,048,576-byte buffer;
allocator and Vec metadata are additional. Allocation is fallible and returns
Allocation on failure. Deterministic collision replacement affects performance
only. A cache exists solely during one exclusive Reduction borrow. It is
discarded on success, failure or cancellation; no cached Order crosses arenas
or separate executions. Only successful finalizers populate it.

Pure finalizers depend on their tag and evaluated operands. Successful
finalizers allocate only immutable hash-consed arena nodes. Repeating one in
the same invocation therefore finds every node already present, even when the
arena is at its allocation limit. Reusing that result preserves node allocation
order and admission behavior. Failed finalizers always execute ordinarily,
including their partial allocations. Service rejection and all child failures
occur before the cache is consulted. The existing traced entrypoints retain
their complete trace behavior and allocate no cache.

Semantic results, exact charged reductions, reached-service rejection, frame
storage and peak, arena allocation order, and cancellation predicate invocation
order match `reduce` and `reduce_controlled` with `NoTrace`, except for the
additional explicitly bounded fallible cache allocation. Cancellation is
checked before allocations and every Enter/Return step, including steps with
a cache hit. Finalizers contain no internal cancellation checkpoints.

A subtree result cache keyed only by object and formula is outside this
contract: replay would require accounting for budget, peak frames, allocation
history and skipped cancellation checkpoints. Successful gas cost alone does
not establish that equivalence.
