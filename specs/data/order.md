# order

an order is the execution context for one order() — a neuron's command to apply a formula to an object. it holds all data created during the computation, provides hash-consed identity, and is freed when the computation completes.

```
type OrderId = u32;                 // an index into this order (the arena)

struct Order {
    id:      H(formula, object),    // axon — the order's identity
    data:    [Data],                // flat array, indexed by OrderId
    index:   BoundedMap,            // hash-consing: H(data) → OrderId
    count:   u32,                   // next free slot
}
```

## identity

every order has a natural id: the axon `H(formula, object)`. this is content-addressed from the computation itself — same formula applied to the same object always produces the same order id, regardless of who orders it or when.

`OrderId` is the order-local index of one data node (the arena slot). it is distinct from a `particle` — the portable, content-derived 32-byte identity of data. order ids are meaningless outside their order; particles are global.

## memory

data is stored in a flat array indexed by OrderId. no heap allocation, no pointer chasing — pure index arithmetic.

## bounds

| parameter | value | rationale |
|-----------|-------|-----------|
| max depth | 64 | covers 2^64 leaves — more than particle count in cybergraph. axis path = 64 bits max |
| max count | 2^24 (16M data nodes) | 16M × 16 bytes = 256 MB. configurable compile-time const. phone mode: 2^20 (16 MB). server: 2^28 (4 GB) |
| max atom size | 4 field elements (32 bytes) | hash type = 4 × F_p. field and word = 1 × F_p |

## structural sharing (DAG)

data is a DAG, not a tree. hash-consing deduplicates structurally identical sub-data:

```
insert(order, pair(l, r)):
  h = H(pair(l, r))
  if order.index[h] exists:
    return order.index[h]         // reuse existing data node
  id = order.alloc(Pair { left: l, right: r })
  order.index[h] = id
  return id
```

properties:
- identical sub-expressions share one slot
- memory proportional to unique structure, not total size
- hash-consing cost: one hemera hash per pair construction
- lookup: O(1) via hash index (BoundedMap)
- DAG is safe because data is immutable — no mutation, no aliasing hazard

hash-consing is required, not optional. it ensures that `H(data)` = order identity — the same data always has the same OrderId. this is the foundation of memoization correctness.

## lifecycle

one order per order() invocation. the order is allocated at entry, all data lives in it, and it is freed when order() returns. no cross-computation data sharing — each order is isolated.

the memo cache stores (H(object), H(formula)) → H(result) — particles, not order ids. order ids are order-local and meaningless outside their order.
