# data

## definition

```
enum Data {
    Atom { value: field },
    Pair { left: OrderId, right: OrderId },
}
```

`OrderId = u32` — an index into the `order` (the arena that stores data). each
child of a pair is itself `data` (an `atom` or a `pair`), referenced by its
`OrderId`. the index is local to the order; the portable identity of data is its
`particle`.

atom: a leaf — one `field`. no type tag.
pair: two children joined, each an `atom` or a `pair`. the structural constructor.

## polynomial representation

every data node is a multilinear polynomial over {0,1}^k where k = ceil(log₂(leaves)).

```
atom(v)      → constant polynomial v
pair(a, b)   → g(x₁, x₂, ...) = (1 - x₁)·a(x₂, ...) + x₁·b(x₂, ...)
```

pair construction is variable prepend: the first variable selects which subtree (0 = left = a, 1 = right = b), and the remaining variables address within that subtree. an atom is the base case — a constant polynomial with no variables.

## axis as polynomial evaluation

axis(s, n) on a polynomial data node is a polynomial evaluation at a binary point in {0,1}^k. the binary encoding of the axis address selects the evaluation point. Lens opening proves the evaluation in O(1) — a ~75 byte proof regardless of depth. this replaces O(depth) tree traversal with O(1) polynomial evaluation.
