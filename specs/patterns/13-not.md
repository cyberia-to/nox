# pattern 13: not


parameterized by W. valid on word type only.

```
abstract:   not_W(a) → bitwise complement over W bits
canonical:  v_a ⊕ (2^32 - 1)
```

cost: 1. constraints: ~32.
