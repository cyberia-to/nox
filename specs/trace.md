# execution trace specification

version: 0.3
status: canonical

## overview

the nox execution trace is the sequence of register states across all reduction steps. it IS the zheng witness — no separate arithmetization. each row is one reduction step. each column is a register.

the trace layout is algebra-independent in structure (16 registers, power-of-2 rows) but per-instantiation in element size (each register holds one element of the instantiated field F). all concrete details below refer to the canonical instantiation: nox<Goldilocks, Z/2^32, Hemera>.

## trace layout

```
columns (16 = 2⁴ registers, each one F element):
  r0:   pattern tag (0-17)
  r1:   object particle             — identity of the object data
  r2:   formula particle            — identity of the formula data
  r3:   result particle             — identity of the result data
  r4:   pattern-specific operand 0
  r5:   pattern-specific operand 1
  r6:   pattern-specific operand 2
  r7:   pattern-specific operand 3
  r8:   budget before step
  r9:   budget after step
  r10:  error kind on error rows; pattern-specific witness on success rows (inv: accumulator, branch: selector)
  r11:  multi-row state / pattern-specific / Lens commitment bytes (when prover active)
  r12:  multi-row state / pattern-specific / Lens commitment bytes (when prover active)
  r13:  multi-row state / pattern-specific / Lens commitment bytes (when prover active)
  r14:  pattern-specific (round index for hash/inv)
  r15:  pattern-specific (input particle for hash)

rows: 2^n (padded to power of 2)
```

the particle is the content identity of a data node — its 32-byte hemera hash (4 field elements). each register holds one field element, so r1–r3 carry a compact field-element reference to each data node's particle, sufficient for row-linking and cross-row wiring. the instance (public input) carries the full particles; the instance constraint verifies the full identities against the first and last rows.

## instance (public input)

```
instance = (object_id, formula_id, result_id, status)

object_id:  F_p (particle of the object data)
formula_id: F_p (particle of the formula data)
result_id:  F_p (particle of the result data, or 0 on failure)
status:     F_p (0 = ok, 1 = halt, 2 = error)
```

the protocol invocation is order() -> result: the caller provides (object, formula), the executor returns result. when status = 0 (success), result_id is the particle of the result data. when status != 0 (halt or error), there is no result data — result_id MUST be zero. the verifier enforces: `status != 0 => result_id = 0`.

the instance links the trace to the computation. the verifier checks:
1. first row: r1 matches instance object_id, r2 matches instance formula_id
2. last row: r3 matches instance result_id, status propagated correctly
3. status-gated result: if status = 0, result_id is a valid particle; if status != 0, result_id = 0
4. data store commitment: instance particles (object_id, formula_id, and result_id when status = 0) are checked against the data store polynomial commitments. each particle is hemera(Lens.commit(data_polynomial) || domain_tag). the commitment scheme is defined by the proof system (zheng) — nox specifies WHAT is committed (the data identities referenced by the trace), not HOW

## register layout

precise register assignments for each of the 18 patterns (0-15 compute, 16=call, 17=look).

every row follows this fixed structure:

```
r0:    pattern tag (0-17)
r1:    object particle
r2:    formula particle
r3:    result particle
r4-r7: pattern-specific operands (meaning varies per pattern, see below)
r8:    budget before step
r9:    budget after step
r10:   error kind on error rows (0-5); pattern-specific witness on success rows
r11-r15: pattern-specific (Lens commitment bytes, round state, input wiring — zero when unused)
```

## per-pattern register map

exact register contents for each pattern (tags 0-17). r0-r3 and r8-r9 follow the fixed layout above. tables below specify r4-r7 (pattern-specific operands) and any reserved registers used by specific patterns.

### pattern 0: axis (single-row, cost 1)

```
r0  = 0
r1  = object particle
r2  = formula particle
r3  = result particle               — particle of the value at the addressed position
r4  = commitment[0] from axis_commitment() when prover is active, 0 otherwise
      (first 8-byte LE limb of the 32-byte Lens commitment; equals r11)
r5  = axis address (raw u64)      — literal address from formula body
r6  = levels descended (depth of navigation)
r7  = result particle               — particle of the value at the addressed position
r8  = budget_in
r9  = budget_out = r8 - 1
r10 = 0 (success) / error kind
r11 = commitment[0] (bytes 0..8 of Lens commitment, LE u64) — same as r4
r12 = commitment[1] (bytes 8..16)
r13 = commitment[2] (bytes 16..24)
r14 = commitment[3] (bytes 24..32)
r15 = 0 (reserved)

r4 and r11-r14 are non-zero only when the executor's CallProvider implements axis_commitment().
when absent (NullCalls / interpreter mode) they are zero; zheng skips commitment binding.

constraint: r9 = r8 - 1 (budget, degree 1)
            Lens opening verified via axis_acc (separate fold sequence in zheng)
budget: r9 = r8 - 1
note: circuit cost is 1 Lens opening (O(1)). interpreter traverses up to 63 tree
      levels (bounded by u64 address bits) — not a circuit concern.
```

### pattern 1: quote (single-row, cost 1)

```
r0  = 1
r1  = object particle
r2  = formula particle
r3  = result particle               — same as body particle
r4  = body                        — the literal data c from [1 c]
r5  = 0 (unused)
r6  = 0 (unused)
r7  = body                        — result = body (identity: r7 = r4)

constraint: r7 = r4 (degree 1)
budget: r9 = r8 - 1
```

### pattern 2: compose (single-row, cost 1)

```
r0  = 2
r1  = object particle
r2  = formula particle
r3  = result particle               — particle of reduce(r4, r5)
r4  = result of reduce(o, x)      — evaluated first sub-expression (new object)
r5  = result of reduce(o, y)      — evaluated second sub-expression (new formula)
r6  = particle(x)                   — identity of first sub-formula (for wiring)
r7  = particle(y)                   — identity of second sub-formula (for wiring)

constraint: r3 wired to result row of reduce(r4, r5) via CCS (degree 1)
budget: r9 = r8 - 1 (dispatch only; sub-expression costs in their own rows)
note: 3 reduce() calls total (x, y, final application) — sub-results in separate rows.
```

### pattern 3: cons (single-row, cost 1)

```
r0  = 3
r1  = object particle
r2  = formula particle
r3  = result particle               — particle of the constructed pair
r4  = result of reduce(o, a)      — evaluated head
r5  = result of reduce(o, b)      — evaluated tail
r6  = 0 (unused)
r7  = 0 (unused)

constraint: r3 = particle(pair(r4, r5)) verified via hemera hash wiring (degree 1)
budget: r9 = r8 - 1
note: sub-expressions evaluated in separate rows; r4, r5 wired via CCS.
```

### pattern 4: branch (single-row, cost 1)

```
r0  = 4
r1  = object particle
r2  = formula particle
r3  = result particle               — particle of the chosen branch result
r4  = test value                   — result of reduce(o, test)
r5  = inverse hint of r4           — r4^-1 when r4 ≠ 0, else 0
r6  = chosen arm result            — particle of the chosen branch result
r10 = selector                     — 0 if r4 = 0 (take yes), 1 if r4 ≠ 0 (take no)

constraint:
  selector * (1 - r4 * r5) = 0   — binds selector to "test != 0" (degree 2)
  r4 * (1 - r10) = 0             — when test=0, selector must be 0 (degree 2)
  result = chosen arm result
budget: r9 = r8 - 1
note: only the chosen branch is evaluated. r6 = particle of chosen result.
      r7 is unused (the unchosen arm is not evaluated, not stored).
```

### pattern 5: add (single-row, cost 1)

```
r0  = 5
r1  = object particle
r2  = formula particle
r3  = result particle
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
r1  = object particle
r2  = formula particle
r3  = result particle
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
r1  = object particle
r2  = formula particle
r3  = result particle
r4  = left operand
r5  = right operand
r6  = (r4 * r5) mod p              — field multiplication result
r7  = 0 (unused)

constraint: r6 = r4 * r5 (degree 2)
budget: r9 = r8 - 1
```

### pattern 8: inv (multi-row, cost 64)

64 consecutive rows, all with r0 = 8. MSB-first square-and-multiply for the
Fermat exponent p-2 = 0xFFFFFFFE_FFFFFFFF (all bits set except bit 32).

Bit 63 (MSB) of p-2 is always 1 → the accumulator initializes to `v` at
row 0 (= v^1), then 63 transition rows process bits 62 down to 0.

```
row 0 (init, bit 63 processed):
  r0  = 8
  r1  = object particle              — same across all rows
  r2  = formula particle             — same across all rows
  r3  = 0                          — result particle set on final row only
  r4  = input value v              — constant across the 64-row block
  r5  = 0 (unused)
  r6  = 0 (not yet computed)
  r7  = 0 (unused)
  r8  = budget_in (constant across the block)
  r9  = 0 on rows 0..62; budget_out on row 63
  r10 = v                          — accumulator = v (= v^1, matches MSB=1)
  r11 = 1                          — exponent bit at step 0 (MSB of p-2 is 1)
  r12 = 0                          — step counter

rows 1..62 (transition; step t processes bit 63 - t):
  r4  = v                          — copied
  r10 = accumulator after step t   — r10_t = r10_{t-1}^2 * (r11_t ? v : 1)
  r11 = bit (63 - t) of p-2        — exponent bit consumed this step
  r12 = t                          — step counter

row 63 (final, step 63 processes bit 0):
  r3  = result particle              — atom(r10, Field)
  r4  = v
  r6  = final inverse v^(p-2)      — equals r10
  r9  = budget_out                 — = r8 - 64
  r10 = v^(p-2) = v^-1             — final accumulator
  r11 = bit 0 of p-2 = 1
  r12 = 63

transition constraint (rows 1..63): r10_t = r10_{t-1}^2 * (r11_t * v + (1 - r11_t))  (degree 3)
init constraint (row 0):            r10_0 = v ∧ r11_0 = 1
final constraint (row 63):          r6 * v = 1  (degree 2)
error: if r4 = 0, status = error, error kind = 2 (inv_zero)
budget: r8 carries budget_in across all rows; r9 set only on the final row.
note: bit ordering is MSB-first. bit 63 of p-2 = 1 is processed at step 0.
      bit 32 of p-2 = 0 (the one unset bit). bits 31-0 of p-2 = 1 complete the exponent.
      r8 carries budget_in across all 64 rows; r9 is set only on the final row (row 63).
```

### pattern 9: eq (single-row, cost 1)

```
r0  = 9
r1  = object particle
r2  = formula particle
r3  = result particle
r4  = left operand field value       — for atoms: field value; for pairs: digest[0] of left data
r5  = right operand field value      — for atoms: field value; for pairs: digest[0] of right data
r6  = result (0 if equal, 1 if not equal)
r7  = (r4 - r5)^-1                  — inverse of difference (hint; 0 when r4 = r5)

note: for pairs, r4 = digest[0] of left data, r5 = digest[0] of right data.
      structurally equal pairs always produce equal digests across all four limbs.

constraint:
  (r4 - r5) * (1 - r6) = 0          — if r4 != r5, then r6 = 1 (degree 2)
  r6 * (1 - r6) = 0                 — r6 is boolean (degree 2)
  (r4 - r5) * r7 = r6               — r7 is inverse hint; forces r6 = 1 when unequal, r6 = 0 when equal
                                       (standard non-equality gadget, degree 2)
budget: r9 = r8 - 1
```

### pattern 10: lt (multi-row, cost 64)

64 consecutive rows, all with r0 = 10. one row per bit of the 64-bit
canonical Goldilocks representative, LSB-first.

```
each row k (k = 0..63):
  r0  = 10
  r1  = object particle
  r2  = formula particle
  r3  = 0 except final row (k = 63): result particle
  r4  = left operand a (full u64, constant across the 64-row block)
  r5  = right operand b (full u64, constant across the block)
  r6  = comparison result (0 if a < b under canonical order, else 1)
  r7  = bit position k
  r8  = budget_in (constant across the block)
  r9  = 0 except final row: budget_out
  r10 = a_k = (a >> k) & 1
  r11 = b_k = (b >> k) & 1

constraints (per row):
  - r10, r11 ∈ {0, 1}                (boolean check on each bit)
  - r7_t = t                          (row index = bit position)

constraints (across the 64-row block):
  - sum_{k=0..63} 2^k * r10_k = r4    (a decomposition binding)
  - sum_{k=0..63} 2^k * r11_k = r5    (b decomposition binding)
  - r6 follows from first-differing-bit gadget over the block (MSB-first
    "decided" accumulator) — see zheng for the canonical encoding

budget: r9 = r8 - 64 on final row only.
note: r7, r10, r11 (bit decomposition witnesses): populated by zheng witness generator,
      not by the nox interpreter. the interpreter writes only r4 (left operand) and r5 (right operand).
      r6 (result) is written by the interpreter. r7/r10/r11 are filled by zheng during witness extension.
```

### pattern 11: xor (multi-row, cost 32)

32 consecutive rows, all with r0 = 11. one row per bit of the 32-bit
word, LSB-first.

```
each row k (k = 0..31):
  r0  = 11
  r1  = object particle
  r2  = formula particle
  r3  = 0 except final row (k = 31): result particle
  r4  = left operand a (full 32-bit, constant across the block)
  r5  = right operand b (full 32-bit, constant across the block)
  r6  = c = a XOR b (full 32-bit, constant across the block)
  r7  = bit position k
  r8  = budget_in (constant across the block)
  r9  = 0 except final row: budget_out
  r10 = a_k = (a >> k) & 1
  r11 = b_k = (b >> k) & 1
  r12 = c_k = a_k XOR b_k

constraints (per row):
  - r10, r11, r12 ∈ {0, 1}
  - r12 = r10 + r11 - 2 * r10 * r11    (XOR gadget, degree 2)
  - r7_t = t

constraints (across the block):
  - sum_{k=0..31} 2^k * r10_k = r4
  - sum_{k=0..31} 2^k * r11_k = r5
  - sum_{k=0..31} 2^k * r12_k = r6

budget: r9 = r8 - 32 on final row only.
note: r7, r10, r12 (bit decomposition witnesses): populated by zheng witness generator,
      not by the nox interpreter. the interpreter writes r4, r5, r6 only.
```

### pattern 12: and (multi-row, cost 32)

32 rows, identical layout to xor with the per-row gadget replaced.

```
registers same as pattern 11 with r0 = 12.

constraints (per row):
  - r10, r11, r12 ∈ {0, 1}
  - r12 = r10 * r11                    (AND gadget, degree 2)

decomposition and budget constraints: identical to xor.
note: r7, r10, r11, r12 (bit decomposition witnesses): populated by zheng witness generator,
      not by the nox interpreter. the interpreter writes r4, r5, r6 only.
```

### pattern 13: not (multi-row, cost 32)

32 rows. unary: r5 and r11 are zero on every row.

```
each row k (k = 0..31):
  r0  = 13
  r1  = object particle
  r2  = formula particle
  r3  = 0 except final row: result particle
  r4  = operand a (full 32-bit)
  r5  = 0                             (unused — operation is unary)
  r6  = c = NOT a (= a XOR 0xFFFFFFFF, full 32-bit)
  r7  = bit position k
  r8  = budget_in
  r9  = 0 except final row: budget_out
  r10 = a_k
  r11 = 0                             (unused)
  r12 = c_k = 1 - a_k

constraints (per row):
  - r10, r12 ∈ {0, 1}
  - r12 = 1 - r10                     (NOT gadget, degree 1)
  - r11 = 0

decomposition (across the block): sum 2^k * r10 = r4, sum 2^k * r12 = r6.
budget: r9 = r8 - 32 on final row only.
note: r7, r10, r12 (bit decomposition witnesses): populated by zheng witness generator,
      not by the nox interpreter. the interpreter writes r4, r6 only.
```

### pattern 14: shl (multi-row, cost 32)

32 rows, one per output bit position. asymmetric: input bit decomposition
is partial — only positions that feed the output are exposed.

```
each row k (k = 0..31, k = output bit position):
  r0  = 14
  r1  = object particle
  r2  = formula particle
  r3  = 0 except final row: result particle
  r4  = a (input value)
  r5  = n (shift amount)
  r6  = c = (a << n) & 0xFFFFFFFF (output, constant across the block)
  r7  = k (output bit position)
  r8  = budget_in
  r9  = 0 except final row: budget_out
  r10 = a_k                           (bit k of input — for a's decomp)
  r11 = src_bit                       (= a_{k-n} when k ≥ n and k-n < 32, else 0)
  r12 = c_k                           (output bit; = r11 by definition)
  r13 = src_idx                       (= k - n when in range, else 32 sentinel)

constraints (per row):
  - r10, r11, r12 ∈ {0, 1}
  - r12 = r11                         (output bit equals source bit)
  - src_idx relationship: r13 = k - n when k ≥ n and k - n < 32,
                          else r13 = 32 (sentinel)
  - r11 ties to r10 at source row r13 across the block (cross-row equality)

decomposition (across the block):
  - sum 2^k * r10_k = r4              (input decomp)
  - sum 2^k * r12_k = r6              (output decomp)

budget: r9 = r8 - 32 on final row only.
note: r7, r10, r11, r13 (bit decomposition witnesses): populated by zheng witness generator,
      not by the nox interpreter. the interpreter writes r4 (input), r5 (shift amount), r6 (output) only.
```

### pattern 15: hash (multi-row, cost 25)

25 consecutive rows, all with r0 = 15. Driven by [`hemera::StepSponge`]:
24 Poseidon2 round rows (k = 0..23) plus a final squeeze row (k = 24).

The input data's cached structural digest (4 Goldilocks elements) is packed
into the 8-element sponge rate; capacity slots are zero. Each round row
records the post-round state. The squeeze row records the result particle
and the 4-element output digest extracted from the final state.

```
each round row k (k = 0..23):
  r0  = 15
  r1  = object particle              — constant across the block
  r2  = formula particle             — constant across the block
  r3  = 0 (set only on squeeze row)
  r4..r7  = state[0..3]            — first 4 elements of post-round state
  r8  = budget_in                  — constant across the block
  r9  = 0 (set only on squeeze row)
  r10..r13 = state[4..7]           — remaining rate elements
  r14 = round index k              — 0..23
  r15 = input particle               — constant (for wiring)

squeeze row (k = 24):
  r0  = 15
  r1  = object particle
  r2  = formula particle
  r3  = result particle              — hash-data pair(pair(h0,h1), pair(h2,h3))
  r4..r7 = output digest h0..h3    — first 4 elements of final post-permutation state
  r8  = budget_in
  r9  = budget_out                 — = r8 - 25
  r10..r13 = final_state[4..7]
  r14 = 24                         — squeeze sentinel
  r15 = input particle

transition constraints (rows 1..23):
  full round (k ∈ {0,1,2,3,20,21,22,23}): add round constants on all 16 elements,
    apply x^7 S-box (degree 7), MDS-light permutation
  partial round (k ∈ 4..19): add constant to state[0] only, apply x^{-1}
    S-box on state[0] (degree 2 via y*state[0] = 1), matmul_internal

init constraint (row 0): state initialized from rate input + initial MDS-light
final constraint (squeeze): r3 = order.hash_data(state[0..4])
budget: r9 = r8 - 25 on squeeze row only.
note: current implementation: single summary row; multi-row round-progression trace
      (25 rows with full state) is deferred to step-level hemera API integration.
      the result IS correct; only the trace witness is incomplete.
      the 25-row layout above describes the target; the interpreter today emits 1 row.
```

### pattern 16: call (single-row, cost 1)

```
r0  = 16
r1  = object particle
r2  = formula particle
r3  = result particle               — particle of reduce([witness, o], check_f) result
r4  = tag value                    — result of reduce(o, tag_f): identifies witness type
r5  = witness particle               — non-deterministic: injected by prover via CallProvider
r6  = result                       — result of reduce([witness, o], check_f)
r7  = particle(check_f)             — identity of the check formula (for wiring)

constraint: r6 wired to check formula result row via CCS; result must be 0 (degree 1)
budget: r9 = r8 - 1 (dispatch only; check cost in its own rows)
note: r5 is the ONLY non-deterministic column in the entire trace. the verifier
      checks constraint satisfaction via the zheng proof, never executes the provider.
```

### pattern 17: look (single-row, cost 1)

```
r0  = 17
r1  = object particle
r2  = formula particle
r3  = result particle               — particle of the looked-up value
r4  = C_t[0]                      — first limb of BBG commitment root (from object axis 4)
r5  = ns                           — namespace (result of reduce(o, ns_f))
r6  = key                          — lookup key (result of reduce(o, key_f))
r7  = value                        — result (u64 of Goldilocks element), or NIL if unavailable
r11 = C_t[1]                       — second limb of BBG root (from object axis 10)
r12 = C_t[2]                       — third limb of BBG root (from object axis 22)
r13 = C_t[3]                       — fourth limb of BBG root (from object axis 23)

constraint: Lens.verify(C_t, ns, key, r7) — inline wiring constraints
            Brakedown opening verification via folded CCS sub-instance (deferred to zheng)
budget: r9 = r8 - 1
note: fully deterministic — r4/r11-r13 form the full BBG commitment root (4 × u64 LE).
      Brakedown opening proof assembly is deferred to zheng witness generator
      (collect_look_openings() reads r4..r7 and r11..r13 after execution).
      memoizable at a given block height.
```

### call vs look: trace difference

call (16) has one non-deterministic column (r5 = prover witness); look (17) is fully deterministic (r5 = BBG_root, r7/r10-r11 = Brakedown/Lens opening proof). call results are not memoizable; look results are memoizable at a given block height.

### prover-side witness extensions

bit-decomposition witnesses (r7/r10 for patterns lt/xor/and/not/shl) and lens-opening columns (axis, look) are populated by the zheng witness generator, not by the nox interpreter. the interpreter fills result and operand registers only.

## constraint system

each pattern defines constraints over its trace row. single-row patterns use in-row constraints. multi-row patterns (inv, hash) use transition constraints across consecutive rows. SuperSpartan CCS handles both forms and mixed degrees natively.

constraint degrees and counts are per-instantiation. the structure of the constraint system (which registers constrain which patterns) is algebra-independent. the concrete degrees below refer to the canonical instantiation.

per-pattern constraints, register usage, and degrees are specified in the per-pattern register map above. summary of constraint degrees:

```
degree 1: axis, quote, add, sub (linear constraints)
degree 1 inline + folded sub-instance: look (wiring constraints inline; Brakedown verification folded via HyperNova)
degree 2: mul, eq, lt, branch, call, compose, cons (quadratic constraints)
degree 3: inv transition (square-and-multiply step)
degree 7: hash full rounds (x^7 s-box)
~32 constraints: xor, and, not, shl (bit decomposition in F_p; 1 in F₂)
~64 constraints: lt (range decomposition in Goldilocks)
```

## single-row vs multi-row patterns

most patterns produce exactly 1 trace row per reduce() call. two patterns produce multiple rows:

```
single-row (cost 1): 0-7, 9, 16, 17 (axis through mul, eq, call, look)
multi-row:           8 inv (64 rows), 10 lt (64 rows),
                     11 xor (32 rows), 12 and (32 rows),
                     13 not (32 rows), 14 shl (32 rows),
                     15 hash (25 rows: 24 rounds + 1 squeeze)
```

see per-pattern register map above for row-by-row layout of multi-row patterns.

## row linking

consecutive rows are linked by constraints on object, formula, and result particles:

```
same object:  r1_{t+1} = r1_t
new object:   r1_{t+1} set by compose result particle
same formula: r2_{t+1} = r2_t
new formula:  r2_{t+1} set by sub-expression dispatch
result link:  r3_t wired to the result particle of the completed sub-computation
```

note: the row-linking constraints (r1_{t+1}=r1_t etc.) are verified by zheng, not enforced at
trace-emit time by the interpreter. the interpreter populates the registers; zheng checks the linking.

## budget decrement constraints

```
single-row patterns:  r9 = r8 - 1            (1 per reduce call)
axis:                 r9 = r8 - 1            (1 — O(1) Lens opening)
inv:                  r9 = r8 - 64           (64 for square-and-multiply)
hash:                 r9 = r8 - 25           (24 Poseidon2 rounds + 1 squeeze)
```

## error and halt encoding

status is encoded in the instance. each row's status is implicit from the budget and pattern constraints.

```
status = 0: ok — pattern completed successfully
status = 1: halt — budget exhausted (r8 < cost)
status = 2: error — type/semantic error

when status = 1 (halt):
  r8 = remaining budget (insufficient for next pattern)
  r3 = 0 (no result particle)

when status = 2 (error):
  r10 = error kind:
    0 = type error    — wrong tag for operation
    1 = axis error    — axis navigation on atom
    2 = inv zero      — inversion of zero
    3 = unavailable   — resource exhausted (order full, provider returned nothing)
    4 = malformed     — formula structure invalid or depth limit exceeded
    5 = call rejected — call pattern: check formula returned non-zero or error
  r3 = 0 (no result particle)
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

the zheng verifier is a nox program. with Brakedown (Merkle-free PCS), the verifier is pure field arithmetic — no Merkle paths, no FRI folding. Fiat-Shamir via hemera hash is the only non-field operation.

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
