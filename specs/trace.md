# execution trace specification

version: 0.3
status: canonical

## overview

the nox execution trace is the sequence of register states across all reduction steps. it IS the stark witness — no separate arithmetization. each row is one reduction step. each column is a register.

the trace layout is algebra-independent in structure (16 registers, power-of-2 rows) but per-instantiation in element size (each register holds one element of the instantiated field F). all concrete details below refer to the canonical instantiation: nox<Goldilocks, Z/2^32, Hemera>.

## trace layout

```
columns (16 = 2⁴ registers, each one F element):
  r0:   pattern tag (0-17)
  r1:   object NounId             — identity of the object noun
  r2:   formula NounId            — identity of the formula noun
  r3:   result NounId             — identity of the result noun
  r4:   pattern-specific operand 0
  r5:   pattern-specific operand 1
  r6:   pattern-specific operand 2
  r7:   pattern-specific operand 3
  r8:   budget before step
  r9:   budget after step
  r10:  reserved
  r11:  reserved
  r12:  reserved
  r13:  reserved
  r14:  reserved
  r15:  reserved

rows: 2^n (padded to power of 2)
```

a NounId is a single field element that uniquely identifies a noun in the noun store. the instance (public input) contains the full identities. the trace stores one NounId per noun per row — sufficient for row-linking and cross-row wiring. the instance constraint verifies full noun identities against the first and last rows.

## instance (public input)

```
instance = (object_id, formula_id, result_id, status)

object_id:  F_p (NounId of the object noun)
formula_id: F_p (NounId of the formula noun)
result_id:  F_p (NounId of the result noun, or 0 on failure)
status:     F_p (0 = ok, 1 = halt, 2 = error)
```

the protocol invocation is order() -> result: the caller provides (object, formula), the executor returns result. when status = 0 (success), result_id is the NounId of the result noun. when status != 0 (halt or error), there is no result noun — result_id MUST be zero. the verifier enforces: `status != 0 => result_id = 0`.

the instance links the trace to the computation. the verifier checks:
1. first row: r1 matches instance object_id, r2 matches instance formula_id
2. last row: r3 matches instance result_id, status propagated correctly
3. status-gated result: if status = 0, result_id is a valid NounId; if status != 0, result_id = 0
4. noun store commitment: instance NounIds (object_id, formula_id, and result_id when status = 0) are checked against the noun store polynomial commitments. each NounId is hemera(Lens.commit(noun_polynomial) || domain_tag). the commitment scheme is defined by the proof system (zheng) — nox specifies WHAT is committed (the noun identities referenced by the trace), not HOW

## register layout

precise register assignments for each of the 18 patterns (0-15 compute, 16=call, 17=look).

every row follows this fixed structure:

```
r0:    pattern tag (0-17)
r1:    object NounId
r2:    formula NounId
r3:    result NounId
r4-r7: pattern-specific operands (meaning varies per pattern, see below)
r8:    budget before step
r9:    budget after step
r10-r15: reserved (zero-filled unless otherwise specified)
```

## per-pattern register map

exact register contents for each pattern (tags 0-17). r0-r3 and r8-r9 follow the fixed layout above. tables below specify r4-r7 (pattern-specific operands) and any reserved registers used by specific patterns.

### pattern 0: axis (single-row, cost 1)

```
r0  = 0
r1  = object NounId
r2  = formula NounId
r3  = result NounId               — NounId of the value at the addressed position
r4  = noun polynomial commitment  — Lens commitment to the object noun polynomial
r5  = axis index                  — evaluated axis address (integer)
r6  = evaluation point            — binary encoding of axis index for Lens opening
r7  = result value                — value at the addressed position

constraint: Lens opening proof verifies r7 = noun_poly(r6) (degree 1)
budget: r9 = r8 - 1
```

### pattern 1: quote (single-row, cost 1)

```
r0  = 1
r1  = object NounId
r2  = formula NounId
r3  = result NounId               — same as body NounId
r4  = body                        — the literal noun c from [1 c]
r5  = 0 (unused)
r6  = 0 (unused)
r7  = body                        — result = body (identity: r7 = r4)

constraint: r7 = r4 (degree 1)
budget: r9 = r8 - 1
```

### pattern 2: compose (single-row, cost 1)

```
r0  = 2
r1  = object NounId
r2  = formula NounId
r3  = result NounId               — NounId of reduce(r4, r5)
r4  = result of reduce(o, x)      — evaluated first sub-expression (new object)
r5  = result of reduce(o, y)      — evaluated second sub-expression (new formula)
r6  = NounId(x)                   — identity of first sub-formula (for wiring)
r7  = NounId(y)                   — identity of second sub-formula (for wiring)

constraint: r3 wired to result row of reduce(r4, r5) via CCS (degree 1)
budget: r9 = r8 - 1 (dispatch only; sub-expression costs in their own rows)
note: 3 reduce() calls total (x, y, final application) — sub-results in separate rows.
```

### pattern 3: cons (single-row, cost 1)

```
r0  = 3
r1  = object NounId
r2  = formula NounId
r3  = result NounId               — NounId of the constructed cell
r4  = result of reduce(o, a)      — evaluated head
r5  = result of reduce(o, b)      — evaluated tail
r6  = 0 (unused)
r7  = 0 (unused)

constraint: r3 = NounId(cell(r4, r5)) verified via hemera hash wiring (degree 1)
budget: r9 = r8 - 1
note: sub-expressions evaluated in separate rows; r4, r5 wired via CCS.
```

### pattern 4: branch (single-row, cost 1)

```
r0  = 4
r1  = object NounId
r2  = formula NounId
r3  = result NounId               — NounId of the chosen branch result
r4  = test value                   — result of reduce(o, test)
r5  = inverse of test value        — r4^-1 (non-deterministic hint, 0 when r4 = 0)
r6  = yes-branch result            — result of reduce(o, yes) (wired from sub-row)
r7  = no-branch result             — result of reduce(o, no) (wired from sub-row)
r10 = selector                     — 1 if r4 = 0 (take yes), 0 if r4 != 0 (take no)

constraint:
  r10 = 1 - r4 * r5               — selector computation (degree 2)
  r4 * r10 = 0                    — ensures selector is valid (degree 2)
  result = r10 * r6 + (1 - r10) * r7  — mux (degree 2)
budget: r9 = r8 - 1
note: only the chosen branch is evaluated; the unchosen register (r6 or r7) is zero.
```

### pattern 5: add (single-row, cost 1)

```
r0  = 5
r1  = object NounId
r2  = formula NounId
r3  = result NounId
r4  = left operand                 — evaluated first sub-expression
r5  = right operand                — evaluated second sub-expression
r6  = (r4 + r5) mod p              — field addition result
r7  = 0 (unused)

constraint: r6 = r4 + r5 (degree 1)
budget: r9 = r8 - 1
```

### pattern 6: sub (single-row, cost 1)

```
r0  = 6
r1  = object NounId
r2  = formula NounId
r3  = result NounId
r4  = left operand
r5  = right operand
r6  = (r4 - r5) mod p              — field subtraction result
r7  = 0 (unused)

constraint: r6 = r4 - r5, equivalently r6 + r5 = r4 (degree 1)
budget: r9 = r8 - 1
```

### pattern 7: mul (single-row, cost 1)

```
r0  = 7
r1  = object NounId
r2  = formula NounId
r3  = result NounId
r4  = left operand
r5  = right operand
r6  = (r4 * r5) mod p              — field multiplication result
r7  = 0 (unused)

constraint: r6 = r4 * r5 (degree 2)
budget: r9 = r8 - 1
```

### pattern 8: inv (multi-row, cost 64)

```
64 consecutive rows, all with r0 = 8.

row 0:
  r0  = 8
  r1  = object NounId
  r2  = formula NounId
  r3  = result NounId              — (set on final row)
  r4  = input value                — the value to invert
  r5  = 0 (unused)
  r6  = 0 (not yet computed)
  r7  = 0 (unused)
  r10 = 1                          — accumulator initialized to 1
  r11 = exponent bit 0             — LSB of p-2 (Fermat exponent)
  r12 = step counter               — 0

rows 1-62 (transition constraints):
  r0  = 8
  r1  = object NounId              — same across all rows
  r2  = formula NounId             — same across all rows
  r3  = result NounId              — (set on final row)
  r4  = input value                — same across all rows (copied)
  r5  = 0 (unused)
  r6  = 0 (not yet computed)
  r7  = 0 (unused)
  r10 = accumulator                — r10_{t+1} = r10_t^2 * (r11_t ? r4 : 1)
  r11 = exponent bit t             — bit t of p-2
  r12 = step counter               — t

row 63 (final):
  r0  = 8
  r1  = object NounId
  r2  = formula NounId
  r3  = result NounId
  r4  = input value
  r5  = 0 (unused)
  r6  = final inverse              — r6 = r10 (accumulator after 63 square-and-multiply steps)
  r7  = 0 (unused)
  r10 = final accumulator          — equal to r6
  r11 = exponent bit 63
  r12 = 63                         — step counter

transition constraint: r10_{t+1} = r10_t * r10_t * (r11_t * r4 + (1 - r11_t))  (degree 3)
final constraint: r6 * r4 = 1 (degree 2, on row 63 only)
error: if r4 = 0, status = error, error kind = 2 (inv_zero)
budget: r9 = r8 - 64 (on row 0; rows 1-63 have r8 = r9 = 0, budget tracked only at boundaries)
```

### pattern 9: eq (single-row, cost 1)

```
r0  = 9
r1  = object NounId
r2  = formula NounId
r3  = result NounId
r4  = left operand
r5  = right operand
r6  = 0 if r4 = r5, else 1         — equality result (0 = true)
r7  = (r4 - r5)^-1                 — inverse of difference (hint; 0 when r4 = r5)

constraint:
  (r4 - r5) * (1 - r6) = 0          — if r4 != r5, then r6 = 1 (degree 2)
  r6 * (1 - r6) = 0                 — r6 is boolean (degree 2)
  (r4 - r5) * r7 = r6               — r7 is inverse hint; forces r6 = 1 when unequal, r6 = 0 when equal
                                       (standard non-equality gadget, degree 2)
budget: r9 = r8 - 1
```

### pattern 10: lt (single-row, cost 1)

```
r0  = 10
r1  = object NounId
r2  = formula NounId
r3  = result NounId
r4  = left operand
r5  = right operand
r6  = 0 if r4 < r5, else 1         — less-than result (0 = true)
r7  = range decomposition limb 0   — bit/limb decomposition of (r4 - r5) mod p
r10 = range decomposition limb 1   — for non-native comparison in Goldilocks
r11 = borrow/sign bit              — determines comparison outcome

constraint: ~64 constraints for range decomposition (bit decomposition proves
            the difference fits the correct half of the field)
budget: r9 = r8 - 1
```

### pattern 11: xor (single-row, cost 1)

```
r0  = 11
r1  = object NounId
r2  = formula NounId
r3  = result NounId
r4  = left operand                 — word-type atom
r5  = right operand                — word-type atom
r6  = r4 XOR r5                    — 32-bit bitwise exclusive-or
r7  = bit decomposition witness 0  — packed bits of r4 (for constraint verification)
r10 = bit decomposition witness 1  — packed bits of r5

constraint: ~32 constraints (bit decomposition in F_p; in F_2 this is 1 constraint)
budget: r9 = r8 - 1
```

### pattern 12: and (single-row, cost 1)

```
r0  = 12
r1  = object NounId
r2  = formula NounId
r3  = result NounId
r4  = left operand                 — word-type atom
r5  = right operand                — word-type atom
r6  = r4 AND r5                    — 32-bit bitwise conjunction
r7  = bit decomposition witness 0
r10 = bit decomposition witness 1

constraint: ~32 constraints (bit decomposition in F_p)
budget: r9 = r8 - 1
```

### pattern 13: not (single-row, cost 1)

```
r0  = 13
r1  = object NounId
r2  = formula NounId
r3  = result NounId
r4  = operand                      — word-type atom
r5  = 0 (unused, unary operation)
r6  = NOT r4                       — 32-bit bitwise complement (r4 XOR 0xFFFFFFFF)
r7  = bit decomposition witness    — packed bits of r4

constraint: ~32 constraints (bit decomposition in F_p)
budget: r9 = r8 - 1
```

### pattern 14: shl (single-row, cost 1)

```
r0  = 14
r1  = object NounId
r2  = formula NounId
r3  = result NounId
r4  = value                        — word-type atom to shift
r5  = shift amount                 — word-type atom, must be in [0, 32)
r6  = (r4 << r5) mod 2^32         — left shift result (shifts >= 32 produce 0)
r7  = bit decomposition witness    — packed bits for shift verification

constraint: ~32 constraints (bit decomposition + shift verification in F_p)
budget: r9 = r8 - 1
```

### pattern 15: hash (multi-row, cost 200)

```
~200 consecutive rows, all with r0 = 15.

row 0 (absorption):
  r0  = 15
  r1  = object NounId
  r2  = formula NounId
  r3  = result NounId              — (set on final row)
  r4  = input value                — the noun to hash (Lens commitment or atom value)
  r5  = domain tag                 — domain separation constant (capacity[11])
  r6  = 0 (not yet computed)
  r7  = 0 (unused)
  r10 = sponge state[0]            — initialized from capacity constants
  r11 = sponge state[1]
  r12 = step counter               — 0

rows 1-198 (round state progression):
  r0  = 15
  r1  = object NounId              — same across all rows
  r2  = formula NounId             — same across all rows
  r3  = result NounId              — (set on final row)
  r4  = input value                — held constant (for wiring)
  r5  = 0 (unused after absorption)
  r6  = 0 (not yet computed)
  r7  = 0 (unused)
  r10 = round state element        — progresses through Poseidon2 rounds
  r11 = round state element        — (8 full rounds + 16 partial rounds = 24 total)
  r12 = step counter               — t

row 199 (squeeze / final):
  r0  = 15
  r1  = object NounId
  r2  = formula NounId
  r3  = result NounId
  r4  = input value
  r5  = 0 (unused)
  r6  = H(input)[0]                — first element of 4-element hash output
  r7  = H(input)[1]                — second element of hash output
  r10 = H(input)[2]                — third element
  r11 = H(input)[3]                — fourth element
  r12 = 199                        — step counter

transition constraints:
  full round rows: degree 7 (s-box x^7 applied to all state elements)
  partial round rows: degree 2 (s-box x^{-1} on one element, verified as x * y = 1)
budget: r9 = r8 - 200 (on row 0; internal rows do not independently track budget)
```

### pattern 16: call (single-row, cost 1)

```
r0  = 16
r1  = object NounId
r2  = formula NounId
r3  = result NounId               — NounId of reduce([witness, o], check_f) result
r4  = tag value                    — result of reduce(o, tag_f): identifies witness type
r5  = witness value                — non-deterministic: injected by prover via CallProvider
r6  = result                       — result of reduce([witness, o], check_f)
r7  = NounId(check_f)             — identity of the check formula (for wiring)

constraint: r6 wired to check formula result row via CCS; result must be 0 (degree 1)
budget: r9 = r8 - 1 (dispatch only; check cost in its own rows)
note: r5 is the ONLY non-deterministic column in the entire trace. the verifier
      checks constraint satisfaction via the stark proof, never executes the provider.
```

### pattern 17: look (single-row, cost 1)

```
r0  = 17
r1  = object NounId
r2  = formula NounId
r3  = result NounId               — NounId of the looked-up value
r4  = key value                    — result of reduce(o, key_f): the BBG lookup key
r5  = commitment root              — BBG sub-root commitment (NMT root for the namespace)
r6  = value                        — the looked-up value from BBG authenticated state
r7  = opening proof element 0      — NMT inclusion proof data (deterministic)
r10 = opening proof element 1
r11 = opening proof element 2

constraint: NMT/Lens opening proof verifies r6 at key r4 under root r5 (degree 1)
budget: r9 = r8 - 1
note: fully deterministic — r7, r10-r11 are the NMT inclusion proof, verifiable against
      the BBG state commitment in the instance. memoizable at a given block height.
```

### call vs look: trace difference

call (16) has one non-deterministic column (r5 = prover witness); look (17) is fully deterministic (r5 = BBG commitment root, r7/r10-r11 = NMT proof). call results are not memoizable; look results are memoizable at a given block height.

## constraint system

each pattern defines constraints over its trace row. single-row patterns use in-row constraints. multi-row patterns (inv, hash) use transition constraints across consecutive rows. SuperSpartan CCS handles both forms and mixed degrees natively.

constraint degrees and counts are per-instantiation. the structure of the constraint system (which registers constrain which patterns) is algebra-independent. the concrete degrees below refer to the canonical instantiation.

per-pattern constraints, register usage, and degrees are specified in the per-pattern register map above. summary of constraint degrees:

```
degree 1: axis, quote, add, sub, look (linear constraints)
degree 2: mul, eq, lt, branch, call, compose, cons (quadratic constraints)
degree 3: inv transition (square-and-multiply step)
degree 7: hash full rounds (x^7 s-box)
~32 constraints: xor, and, not, shl (bit decomposition in F_p; 1 in F₂)
~64 constraints: lt (range decomposition in Goldilocks)
```

## single-row vs multi-row patterns

most patterns produce exactly 1 trace row per reduce() call. two patterns produce multiple rows:

```
single-row (cost 1): 0-7, 9-14, 16, 17 (axis through mul, eq through shl, call, look)
multi-row:           8 inv (64 rows), 15 hash (~200 rows)
```

see per-pattern register map above for row-by-row layout of multi-row patterns.

## row linking

consecutive rows are linked by constraints on object, formula, and result NounIds:

```
same object:  r1_{t+1} = r1_t
new object:   r1_{t+1} set by compose result NounId
same formula: r2_{t+1} = r2_t
new formula:  r2_{t+1} set by sub-expression dispatch
result link:  r3_t wired to the result NounId of the completed sub-computation
```

## budget decrement constraints

```
single-row patterns:  r9 = r8 - 1            (1 per reduce call)
axis:                 r9 = r8 - 1            (1 — O(1) Lens opening)
inv:                  r9 = r8 - 64           (64 for square-and-multiply)
hash:                 r9 = r8 - 200          (200 for Poseidon2 hemera)
```

## error and halt encoding

status is encoded in the instance. each row's status is implicit from the budget and pattern constraints.

```
status = 0: ok — pattern completed successfully
status = 1: halt — budget exhausted (r8 < cost)
status = 2: error — type/semantic error

when status = 1 (halt):
  r8 = remaining budget (insufficient for next pattern)
  r3 = 0 (no result NounId)

when status = 2 (error):
  r10 = error kind (0 = type error, 1 = axis on atom, 2 = inv(0), 3 = unavailable, 4 = malformed)
  r3 = 0 (no result NounId)
```

errors and halts propagate: once status != 0, subsequent rows maintain the same status.

## encoding as multilinear polynomial

the entire trace (2^n rows × 16 columns) encodes as one multilinear polynomial:

```
f(x_1, ..., x_{n+4}) : F_p → F_p

where:
  x_1..x_n   select the row (step index in binary)
  x_{n+1}..x_{n+4}  select the column (register index)
```

Brakedown commits to f. sumcheck verifies transition constraints. the verifier checks O(log n) evaluation queries.

## self-verification

the stark verifier is a nox program. with Brakedown (Merkle-free PCS), the verifier is pure field arithmetic — no Merkle paths, no FRI folding. Fiat-Shamir via hemera hash is the only non-field operation.

canonical verifier cost (from zheng specs):

```
tier                        constraints
──────────────────────────  ───────────
generic (no jets)              ~8,000
CCS jet + batch Brakedown        ~825
+ algebraic Fiat-Shamir            ~89
```

see zheng/specs/verifier.md for the canonical breakdown.
recursive composition: proof-of-proof at every block. constant proof size (~2 KiB with zheng). per-fold cost: ~30 field ops + 1 hemera hash.
