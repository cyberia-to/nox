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
