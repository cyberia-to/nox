# pattern 12: and


parameterized by W. valid on word type only.

```
abstract:   and_W(a, b) → bitwise conjunction over W bits
canonical:  v_a ∧ v_b (32-bit AND)
```

cost: 32. multi-row pattern, one row per bit of the 32-bit word. each row exposes (a_k, b_k, c_k) with the per-row AND gadget c_k = a_k * b_k, plus block-level decomposition binding. see specs/trace.md §pattern 12.
