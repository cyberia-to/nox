# Bounded compacting NoTrace execution

Status: implemented; opt-in component profile.

`sequential::reduce_compacting_cached` and its `_controlled` variant execute
the same pure tags and scalar budget rules as cached sequential execution.
They accept `CompactionLimits { max_frames, max_total_allocations,
max_collection_work }`. The controlled variant accepts the existing cooperative
cancellation callback. Legacy sequential, traced and recursive APIs preserve
their allocation and trace contracts.

Every Order present at entry is pinned at its original index. Their data,
digests and cached formula bounds remain unchanged, including data unrelated
to the initial subject and formula. New internal entries may move. The returned
result Order is valid in the caller's arena; a later call pins that result with
all other entries present at its own entry. Compaction uses exclusive access
to the arena. Only the NoTrace profile supports relocation.

The arena's existing allocation limit bounds resident entries. The new total
allocation allowance counts the preloaded entries plus every fresh allocation
during this execution, including partial finalizer allocations and values
recreated after collection. Intern hits do not consume it. Admission within
each evaluator step is capped by both remaining total allowance and resident
capacity. The original resident limit is restored before returning from the
step. A fresh allocation rejected by the total allowance is a host resource
failure. The new profile does not reproduce the legacy lifetime unique-entry
quota: its explicit resident and allocation-event counters serve different
purposes.

Collection happens between evaluator steps, before a budget-admitted axis-zero
Enter or before a successful child return
enters a finalizer with less free resident capacity than its maximum output
allocation count: seven entries for axis-zero/hash, one for the other allocating pure
finalizers. It may precede a finalizer whose cache or interner eventually hits,
or whose operands cause a semantic error. One collection attempt precedes each
such finalizer step; insufficient reclamation leaves normal resident admission
to return `Unavailable`. Collection neither reevaluates a child nor changes
guest charging, budget partitions, scalar refunds, or logical frame depth.
Missing external operands have no allocation headroom: their semantic failure
does not start collection or become a cumulative allocation error. Cumulative
exhaustion classification applies only to an allocating step with valid operand
Orders, never to propagation of a previous `Unavailable` result.

Roots consist of the entire preloaded prefix, the current action's object and
formula (Enter) or value (successful Return), and pending phase references:
BinaryLeft retains object/right formula, BinaryRight retains its left value,
and BranchTest retains object/both alternatives. Unary, BranchChosen and
Compose retain scalar metadata only. Trace-only historical fields contribute
no roots. The per-execution finalizer cache is cleared before relocation and
contributes no roots.

The collector requires topologically ordered pairs, with both children earlier
than their parent. It rejects malformed roots or allocated pairs before
mutation. It also verifies that all resident entries participate in the
private interning index. Public `alloc_raw` entries which bypass that index
therefore reject collection; rebuilding must not silently start interning
previously unindexed entries.

Scratch is fallibly reserved before evaluation: one Order per logical resident
slot for mark/forwarding state and one Order per physical hash-index slot for
planned placement. `compaction_storage_bytes(physical_capacity, resident_limit)`
reports the requested buffers, excluding allocator metadata and the existing
arena, frame buffer and finalizer cache. No graph recursion or growing traversal
stack is used. Marking proceeds backwards; relocation preserves survivor order.
Digests and cached cost bounds are copied, while pair children and live
continuations are remapped through the forwarding array.

Collection work is separate from guest reductions. One unit is charged before
each visited arena entry, index slot, planned insertion probe, root/action or
frame visit, and cache slot cleared. Fixed-size child/reference operations are
included in their containing visit. Repeated scans each charge their visits.
Planning constructs the future index in scratch, so adversarial probe chains
consume the work allowance and can be cancelled before arena mutation.

Planning polls cancellation periodically, at most 4,096 charged work units
apart. Before mutation, it reserves the exact linear commit visits (old resident
entries, physical index slots, action/frame remapping and cache clearing) from
the work allowance and polls cancellation. `collection_work` is charged work;
the reserved commit is included even if that final pre-mutation poll cancels.
Compaction and index replacement
then finish as one bounded, allocation-free commit before cancellation is
polled again. Cancellation during that commit is observed afterward; the arena
always has a valid index when returned. The host must check its publication
deadline after execution. Commit is cooperative, with a capacity-bounded
non-preemptible interval.

`CompactionStats` reports pinned, final resident and peak resident entries;
total allocation events; reclaimed entries; completed collection passes;
charged collection work; scratch bytes; and separate evaluator/collection
cancellation checkpoint counts. Host failures carry these stats. Failed guest
outcomes carry no invented root remaining-budget or charged-reduction figure.
Where both profiles finish, values are compared structurally/by digest rather
than by relocated Order; successful remaining budget and peak frame depth must
match. Observed performance and workload results belong in `audit/`.
