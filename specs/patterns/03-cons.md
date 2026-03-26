# pattern 3: cons


algebra-independent.

```
reduce(o, [3 [a b]], f) =
  let (ra, f1) = reduce(o, a, f - 1)
  let (rb, f2) = reduce(o, b, f1)
  (cell(ra, rb), f2)
```

build a cell from two evaluated sub-expressions.

PARALLELISM: reduce(o,a) and reduce(o,b) are INDEPENDENT.

cost: 1. constraints: 1.
