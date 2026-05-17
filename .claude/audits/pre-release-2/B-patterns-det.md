# pre-release-2 audit: deterministic patterns (B)

scope: rs/patterns/{axis,quote,compose,cons,branch,add,sub,mul,inv,eq,lt,xor,and,not,shl,mod}.rs
ground truth: specs/trace.md per-pattern register map, specs/patterns/*.md
verified: P_MINUS_2 = 0xFFFFFFFEFFFFFFFF == Goldilocks p - 2 (correct, 64 bits, bit-32 cleared)

## blocker

(none)

## major

- [pass 1, pass 3] inv.rs:46-53 — accumulator init deviates from spec trace layout: code initializes acc=v processing bit 63 first (left-to-right, MSB), spec/trace.md row 0 says r10=1 (acc starts at 1) and r11 = LSB of p-2 (right-to-left, LSB). Algorithm is mathematically correct (computes v^(p-2)) but the trace will not satisfy the spec's transition constraint `r10_{t+1} = r10_t^2 * (r11_t*r4 + (1-r11_t))` interpreted with LSB-first bit ordering. fix: either rewrite to LSB-first square-and-multiply matching spec, or update specs/trace.md §pattern 8 to MSB-first and document r10 row 0 = v, r11 row 0 = 1.
- [pass 1, pass 9] branch.rs:29-31 — r4 set to NounId of test_result, r5 to test_value, r6 to chosen formula NounId. Spec/trace.md §pattern 4 mandates r4 = test value, r5 = inverse hint of r4, r6 = yes-branch result NounId, r7 = no-branch result NounId, r10 = selector. fix: r4 = test_value; compute r5 as inverse of r4 when r4 != 0 (else 0); place result NounId in r6 if selector or r7 otherwise; set r10 = selector.
- [pass 1, pass 9] axis.rs:19,54-56 — r4 written twice (once before match at line 19 with addr, then again at line 54), and registers diverge from spec/trace.md §pattern 0 (r4 = noun-poly commitment, r5 = axis index, r6 = evaluation point, r7 = result value). Code uses r4=addr, r5=addr, r6=levels, r7=node-as-u64. fix: r4 = noun poly commitment (or 0 placeholder if not yet wired), r5 = addr, r6 = binary evaluation point encoding of addr, r7 = canonical value at addressed position (atom value or NounId; pick one and document).
- [pass 1, pass 9] eq.rs:35-37 — r4/r5 set to NounIds (ra, rb), but spec/trace.md §pattern 9 says r4 = left operand VALUE, r5 = right operand VALUE, r6 = result, r7 = (r4 - r5)^-1 hint. r7 left unset (will be 0 even when operands differ), which violates the non-equality gadget constraint `(r4 - r5) * r7 = r6`. fix: evaluate operands as field elements, store values in r4/r5, compute r7 = (r4 - r5).inverse() when unequal else 0.
- [pass 3, pass 12] inv.rs tests — only inv(2) and inv(0) tested; no coverage for inv(1), inv(p-1), or inv at random elements with multiplicative-inverse round-trip across the field. Risk: a 1-bit off in P_MINUS_2 traversal would still pass inv(2)*2=1 if it accidentally computed v^(p-2±2k). fix: add property test: for k in [1, 2, p/2, p-1, random sample], assert inv(k) * k == 1.

## minor

- [pass 1] inv.rs:27,39 — error rows write `row.r[10] = ErrorKind as u64`, but r10 is the accumulator register in spec. error kind belongs in r10 only when status=error per trace.md §error encoding — that overload is documented for the LAST row; intermediate rows in patterns that haven't started multi-row emission should not stomp accumulator semantics. fix: keep error-kind in r10 only on a dedicated terminal error row with r0=8 and r12 marking it as terminal.
- [pass 9] inv.rs:25-32 — error path before zero-check still emits a row with NIL in r3 and 0 in r9; trace.md error-encoding says r3 = 0 (not NIL) on error. NIL is a non-zero NounId. fix: row.r[3] = 0 on error.
- [pass 9, pass 5] axis.rs:38-58 — addr=2..=u64::MAX path uses `bits = 64 - addr.leading_zeros() - 1` which yields 63 when addr's MSB is set. For addr = 1 the special-case at line 34 catches it; for addr ≥ 2, bits ≥ 1. OK. But `levels` is recorded in r6 — semantically meaningful but undocumented. fix: add a comment cross-referencing spec/trace.md §pattern 0; preferably store binary-encoded evaluation point per spec.
- [pass 9] compose.rs:24-27 — r4/r5/r6/r7 set, matches spec (r4=reduce(o,x), r5=reduce(o,y), r6=NounId(x), r7=NounId(y)). minor: comment that r4/r5 are NounIds of reduced sub-expressions, not raw values.
- [pass 9] cons.rs:23-24 — only r4/r5 set, r6/r7 default to 0 (matches spec). add comment.
- [pass 9] xor/and/mul/add/sub.rs — r7 left unset by field_binary_op/word_binary_op; spec/trace.md says r7 = 0 for add/sub/mul (good), but for xor/and/not/shl r7 = bit-decomposition witness. Acceptable: prover fills witness columns; interpreter only computes the result value. fix: add comment in word_binary_op/not/shl that bit-decomp witnesses (r7, r10) are populated by the prover, not the interpreter.
- [pass 9] lt.rs:25-28 — `va.as_u64() < vb.as_u64()` evaluated twice; minor compactness. fix: bind once: `let lt = va.as_u64() < vb.as_u64();`.
- [pass 10] eq.rs / lt.rs — both files duplicate the make_field-on-Goldilocks::ZERO/ONE branch logic. fix: extract `make_bool_field(order, cond, budget)` helper in reduce.rs.
- [pass 10] add/sub/mul.rs and xor/and.rs — each file is a 1-line dispatch + ~40 lines of tests. consider consolidating tests into a parametric harness (one file with all field binops). minor; current layout is more readable.
- [pass 12] shl.rs — `shl_overflow_clamps_to_zero` tests vn=32 only. fix: add tests for vn=33, vn=63, vn=64, vn=u32::MAX to confirm the `>= 32` guard catches all overflow cases including the Rust UB region (`u64 << 64` is UB).
- [pass 12] not.rs — `not_involution` test name promises double-not but only does single. fix: rename to `not_inverts_pattern` or actually compose two nots.
- [pass 12] axis.rs — no test for axis on cell with addr=0 (hash introspection on cell); only atom case at line 117. fix: add cell variant.
- [pass 12] axis.rs — no test for axis with very large addr (e.g., 0xFFFF...) hitting AxisError mid-traversal. fix: add adversarial test.
- [pass 12] add/sub/mul — no edge tests at field boundary (p-1, 2*(p-1) wrap, mul(p-1, p-1)). nebu handles reduction but the integration is untested here. fix: add `add(p-1, 1) == 0`, `sub(0, 1) == p-1`, `mul(p-1, p-1) == 1`.
- [pass 12] eq.rs — only tests atom equality; no test for cell equality (the digest-comparison path's main reason for existing). fix: add `eq(cell(1,2), cell(1,2)) == 0` and `eq(cell(1,2), cell(1,3)) == 1`.
- [pass 11] inv.rs:64-74 — each step allocates a fresh `TraceRow::default()` and copies r0/r1/r2/r8. 64 allocations per inv call. Each TraceRow is on the stack (struct), so no heap alloc, but the pattern is verbose. fix: mutate `row` in place across steps; only the final allocation needs `order.atom()`.
- [pass 2] all field/word patterns — read-set is bounded by the formula tree depth via `evaluate` recursion. Depth limit enforced by reduce_inner (not visible here). assumed OK; cross-check that reduce_inner has explicit depth cap.
- [pass 4] eq.rs — digest comparison relies on Hemera collision resistance. For deterministic semantics this is sound; document that eq on cells is hash-based, not structural recursion (relevant if someone replaces the hash later).
- [pass 5] axis.rs:56 — `row.r[7] = node as u64` casts a NounId (u32?) to u64. If NounId ever widens past 64 bits the cast silently truncates. fix: use `NounId::from(node).as_u64()` or assert width in a const block.
- [pass 6] inv.rs:86-91 — Unavailable on final atom allocation emits a step_row but the partial trace from steps 1..62 was already recorded. consumer sees 63 rows then error — not quite the "64 rows" the test asserts. minor: error path emits 63 rows, not 64; document or make error path emit a synthetic final row.
- [pass 9] mod.rs — list shows "0-15 compute, 16 call, 17 look", matching spec. clean.

## notes (informational)

- P_MINUS_2 = 0xFFFFFFFEFFFFFFFF verified: Goldilocks p = 2^64 - 2^32 + 1, p-2 = 0xFFFFFFFEFFFFFFFF. binary: 32 ones, one zero (bit 32), 32 ones. Correct.
- inv emits 64 rows (1 init + 63 steps): matches spec/trace.md "64 consecutive rows".
- branch correctly evaluates only the chosen sub-formula (matches spec note "only the chosen branch is evaluated").
- shl uses canonical guard `if vn >= 32 { 0 }` — avoids Rust UB on u64 shifts ≥ 64. Correct.
- not uses `(!v) & 0xFFFF_FFFF` — correct 32-bit semantics matching spec width W=32.
- eq returns Goldilocks::ZERO for equal and ONE for unequal — matches spec "0 = true".
- lt compares as_u64() of canonical Goldilocks representative — this IS the canonical ordering per spec/patterns/10-lt.md ("0 if v_a < v_b").

## summary

no blockers found. five major issues, all centered on trace-register layout drift from spec/trace.md: branch (pattern 4), eq (pattern 9), axis (pattern 0), and inv (pattern 8) write registers in positions that do not match the spec's per-pattern register map. The interpreted RESULTS are correct in every case; the trace WITNESS layout is what diverges. This matters because the constraint system in zheng expects the spec layout. Fix priority: align spec/trace.md or align code; pick one source of truth per pattern and update the other in the same commit (sync rules from CLAUDE.md).
