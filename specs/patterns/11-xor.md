# pattern 11: xor


parameterized by W. valid on word type only. bitwise on hash → ⊥_error.

```
abstract:   xor_W(a, b) → bitwise exclusive-or over W bits
canonical:  v_a ⊕ v_b (32-bit XOR)
```

cost: 32. multi-row pattern, one row per bit of the 32-bit word. each row exposes (a_k, b_k, c_k) with the per-row XOR gadget c_k = a_k + b_k - 2 * a_k * b_k and block-level decomposition binding sum 2^k * a_k = a (similarly b, c). see specs/trace.md §pattern 11. in F₂: 1 constraint per row.
