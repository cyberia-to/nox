# pattern 14: shl


parameterized by W. valid on word type only.

```
abstract:   shl_W(a, n) → left shift over W bits, n must be in [0, W)
canonical:  (v_a << v_n) mod 2^32, shifts ≥ 32 produce 0
```

right shift is expressible as `shl(a, W-n)` followed by `and` with a mask.

cost: 32. multi-row pattern, one row per output bit position. each row exposes (a_k, src_bit, c_k, src_idx) with per-row gadget c_k = src_bit and a cross-row binding src_bit = a_{src_idx} where src_idx = k - n (or 32 sentinel when out of range). see specs/trace.md §pattern 14.
