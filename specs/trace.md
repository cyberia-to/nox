# execution trace specification

version: 0.2
status: canonical

## overview

the nox execution trace is the sequence of register states across all reduction steps. it IS the stark witness — no separate arithmetization. each row is one reduction step. each column is a register.

the trace layout is algebra-independent in structure (16 registers, power-of-2 rows) but per-instantiation in element size (each register holds one element of the instantiated field F). all concrete details below refer to the canonical instantiation: nox<Goldilocks, Z/2^32, Hemera>.

## trace layout

```
columns (16 = 2⁴ registers, each one F element):
  r0:   pattern tag (0-16)
  r1:   object hash[0]          ┐ 128-bit compressed identity
  r2:   object hash[1]          ┘ (first 2 of 4 elements)
  r3:   formula hash[0]         ┐ 128-bit compressed identity
  r4:   formula hash[1]         ┘ (first 2 of 4 elements)
  r5:   operand A value
  r6:   operand B value
  r7:   result value (atom value for atoms, H(result)[0] for cells,
                       or Lens commitment element for polynomial noun references)
  r8:   budget before
  r9:   budget after
  r10:  type tag A (0x00 field, 0x01 word, 0x02 hash)
  r11:  type tag result
  r12:  auxiliary 0 (pattern-specific)
  r13:  auxiliary 1
  r14:  auxiliary 2
  r15:  status (0 = ok, 1 = halt, 2 = error)

rows: 2^n (padded to power of 2)
```

full 32-byte identities (4 elements each) are in the instance (public input). the trace stores 2 elements per hash for row-linking — 128-bit collision security (birthday bound 2⁶⁴), matching Goldilocks field security. the instance constraint verifies full hashes against the first row.

## instance (public input)

```
instance = (H(object), H(formula), H(result), focus_initial, focus_final, status)

H(object):  4 × F_p (full 32-byte identity)
H(formula): 4 × F_p (full 32-byte identity)
H(result):  4 × F_p (full 32-byte identity)
focus_initial: F_p
focus_final:   F_p
status:        F_p (0 = ok, 1 = halt, 2 = error)
```

when status = 0 (success), H(result) is the structural hash of the result noun. when status ≠ 0 (halt or error), there is no result noun — H(result) MUST be the all-zero hash (eight zero elements). the verifier enforces: `status ≠ 0 ⟹ H(result) = 0`.

the instance links the trace to the computation. the verifier checks:
1. first row: r1,r2 match instance H(object)[0..2], r3,r4 match instance H(formula)[0..2]
2. last row: r7 matches instance H(result)[0], r9 matches focus_final, r15 matches status
3. status-gated result: if status = 0, H(result) is a valid noun identity; if status ≠ 0, H(result) = 0
4. noun store commitment: instance identities (identity(object), identity(formula), and identity(result) when status = 0) are checked against the noun store polynomial commitments. each noun identity is hemera(Lens.commit(noun_polynomial) ‖ domain_tag) — trace register values may include Lens commitments (32-byte polynomial commitments stored as 4 field element tuples). the commitment scheme is defined by the proof system (zheng) — nox specifies WHAT is committed (the noun identities referenced by the trace), not HOW

## constraint system

each pattern defines constraints over its trace row. single-row patterns use in-row constraints. multi-row patterns (axis depth>1, inv, hash) use transition constraints across consecutive rows. SuperSpartan CCS handles both forms and mixed degrees natively.

constraint degrees and counts are per-instantiation. the structure of the constraint system (which registers constrain which patterns) is algebra-independent. the concrete degrees below refer to the canonical instantiation.

### single-row patterns (cost = 1)

each reduce() call produces 1 trace row. operand values (r5, r6) are connected to sub-expression result rows through CCS wiring constraints.

```
pattern 5 (add):
  r7 = r5 + r6                              degree 1
  r9 = r8 - 1                               budget decrement

pattern 7 (mul):
  r7 = r5 × r6                              degree 2
  r9 = r8 - 1

pattern 9 (eq):
  r7 = (r5 == r6 ? 0 : 1)                   degree 1
  r9 = r8 - 1

pattern 4 (branch):
  selector = (1 - r5_test × r5_test_inv)    ; 1 if zero, 0 if nonzero
  r7 = selector × r12_yes + (1-selector) × r12_no
  r9 = r8 - 1
```

### multi-row patterns

patterns with multi-step overhead use transition constraints across consecutive rows. all rows share the same r0 tag.

```
pattern 8 (inv), cost 64:
  row 0:     r5 = input value, r12 = accumulator = 1
  rows 1-63: r12_{t+1} = r12_t × r12_t × (bit ? r5 : 1)   square-and-multiply
  row 63:    r7 = r12 (final inverse)
  verification: r7 × r5 = 1                                 degree 2

pattern 15 (hash), cost 200:
  row 0:     r5 = input value, r12-r14 = initial sponge state
  rows 1-199: round state progression (8 full + 16 partial Poseidon2 rounds,
              multiple rows per round for state element constraints)
  row 199:   r7 = H(result)[0], r12-r14 = remaining hash elements
  total rows: ~200 (24 rounds × ~8 rows/round + absorption/squeeze)
  full round constraint rows: degree 7 (s-box x^7)
  partial round constraint rows: degree 2 (s-box x^{-1}, verified as x × y = 1)

pattern 0 (axis), cost = 1:
  row 0:     r5 = root noun polynomial commitment, r12 = evaluation point (binary encoding of axis address)
  Lens opening verifies the evaluation in 1 constraint (degree 1)
  legacy (tree traversal): cost = depth, rows 0..d with index bit selection per row
```

## single-row vs multi-row patterns

most patterns produce exactly 1 trace row per reduce() call. two patterns produce multiple rows for their internal computation:

```
single-row (cost 1): axis, quote, compose, cons, branch, add, sub, mul,
                     eq, lt, xor, and, not, shl, hint

multi-row:
  axis: 1 row — O(1) polynomial evaluation via Lens opening (legacy: d rows for tree traversal)
  inv (cost 64):  64 rows — square-and-multiply chain
  hash (cost 200): ~200 rows — Poseidon2 permutation rounds
```

single-row patterns store operands in r5/r6 and result in r7. the operand values are wired to sub-expression result rows through CCS wiring constraints. compose and cons dispatch sub-expressions whose results flow back through the wiring; compose additionally generates a third reduce() call (its own row) for the final application.

## row linking

consecutive rows are linked by constraints on object and formula hash elements:

```
same object:  r1_{t+1} = r1_t AND r2_{t+1} = r2_t
new object:   r1_{t+1}, r2_{t+1} set by compose result hash
same formula: r3_{t+1} = r3_t AND r4_{t+1} = r4_t
new formula:  r3_{t+1}, r4_{t+1} set by sub-expression dispatch
```

## budget decrement constraints

```
single-row patterns:  r9 = r8 - 1            (1 per reduce call)
axis:                 r9 = r8 - 1            (1 — O(1) Lens opening)
inv:                  r9 = r8 - 64           (64 for square-and-multiply)
hash:                 r9 = r8 - 200          (200 for Poseidon2 hemera)
```

## error and halt encoding

```
r15 = 0: ok — pattern completed successfully
r15 = 1: halt — budget exhausted (r8 < cost)
r15 = 2: error — type/semantic error

when r15 = 1 (halt):
  r8 = remaining budget (insufficient for next pattern)
  r7 = 0 (no result)

when r15 = 2 (error):
  r12 = error kind (0 = type error, 1 = axis on atom, 2 = inv(0), 3 = unavailable, 4 = malformed)
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

recursive composition: prove the verifier's execution. proof-of-proof at every block. constant proof size at every recursion level (~1-5 KiB with zheng-2).
