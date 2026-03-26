# pattern 2: compose


algebra-independent.

```
reduce(o, [2 [x y]], f) =
  let (rx, f1) = reduce(o, x, f - 1)
  let (ry, f2) = reduce(o, y, f1)
  reduce(rx, ry, f2)
```

evaluate x to get a new object, evaluate y to get a formula, then apply. this is the recursion mechanism — all control flow, looping, and function application reduce to compose.

PARALLELISM: reduce(o,x) and reduce(o,y) are INDEPENDENT.

cost: 1. constraints: 1.
