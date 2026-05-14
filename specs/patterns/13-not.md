# pattern 13: not


parameterized by W. valid on word type only.

```
abstract:   not_W(a) → bitwise complement over W bits
canonical:  v_a ⊕ (2^32 - 1)
```

cost: 32. multi-row pattern, one row per bit of the 32-bit word. unary: r5 / r11 (b-side) are zero on every row. per-row gadget c_k = 1 - a_k. see specs/trace.md §pattern 13.
