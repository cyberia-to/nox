# execution trace specification

version: 0.1
status: canonical

## overview

the nox execution trace is the sequence of register states across all reduction steps. it IS the stark witness — no separate arithmetization. each row is one reduction step. each column is a register.

## trace layout

```
columns (16 registers):
  r0:  pattern tag (0-16)
  r1:  object hash (4 × F_p, packed)
  r2:  formula hash
  r3:  operand A value
  r4:  operand B value
  r5:  result value
  r6:  focus before
  r7:  focus after
  r8:  type tag A
  r9:  type tag B
  r10: type tag result
  r11: auxiliary 0 (pattern-specific)
  r12: auxiliary 1
  r13: auxiliary 2
  r14: auxiliary 3
  r15: status (0 = ok, 1 = halt, 2 = error)

rows: 2^n (padded to power of 2)
```

## AIR transition constraints

each pattern defines a transition constraint polynomial over consecutive rows.

```
pattern 5 (add):
  r5_{t+1} = r3_t + r4_t                    degree 1
  r7_{t+1} = r6_t - 1                       focus decrement

pattern 7 (mul):
  r5_{t+1} = r3_t × r4_t                    degree 2
  r7_{t+1} = r6_t - 1

pattern 8 (inv):
  r5_{t+1} × r3_t = 1                       degree 2
  r7_{t+1} = r6_t - 64

pattern 15 (hash):
  Poseidon2 round constraints               degree 7
  spanning consecutive rows

pattern 4 (branch):
  selector = (r5_test == 0)
  r5_{t+1} = selector × r5_yes + (1-selector) × r5_no
```

constraint selector: `r0_t = tag` gates each pattern's constraints. only the active pattern's constraints apply per row. SuperSpartan CCS handles mixed degrees natively.

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
