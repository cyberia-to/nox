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

data is stored in a flat array indexed by `Order`. no heap allocation, no
pointer chasing — pure index arithmetic.

## bounds

| parameter | value | rationale |
|-----------|-------|-----------|
| max depth | 64 | covers 2^64 leaves — more than particle count in cybergraph. axis path = 64 bits max |
| max count | 2^24 (16M data nodes) | 16M × 16 bytes = 256 MB. configurable compile-time const. phone mode: 2^20 (16 MB). server: 2^28 (4 GB) |
| max atom size | 4 field elements (32 bytes) | hash type = 4 × F_p. field and word = 1 × F_p |

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
