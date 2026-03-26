# pattern 3: cons

rs module: patterns/cons.rs

algebra-independent.

```
reduce(s, [3 [a b]], f) =
  let (ra, f1) = reduce(s, a, f - 1)
  let (rb, f2) = reduce(s, b, f1)
  (cell(ra, rb), f2)
```

build a cell from two evaluated sub-expressions.

PARALLELISM: reduce(s,a) and reduce(s,b) are INDEPENDENT.

cost: 1. constraints: 1.
