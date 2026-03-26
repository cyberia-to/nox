# pattern 0: axis

rs module: patterns/axis.rs

algebra-independent.

```
reduce(s, [0 a], f) = (axis(s, eval(a)), f - 1)
```

the evaluated axis index must be a field-type or word-type atom, interpreted as an integer. if eval(a) produces a cell or hash-type atom → ⊥_error.

with polynomial nouns, axis is O(1) via Lens opening: the binary encoding of the axis address is the evaluation point. this replaces O(depth) tree traversal with a single polynomial evaluation.

cost: 1. constraints: 1.
