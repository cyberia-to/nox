# pattern 8: inv


parameterized by F.

```
abstract:   inv_F(a) → a⁻¹ in F, or ⊥_error if a = 0
canonical:  v_a^(p-2) mod p    (Fermat's little theorem)
```

execution cost: 64 (reflects ~64 multiplications in square-and-multiply for Goldilocks).
zheng verification cost: 1 constraint (verifier checks a × a⁻¹ = 1).

the asymmetry between execution cost and verification cost is fundamental: inversion is expensive to compute but cheap to verify.

## trace ordering

implementation uses MSB-first square-and-multiply. the Fermat exponent p-2 has bit 63 (MSB) = 1, so the accumulator initializes to `v` at row 0 (not acc=1). bits are processed 63..0; r11 holds the current bit.

note: r10_{row0} = v, r11_{row0} = bit 63 of P_MINUS_2 (= 1). see specs/trace.md §pattern 8 for the full row-by-row register layout.
