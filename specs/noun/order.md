# order

an order is the execution context for one order() — a neuron's command to apply a formula to an object. it holds all nouns created during the computation, provides hash-consed identity, and is freed when the computation completes.

```
type NounId = u32;

struct Order {
    id:      H(formula, object),    // axon — the order's identity
    nouns:   [Noun],                // flat array, indexed by NounId
    index:   BoundedMap,            // hash-consing: H(noun) → NounId
    count:   u32,                   // next free slot
}
```

## identity

every order has a natural id: the axon `H(formula, object)`. this is content-addressed from the computation itself — same formula applied to the same object always produces the same order id, regardless of who orders it or when.

## memory

nouns are stored in a flat array indexed by NounId. no heap allocation, no pointer chasing — pure index arithmetic.

## bounds

| parameter | value | rationale |
|-----------|-------|-----------|
| max depth | 64 | covers 2^64 leaves — more than particle count in cybergraph. axis path = 64 bits max |
| max count | 2^24 (16M nouns) | 16M × 16 bytes = 256 MB. configurable compile-time const. phone mode: 2^20 (16 MB). server: 2^28 (4 GB) |
| max atom size | 4 field elements (32 bytes) | hash type = 4 × F_p. field and word = 1 × F_p |

## structural sharing (DAG)

nouns are DAGs, not trees. hash-consing deduplicates structurally identical sub-nouns:

```
insert(order, cell(l, r)):
  h = H(cell(l, r))
  if order.index[h] exists:
    return order.index[h]         // reuse existing noun
  id = order.alloc(Cell { left: l, right: r })
  order.index[h] = id
  return id
```

properties:
- identical sub-expressions share one slot
- memory proportional to unique structure, not total size
- hash-consing cost: one hemera hash per cell construction
- lookup: O(1) via hash index (BoundedMap)
- DAG is safe because nouns are immutable — no mutation, no aliasing hazard

hash-consing is required, not optional. it ensures that `H(noun)` = order identity — the same noun always has the same NounId. this is the foundation of memoization correctness.

## lifecycle

one order per order() invocation. the order is allocated at entry, all nouns live in it, and it is freed when order() returns. no cross-computation noun sharing — each order is isolated.

the memo cache stores (H(object), H(formula)) → H(result) — hashes, not NounIds. NounIds are order-local and meaningless outside their order.
