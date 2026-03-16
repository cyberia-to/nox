# execution trace specification

version: 0.1
status: canonical

## overview

the nox execution trace is the sequence of register states across all reduction steps. it IS the stark witness — no separate arithmetization. each row is one reduction step. each column is a register.

## trace layout

```
columns (16 = 2⁴ registers, each one F_p element):
  r0:   pattern tag (0-16)
  r1:   object hash[0]          ┐ 128-bit compressed identity
  r2:   object hash[1]          ┘ (first 2 of 8 elements)
  r3:   formula hash[0]         ┐ 128-bit compressed identity
  r4:   formula hash[1]         ┘ (first 2 of 8 elements)
  r5:   operand A value
  r6:   operand B value
  r7:   result value
  r8:   focus before
  r9:   focus after
  r10:  type tag A (0x00 field, 0x01 word, 0x02 hash)
  r11:  type tag result
  r12:  auxiliary 0 (pattern-specific)
  r13:  auxiliary 1
  r14:  auxiliary 2
  r15:  status (0 = ok, 1 = halt, 2 = error)

rows: 2^n (padded to power of 2)
```

full 64-byte identities (8 elements each) are in the instance (public input). the trace stores 2 elements per hash for row-linking — 128-bit collision security (birthday bound 2⁶⁴), matching Goldilocks field security. the instance constraint verifies full hashes against the first row.

## instance (public input)

```
instance = (H(object), H(formula), H(result), focus_initial, focus_final, status)

H(object):  8 × F_p (full 64-byte identity)
H(formula): 8 × F_p (full 64-byte identity)
H(result):  8 × F_p (full 64-byte identity)
focus_initial: F_p
focus_final:   F_p
status:        F_p (0 = ok, 1 = halt, 2 = error)
```

the instance links the trace to the computation. the verifier checks:
1. first row: r1,r2 match instance H(object)[0..2], r3,r4 match instance H(formula)[0..2]
2. last row: r7 matches instance H(result)[0..2], r9 matches focus_final, r15 matches status
3. full hash verification: instance hashes are checked against committed noun store

## AIR transition constraints

each pattern defines a transition constraint polynomial over consecutive rows.

```
pattern 5 (add):
  r7_{t+1} = r5_t + r6_t                    degree 1
  r9_{t+1} = r8_t - 1                       focus decrement

pattern 7 (mul):
  r7_{t+1} = r5_t × r6_t                    degree 2
  r9_{t+1} = r8_t - 1

pattern 8 (inv):
  r7_{t+1} × r5_t = 1                       degree 2
  r9_{t+1} = r8_t - 64

pattern 15 (hash):
  Poseidon2 round constraints               degree 7
  consumes multiple consecutive rows (see multi-row patterns)

pattern 4 (branch):
  selector = (1 - r7_test × r7_test_inv)    ; 1 if zero, 0 if nonzero
  r7_{t+1} = selector × r7_yes + (1-selector) × r7_no
```

constraint selector: `r0_t = tag` gates each pattern's constraints. only the active pattern's constraints apply per row. SuperSpartan CCS handles mixed degrees natively.

## multi-row patterns

patterns with cost > 1 consume multiple trace rows. each row has r0 = pattern tag, and intermediate rows use auxiliary registers for internal state.

```
pattern 15 (hash), cost 300:
  row 0:     r5 = input value, r12-r14 = initial sponge state
  rows 1-71: r12-r14 = round state (8 full + 64 partial Poseidon2 rounds)
  row 72+:   r7 = hash output element, r12-r14 = remaining squeeze state
  total rows: ~75 (72 rounds + absorption + squeeze overhead)
  each row: degree 7 constraint (s-box x^7)

pattern 8 (inv), cost 64:
  row 0:     r5 = input value, r12 = accumulator
  rows 1-63: r12 = intermediate square-and-multiply state
  row 64:    r7 = inverse result
  total rows: 64
  each row: degree 2 constraint (single multiplication)

pattern 2 (compose), cost 2:
  row 0: dispatch to sub-expression x
  row 1: dispatch to sub-expression y
  sub-expressions generate their own rows recursively

pattern 3 (cons), cost 2:
  row 0: dispatch to sub-expression a
  row 1: dispatch to sub-expression b
  result row: r7 = cell identity (compressed hash of cell(ra, rb))
```

## row linking

consecutive rows are linked by constraints on object and formula hash elements:

```
same object:  r1_{t+1} = r1_t AND r2_{t+1} = r2_t
new object:   r1_{t+1}, r2_{t+1} set by compose result hash
same formula: r3_{t+1} = r3_t AND r4_{t+1} = r4_t
new formula:  r3_{t+1}, r4_{t+1} set by sub-expression dispatch
```

## error and halt encoding

```
r15 = 0: ok — pattern completed successfully
r15 = 1: halt — focus exhausted (r8 < cost)
r15 = 2: error — type/semantic error

when r15 = 1 (halt):
  r8 = remaining focus (insufficient for next pattern)
  r7 = 0 (no result)

when r15 = 2 (error):
  r12 = error kind (0 = type error, 1 = axis on atom, 2 = inv(0), 3 = unavailable)
  r7 = 0 (no result)
```

errors and halts propagate: once r15 ≠ 0, subsequent rows maintain the same status.

## encoding as multilinear polynomial

the entire trace (2^n rows × 16 columns) encodes as one multilinear polynomial:

```
f(x_1, ..., x_{n+4}) : F_p → F_p

where:
  x_1..x_n   select the row (step index in binary)
  x_{n+1}..x_{n+4}  select the column (register index)
```

WHIR commits to f. sumcheck verifies transition constraints. the verifier checks O(log n) evaluation queries.

## self-verification

the stark verifier is a nox program. it reads a proof (a noun), computes Fiat-Shamir challenges (hash jet), verifies Merkle paths (merkle_verify jet), checks polynomial evaluations (poly_eval jet), and performs FRI folding (fri_fold jet).

```
verifier cost without jets:  ~600,000 patterns
verifier cost with jets:     ~70,000 patterns (8.5× reduction)

breakdown:
  parse proof:            ~1,000
  Fiat-Shamir challenges: ~5,000  (was ~30,000)
  Merkle verification:    ~50,000 (was ~500,000)
  constraint evaluation:  ~3,000  (was ~10,000)
  WHIR verification:      ~10,000 (was ~50,000)
```

recursive composition: prove the verifier's execution. proof-of-proof at every block. constant proof size at every recursion level (~60-157 KiB).
