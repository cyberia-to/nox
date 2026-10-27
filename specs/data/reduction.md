# reduction

a reduction is the execution context for one run — a neuron's command to apply a
formula to an object under a budget. it holds all data created during the
computation, provides hash-consed identity, and is freed when the computation
completes.

```
type Order = u32;                    // a data node's local identity (its slot)

struct Reduction {
    id:      H(formula, object),     // axon — the reduction's identity
    data:    [Data],                 // flat array, indexed by Order
    index:   BoundedMap,             // hash-consing: H(data) → Order
    count:   u32,                    // next free slot
}
```

## identity

every reduction has a natural id: the axon `H(formula, object)`. this is
content-addressed from the computation itself — the same formula applied to the
same object always produces the same reduction id, regardless of who runs it or
when.

`Order` is a data node's local identity within one reduction — its slot in the
arena. it is distinct from a `particle`, the global, content-derived 32-byte
identity of data. an `order` is meaningless outside its reduction; a `particle`
is the same everywhere. (one global identity, one local — see soft3/specs/terms.)

## memory

Data is stored in flat arrays indexed by `Order`. `Reduction<N>` reserves
`N` entries plus the hash-consing key/value arrays. Its physical size is
`core::mem::size_of::<Reduction<N>>()`; the entry includes its cached particle
and cost bound. The caller admits this storage separately from the logical
node allowance, evaluator frames, codec workspace and trace storage.

`Reduction::new()` returns the fixed-size arena by value. Callers of this path
must provide stack space for the arena and constructor/move temporaries.
`Reduction::try_new_boxed()` allocates and initializes the same representation
directly in the heap and returns `Result<Box<Reduction<N>>, AllocationError>`.
Its stack usage is independent of `N`; it constructs no whole-arena temporary.
Both constructors begin with count zero, empty hash-consing slots and the same
logical allowance. Allocation strategy changes no `Order`, particle, cost,
codec bytes or reduction semantics.

The heap constructor checks that `N` is a nonzero power of two fitting `u32`
before allocation. Invalid capacity returns `AllocationError::InvalidCapacity`;
a null global-allocation result returns `AllocationError::OutOfMemory`.
Success owns one allocation released when its `Box` is dropped. Host allocator
and operating-system policies still determine whether a memory request can be
satisfied. Parallel `fork()` retains its existing by-value construction.

## bounds

| parameter | value | rationale |
|-----------|-------|-----------|
| slots | compile-time power-of-two `N` fitting `u32` | indexed storage and hash-cons table mask |
| distinct nodes | `3 * floor(N / 4)` by default | hash-cons table load factor at most three quarters; callers may tighten the lifetime allowance |
| atom | one Goldilocks field element | compound values, including a four-field particle, use structured pairs |
| axis | field-encoded binary path | traversal consumes the path bits and fails if it reaches an atom early |

The [local arena allowance](../reduction.md#local-arena-allowance) defines
failure and accounting across loading and execution. A larger physical arena
alone does not increase a caller's admitted lifetime allowance.

## structural sharing (DAG)

data is a DAG, not a tree. hash-consing deduplicates structurally identical
sub-data:

```
insert(reduction, pair(l, r)):
  h = H(pair(l, r))
  if reduction.index[h] exists:
    return reduction.index[h]       // reuse existing data node
  o = reduction.alloc(Pair { left: l, right: r })
  reduction.index[h] = o
  return o
```

properties:
- identical sub-expressions share one slot
- memory proportional to unique structure, not total size
- hash-consing cost: one hemera hash per pair construction
- lookup: O(1) via hash index (BoundedMap)
- DAG is safe because data is immutable — no mutation, no aliasing hazard

hash-consing is required, not optional. it ensures that `H(data)` maps to one
slot — the same data always gets the same `order`. this is the foundation of
memoization correctness.

## lifecycle

one reduction per run. the reduction is allocated at entry, all data lives in
it, and it is freed when the run returns. no cross-run data sharing — each
reduction is isolated.

the memo cache stores (H(object), H(formula)) → H(result) — particles, not
orders. an `order` is reduction-local and meaningless outside its reduction.
