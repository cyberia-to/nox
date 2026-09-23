# Complete native artifacts

Version: 1 (`NOXDAG01`). A bounded, complete single-root noun container for
local program/data exchange. This extends the storage encoding; it does not
change the existing network message types or their 16 MiB limit.

## Bytes and identity

All integers are unsigned little-endian. An artifact contains exactly:

```text
magic         8 bytes    ASCII NOXDAG01
root         32 bytes   root particle (four canonical Goldilocks limbs)
node_count    4 bytes    nonzero U32
entries       repeated node_count times:
  particle   32 bytes
  length      1 byte    8 for atom, 64 for pair
  payload     length bytes, per encoding.md (Model B)
```

An atom payload is one canonical Goldilocks field word. A pair payload contains
left then right child particles. All particle limbs, including header and child
references, must also be canonical; reducing out-of-range limbs is forbidden.
Every entry's particle must match the existing native tree hash of its payload.
There is no new hash scheme or host arena identifier.

Entries follow **left-first depth-first postorder**, emitting each particle only
on its first completed visit. All children precede their parents; the root is
last. This order is independent of arena allocation history. An artifact contains
exactly the root's reachable DAG, with no duplicate particles, unreachable nodes,
missing/forward references or extra bytes. Equal shared children appear once.
Hash identity has the native collision-resistance assumption.

For example, `[[1 2] 3]` emits atoms 1,2, pair[1,2], atom3, root. `[1 [2 3]]`
emits atoms1,2,3, pair[2,3], root. Both root identities and artifact bytes differ.
`[x x]` emits the complete DAG of x once, followed by the pair root.

## Bounds and failure

Every encode/decode call supplies `Limits { max_bytes, max_nodes, max_depth }`.
There are no implicit unbounded/default limits. Atom depth is zero; pair depth
is one plus the maximum child depth. Depth is computed for each unique node,
so sharing cannot conceal a longer path. Limits are inclusive; exact-limit
inputs succeed. The declared node count must fit both the node cap and the
remaining bytes at the minimum entry size before allocating per-node storage.

Traversal is iterative and bounded by unique nodes/edges; it never expands the
tree into its leaves or recurses on the host stack. Limits apply to encoding as
well as decoding. All artifact-format checks complete before loading into an arena.
If arena allocation subsequently fails, no successful root is returned; earlier
allocations may remain charged to that arena. Existing nodes remain valid.
The caller uses a fresh job arena when it requires failed loads to be discarded.

Errors distinguish framing/version, noncanonical fields, hash mismatch,
duplicates, missing references/root, noncanonical order, invalid node, byte/node/
depth limits and arena allocation. The format certifies a complete noun, not that
the noun is a valid executable formula, compiler job or proof. Those semantic
schemas and entry profiles are validated by their owning consumers.

## Integration

`nox::artifact::{encode, decode, Limits, Error}` is the shared codec API.
Trident owns compiler job/result noun schemas; Joy owns file handling, execution
and atomic publication. Joy transports an execution result directly from its
root through this codec, preserving topology and sharing. Converting through
flat leaves or bracket text would lose structure or expand a compact DAG.
State/network services and witness calls are unnecessary for local transport.

The soft3 0.4 work uses this format for sources, compiler jobs, formula results
and bootstrap comparisons. Format validation itself makes no execution proof
claim. Zheng must authenticate the complete roots in the selected proof profile.
