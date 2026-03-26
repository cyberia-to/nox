# pattern 10: lt

rs module: patterns/lt.rs

parameterized by F.

```
abstract:   lt(a, b) → 0 if a < b under F's canonical ordering, else 1
canonical:  0 if v_a < v_b else 1
```

cost: 1. constraints: ~64 (range decomposition for non-native comparison in Goldilocks). in F₂, lt is trivial (1 constraint).
