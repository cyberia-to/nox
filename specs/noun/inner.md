# noun

## definition

```
enum Noun {
    Atom { value: F, tag: u8 },
    Cell { left: NounId, right: NounId },
}
```

atom: single element of the instantiated field F, tagged by type.
cell: ordered pair of two nouns — a binary tree node.

## polynomial representation

every noun is a multilinear polynomial over {0,1}^k where k = ceil(log₂(leaves)).

```
atom(v)     → constant polynomial v
cell(a, b)  → g(x₁, x₂, ...) = (1 - x₁)·a(x₂, ...) + x₁·b(x₂, ...)
```

cell construction is variable prepend: the first variable selects which subtree (0 = left = a, 1 = right = b), and the remaining variables address within that subtree. an atom is the base case — a constant polynomial with no variables.

## axis as polynomial evaluation

axis(s, n) on a polynomial noun is a polynomial evaluation at a binary point in {0,1}^k. the binary encoding of the axis address selects the evaluation point. Lens opening proves the evaluation in O(1) — a ~75 byte proof regardless of noun depth. this replaces O(depth) tree traversal with O(1) polynomial evaluation.
