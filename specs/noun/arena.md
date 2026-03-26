# arena

rs module: noun/arena.rs

## memory representation

nouns are stored in a flat arena indexed by u32 references. no heap allocation, no pointer chasing — pure index arithmetic.

```
type NounRef = u32;
```

## bounds

| parameter | value | rationale |
|-----------|-------|-----------|
| max depth | 64 | covers 2^64 leaves — more than particle count in cybergraph. axis path = 64 bits max |
| max count | 2^24 (16M nouns) | 16M × 16 bytes = 256 MB arena. configurable compile-time const. phone mode: 2^20 (16 MB). server: 2^28 (4 GB) |
| max atom size | 4 field elements (32 bytes) | hash type = 4 × F_p. field and word = 1 × F_p |

## structural sharing (DAG)

nouns are DAGs, not trees. hash-consing deduplicates structurally identical sub-nouns:

```
insert(arena, cell(l, r)):
  h = H(cell(l, r))
  if arena.hash_index[h] exists:
    return arena.hash_index[h]     // reuse existing node
  ref = arena.alloc(Cell { left: l, right: r })
  arena.hash_index[h] = ref
  return ref
```

properties:
- identical sub-expressions share one arena slot
- memory proportional to unique structure, not total size
- hash-consing cost: one hemera hash per cell construction
- lookup: O(1) via hash index (BoundedMap)
- DAG is safe because nouns are immutable — no mutation, no aliasing hazard

hash-consing is required, not optional. it ensures that `H(noun)` = arena identity — the same noun always has the same NounRef. this is the foundation of memoization correctness.

## arena lifecycle

one arena per ask() invocation. the arena is allocated at entry, all nouns live in it, and it is freed when ask() returns. no cross-computation noun sharing — each computation is isolated.

the memo cache stores (H(object), H(formula)) → H(result) — hashes, not NounRefs. NounRefs are arena-local and meaningless outside their computation.
