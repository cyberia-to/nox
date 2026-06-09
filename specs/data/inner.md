# data

## definition

data is an atom or a pair:

```
data = atom | pair
atom = field          — a leaf: one field. no type tag.
pair = (data, data)   — two children joined, each itself an atom or a pair.
```

a pair's children are `data` — not a separate kind. abstractly:

```
Data = Atom(field) | Pair(Data, Data)
```

## storage

data is a hash-consed DAG in a flat arena (the `reduction`), so a pair does not
*inline* its children — it *references* each by its `order` (the child's slot).
the stored form is:

```
enum Data {
    Atom { value: field },
    Pair { left: Order, right: Order },   // Order = a handle to a child data node
}
```

`Order = u32` is a data node's *local* identity — its slot in the reduction. the
`Order` in a pair points to a child; the child itself is an `atom` or a `pair`.
the `order` is local to its reduction; the *global* identity of data is its
`particle` (see hash.md, reduction.md).

## polynomial representation

every data node is a multilinear polynomial over {0,1}^k where k = ceil(log₂(leaves)).

```
atom(v)      → constant polynomial v
pair(a, b)   → g(x₁, x₂, ...) = (1 - x₁)·a(x₂, ...) + x₁·b(x₂, ...)
```

pair construction is variable prepend: the first variable selects which subtree (0 = left = a, 1 = right = b), and the remaining variables address within that subtree. an atom is the base case — a constant polynomial with no variables.

## axis as polynomial evaluation

axis(s, n) on a polynomial data node is a polynomial evaluation at a binary point in {0,1}^k. the binary encoding of the axis address selects the evaluation point. Lens opening proves the evaluation in O(1) — a ~75 byte proof regardless of depth. this replaces O(depth) tree traversal with O(1) polynomial evaluation.
