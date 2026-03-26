# pattern 9: eq

rs module: patterns/eq.rs

parameterized by F.

```
abstract:   eq(a, b) → 0 if a = b in F, else 1
canonical:  0 if v_a = v_b else 1
```

equality test across all types (field, word, hash). returns 0 for true (consistent with branch: 0 = take yes-branch).

cost: 1. constraints: 1.
