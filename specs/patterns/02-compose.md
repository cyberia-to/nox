# pattern 2: compose

rs module: patterns/compose.rs

algebra-independent.

```
reduce(s, [2 [x y]], f) =
  let (rx, f1) = reduce(s, x, f - 1)
  let (ry, f2) = reduce(s, y, f1)
  reduce(rx, ry, f2)
```

evaluate x to get a new object, evaluate y to get a formula, then apply. this is the recursion mechanism — all control flow, looping, and function application reduce to compose.

PARALLELISM: reduce(s,x) and reduce(s,y) are INDEPENDENT.

cost: 1. constraints: 1.
