# reduction specification

version: 0.2
status: canonical

## overview

reduction is the execution model of nox. a formula is applied to an object under a resource budget, producing a result. the reduction rules are algebra-independent — they work identically across all nox<F, W, H> instantiations. pattern dispatch costs and constraint counts are per-instantiation (see patterns.md).

## interface

the top-level invocation is `order` — the seven fields of a [[cyberlink]]:

```
order : (ν, Object, Formula, τ, a, v, t) → Result

  ν        : Neuron  — who orders the computation
  Object   : Noun    — the environment, the data, the context
  Formula  : Noun    — the code (cell of form [tag body])
  τ        : Token   — denomination of payment
  a        : Amount  — how much to pay (resource budget)
  v        : Valence — prediction about result quality {-1, 0, +1}
  t        : Time    — block height
```

`order` checks the [[cybergraph]] memo cache before executing. if `axon(Formula, Object)` has a verified result → return it. otherwise → `reduce`, prove, link.

## reduction signature

the internal execution engine:

```
reduce : (Object, Formula, Budget) → Result

  Object   : Noun    — the environment, the data, the context
  Formula  : Noun    — the code (cell of form [tag body])
  Budget   : F       — resource budget (element of the instantiated field),
                       decremented per pattern
                       comparison (f < cost) uses integer ordering on canonical representatives
                       the Halt guard prevents subtraction from ever wrapping

Result = (Noun, Budget')     — success with remaining budget
       | Halt               — budget exhausted (f < cost of next pattern)
       | ⊥_error            — type/semantic error (bitwise on hash, inv(0), axis on atom)
       | ⊥_unavailable      — referenced content not retrievable (network partition)
```

in the canonical instantiation (nox<Goldilocks>), Budget is an F_p element with comparison on [0, p).

## budget metering

every reduce() call costs 1, deducted before the pattern executes. if remaining budget is less than 1 (or less than the multi-step cost for axis/inv/hash), reduction halts.

```
reduce(o, formula, f) =
  if f < cost then Halt          — cost is 1 for most patterns
  let (tag, body) = formula
  ... dispatch by tag, deducting cost from f ...
```

the budget is a resource counter. the protocol decides what token denominates it (will, CYB, or another). the VM does not know — it only decrements.

## evaluation order

formulas are evaluated recursively. the tag determines which pattern fires. the body structure determines the operands.

```
dispatch(s, formula, f) =
  let (tag, body) = formula        — formula must be a cell, else ⊥_error
  match tag:
    0  → axis(s, body, f)
    1  → quote(body, f)
    2  → compose(s, body, f)
    3  → cons(s, body, f)
    4  → branch(s, body, f)
    5  → add(s, body, f)
    ...
    15 → hash(s, body, f)
    16 → call(s, body, f)
    17 → look(s, body, f)
    _  → ⊥_error                   — unknown pattern tag
```

if formula is an atom (not a cell), reduction produces ⊥_error.

## confluence

Layer 1 patterns form an orthogonal rewrite system:
- each pattern has a unique tag (non-overlapping left-hand sides)
- left-hand sides are linear (no variable appears twice)
- patterns are non-overlapping (tag uniquely determines the rule)

by the Huet-Levy theorem (1980), orthogonal term rewriting systems are confluent without requiring termination.

confluence holds for the term rewriting system (the pure reduction rules). with finite budget, the full reduce() function is confluent only when budget is sufficient for all reduction paths to reach a normal form. with insufficient budget, different evaluation strategies may halt at different points — one path may succeed where another exhausts budget. the result noun, when produced, is always the same; whether it is produced depends on evaluation strategy and available budget.

consequence: for any (object, formula) pair with sufficient budget, the result depends only on what the program IS, never on how it was evaluated. parallel reduction, lazy reduction, eager reduction, any mixture — the answer is the same.

consequence: content-addressed memoization is sound. `(H(object), H(formula))` uniquely determines `H(result)` for successful completions. the memo table caches only successful results (status = 0).

Layer 2 (`call`) breaks confluence intentionally — multiple valid witnesses may satisfy the same constraints. soundness is preserved: any witness that passes the Layer 1 constraint check is valid. call is the deliberate injection point for non-determinism. `look` (pattern 17) is deterministic — it reads from BBG authenticated state and preserves confluence.

Layer 3 (jets) preserves confluence — jets are observationally equivalent to their Layer 1 expansions. replacing a jet with its pure equivalent produces identical results.

## parallel reduction

confluence enables safe parallelism. specific patterns have independent sub-computations:

```
Pattern 2 (compose):  [2 [x y]]
  reduce(o,x) ∥ reduce(o,y)  — INDEPENDENT
  Then: reduce(result_x, result_y)

Pattern 3 (cons):     [3 [a b]]
  reduce(o,a) ∥ reduce(o,b)  — INDEPENDENT
  Then: cell(result_a, result_b)

Patterns 5-7, 9-12:   [op [a b]]
  reduce(o,a) ∥ reduce(o,b)  — INDEPENDENT
  Then: apply op

Pattern 4 (branch):   [4 [t [c d]]]
  reduce(o,t) first — MUST evaluate test before choosing
  Then: ONE of reduce(o,c) or reduce(o,d)  — NOT parallel (lazy)
```

all binary arithmetic and bitwise patterns can evaluate both operands in parallel. branch is the only pattern that enforces sequential evaluation (test before choice).

NOTE on budget and parallelism: the formal reduction rules thread budget sequentially (f → f1 → f2), which contradicts parallel evaluation of sub-expressions. for parallelism to work, the resource budget must be partitioned between parallel branches (e.g. split f equally, or pre-compute sub-expression costs). the partitioning scheme is not yet specified. confluence guarantees the result is identical regardless of evaluation order, but the budget accounting must produce the same final value. this is an open specification gap.

## evaluation scope

each pattern specifies which sub-expressions it evaluates, in what order, and whether evaluation is eager (always) or lazy (conditional).

```
tag  pattern   subs  strategy     parallel  reduction rule
───  ────────  ────  ───────────  ────────  ─────────────────────────────────────────
 0   axis       0    —            —         body is literal axis address, no reduce()
 1   quote      0    —            —         body returned literally, no reduce()
 2   compose    2    eager+seq    yes*      reduce(o,x), reduce(o,y), then reduce(rx, ry)
 3   cons       2    eager        yes       reduce(o,a), reduce(o,b), then cell(ra, rb)
 4   branch     1+1  lazy         no        reduce(o,test), then ONE of reduce(o,yes) or reduce(o,no)
 5   add        2    eager        yes       reduce(o,a), reduce(o,b), then a + b
 6   sub        2    eager        yes       reduce(o,a), reduce(o,b), then a - b
 7   mul        2    eager        yes       reduce(o,a), reduce(o,b), then a * b
 8   inv        1    eager        —         reduce(o,a), then a⁻¹
 9   eq         2    eager        yes       reduce(o,a), reduce(o,b), then a = b
10   lt         2    eager        yes       reduce(o,a), reduce(o,b), then a < b
11   xor        2    eager        yes       reduce(o,a), reduce(o,b), then a ⊕ b
12   and        2    eager        yes       reduce(o,a), reduce(o,b), then a ∧ b
13   not        1    eager        —         reduce(o,a), then ¬a
14   shl        2    eager        yes       reduce(o,a), reduce(o,n), then a << n
15   hash       1    eager        —         reduce(o,a), then H(a)
16   call       1+1  eager+lazy   no        reduce(o,tag_f), provider injects witness, reduce([w o],check_f)
17   look       1    eager        —         reduce(o,key_f), then bbg.read(key)
```

column definitions:
- **subs**: number of sub-expressions evaluated via recursive reduce() calls. "1+1" means two sub-expressions evaluated at different times (not both unconditionally).
- **strategy**: eager = all sub-expressions always evaluated. lazy = some sub-expressions evaluated conditionally. "eager+seq" = all evaluated eagerly but the third step depends on the first two results.
- **parallel**: whether sub-expression evaluations can run concurrently (confluence guarantees identical results regardless of order).

### no sub-expression evaluation (tags 0-1)

**axis** (0): body `a` is the axis address, interpreted as an integer literal. axis navigates the already-evaluated object `o` — it does not call reduce() on its body. cost is 1 (the dispatch).

**quote** (1): body `c` is returned literally. the only pattern that touches neither the object nor any sub-formula via reduce(). cost is 1 (the dispatch).

### unary eager evaluation (tags 8, 13, 15, 17)

one sub-expression evaluated unconditionally, then the primitive operation applied to the result.

```
reduce(o, [8 a], f)  = let (v, f1) = reduce(o, a, f-1);  (v⁻¹, f1)
reduce(o, [13 a], f) = let (v, f1) = reduce(o, a, f-1);  (¬v, f1)
reduce(o, [15 a], f) = let (v, f1) = reduce(o, a, f-200); (H(v), f1)
reduce(o, [17 k], f) = let (v, f1) = reduce(o, k, f-1);  bbg.read(v)
```

no parallelism question arises — single sub-expression.

### binary eager evaluation (tags 5-7, 9-12, 14)

two sub-expressions evaluated unconditionally, then the binary operation applied. both sub-expressions receive the same object `o` and do not depend on each other's results — they can be evaluated in parallel.

```
reduce(o, [op [a b]], f) =
  let (v_a, f1) = reduce(o, a, f - 1)
  let (v_b, f2) = reduce(o, b, f1)
  (op(v_a, v_b), f2)
```

the sequential budget threading (f -> f1 -> f2) is a formalism. since reduce(o,a) and reduce(o,b) are independent computations on the same object, an implementation may evaluate them in parallel with budget partitioning.

### structural evaluation (tags 2-4)

**cons** (3): two sub-expressions evaluated eagerly. both receive the same object `o`, both are independent, both can run in parallel. results assembled into a cell.

**compose** (2): two sub-expressions evaluated eagerly, but the pattern has a third step that creates a sequential dependency. reduce(o,x) produces a new object, reduce(o,y) produces a formula, then reduce(rx, ry) applies the formula to the new object. the first two evaluations are independent and parallelizable. the third evaluation depends on both results.

**branch** (4): lazy. evaluates the test sub-expression first. based on the test result (0 or non-zero), evaluates exactly ONE of the two arm sub-expressions. the unchosen arm is never evaluated. this is the only pattern with conditional evaluation — it prevents infinite recursion (a recursive branch terminates as long as the base case arm is eventually chosen).

### Layer 2 evaluation (tags 16-17)

**call** (16): evaluates tag_f eagerly to obtain the tag. the prover injects a witness (external to the VM). then evaluates check_f with the witness prepended to the object: `reduce([witness o], check_f, f')`. the check evaluation is unconditional once the witness arrives, but the witness injection is an external step between the two reduce() calls. not parallelizable — tag must be known before the provider is called, and the witness must exist before check_f is evaluated.

**look** (17): evaluates key_f eagerly to obtain the key, then reads from BBG. single sub-expression, no parallelism question.

## global memoization via cybergraph

the [[cybergraph]] is the memo table. the cache key is the axon — the directed edge from formula to object:

```
Key:   axon(formula, object) = H(formula, object)
Value: result particle linked to the axon
```

before executing, `order` checks whether `axon(formula, object)` already has a verified result linked to it in the graph. if yes → zero computation, return the cached result. if no → reduce, prove, link `axon → result`.

```
order(ν, object, formula, τ, a, v, t) → result

  1. order_axon = H(formula, object)
  2. lookup axon in cybergraph
     → verified result exists: return cached (zero compute)
     → no result: reduce(object, formula, budget=(τ,a)), prove
  3. link order_axon → result (with stark proof)
  4. return result
```

two [[cyberlinks]] per computation:
- order: neuron links formula → object (with payment τ,a and valence v)
- answer: device links order_axon → result (with stark proof)

the order axon is a [[particle]] (axiom A6). multiple devices can answer the same order — competing results. the [[coupling|ICBS]] market determines which answer the graph trusts.

properties:
- universal: any node in the network can contribute and consume
- permanent: results never change (confluence guarantees determinism)
- verifiable: result hash is checkable against the stark proof
- the more the graph grows, the fewer computations actually execute

layer scope:
- Layer 1: fully memoizable (deterministic)
- Layer 2 (call): NOT memoizable (call results are prover-specific)
- Layer 2 (look): fully memoizable (deterministic BBG read)
- Layer 3: fully memoizable (jets are deterministic)

computations containing call anywhere in their reduction tree are excluded from the global cache. pure sub-expressions within a call-containing computation remain memoizable — the exclusion applies to the call-tainted root, not to its pure children. computations containing only look (no call) remain fully memoizable.

## error specification

errors are not nouns. they are Result variants — they exist in the reduction return type, not in the noun store. an error has no identity (no hash) and no content-addressed storage entry.

```
error kinds:
  0: type_error      — wrong atom type for operation (bitwise on hash, arithmetic on hash)
  1: axis_error      — axis on atom with index > 1
  2: inv_zero        — inv(0)
  3: unavailable     — referenced content not in store (network partition, missing noun)
  4: malformed       — formula is atom (not cell), or body has wrong structure
```

## error propagation

errors propagate upward through the reduction tree. if any sub-expression produces ⊥_error or ⊥_unavailable, the parent expression produces the same error.

```
reduce(o, [5 [a b]], f) =
  let (v_a, f1) = reduce(o, a, f - 1)
  if v_a is error → return error
  let (v_b, f2) = reduce(o, b, f1)
  if v_b is error → return error
  ((v_a + v_b) mod p, f2)
```

Halt propagates identically — if a sub-expression exhausts budget, the parent halts.

## call

call (pattern 16) is the non-deterministic witness injection point. see patterns/16-call.md for the full specification, provider interface, tag conventions, and properties.

## look

look (pattern 17) is the deterministic BBG read. see patterns/17-look.md for the full specification, key space, and properties.

## Result encoding

Result is not a noun. it is the return type of reduce(). in the content-addressed protocol:

```
success:     (status=0, H(result), budget_remaining)   — noun identity + remaining budget
halt:        (status=1, budget_remaining)               — no result noun
error:       (status=2, error_kind)                    — no result noun
```

unavailable is an error (status=2) with error_kind=3. it is not a separate status code — the trace only encodes three status values (0, 1, 2). the Result type in the reduction semantics distinguishes ⊥_error from ⊥_unavailable for error reporting, but the trace encoding folds them into status=2 with the error_kind discriminant.

the trace encodes Result in r15 (status) and r12 (error kind). the instance includes status and H(result) for success cases (H(result) = 0 when status ≠ 0, see trace.md). errors are transient computation outcomes, not persistent data — they have no content-addressed storage entry.

## budget accounting

**rule: every reduce() call costs 1.**

this is the entire cost model. when reduce(o, formula, f) is entered, 1 is deducted for dispatch (reading the tag, selecting the pattern). sub-expression reduce() calls deduct their own costs recursively. the total budget consumed by a computation is the total number of reduce() calls in its evaluation tree.

two patterns have multi-step overhead beyond the dispatch cost. the overhead is per-instantiation:

canonical (nox<Goldilocks, Z/2^32, Hemera>):
- axis: 1 (O(1) polynomial evaluation via Lens opening — replaces legacy depth traversal)
- inv: 64 (square-and-multiply chain — 64 sequential multiplications)
- hash: 300 (Poseidon2 permutation — 72 rounds + absorption/squeeze)

all other patterns cost exactly 1 per reduce() call.

```
example: reduce([1,2], [5 [[0 2] [0 3]]], 100)

reduce #1: dispatch pattern 5 (add), deduct 1 → f=99
reduce #2: reduce(o, [0 2], 99)
  dispatch pattern 0 (axis), deduct 1 → f=98
  axis(cell(1,2), 2) = 1
reduce #3: reduce(o, [0 3], 98)
  dispatch pattern 0 (axis), deduct 1 → f=97
  axis(cell(1,2), 3) = 2
apply: 1 + 2 = 3
result: (3, 97)
```

3 reduce() calls = 3 budget consumed. matches test vector.

```
example: reduce([1,2], [4 [[9 [[0 2] [0 3]]] [[1 100] [1 200]]]], 100)

reduce #1: dispatch pattern 4 (branch), deduct 1 → f=99
reduce #2: reduce(o, [9 [[0 2] [0 3]]], 99)
  dispatch pattern 9 (eq), deduct 1 → f=98
  reduce #3: reduce(o, [0 2], 98) → axis → 1, f=97
  reduce #4: reduce(o, [0 3], 97) → axis → 2, f=96
  eq(1, 2) = 1 (not equal)
branch: t=1 ≠ 0, take no-branch
reduce #5: reduce(o, [1 200], 96)
  dispatch pattern 1 (quote), deduct 1 → f=95
  result: 200
result: (200, 95)
```

5 reduce() calls = 5 budget consumed. matches test vector.

## proof-carrying reduction

reduction and proving are one operation. each reduce() call generates a trace row AND folds it into a running [[HyperNova]] accumulator:

```
reduce_with_proof(s, formula, f, acc) =
  if f < cost then Halt
  let (tag, body) = formula
  let (result, f', trace_row) = dispatch(s, tag, body, f)
  let acc' = fold_row(acc, trace_row)         ← ~30 field ops
  (result, f', acc')
```

at computation end, the accumulator IS the proof. run one decider to produce the final verifiable [[zheng]] proof. no separate proving phase — proving overhead is ~30 field operations per reduce() call, folded into execution.

with polynomial nouns, hemera drops to ~3 calls per execution: (1) domain separation wrap for noun identity, (2) Fiat-Shamir seed for the proof, (3) Brakedown binding for the Lens commitment. the legacy model required hundreds of hemera calls for recursive tree hashing (one permutation per cell in the noun). polynomial commitment replaces recursive hashing with O(N) field operations + 1 hemera call per identity.

hemera hash operations (pattern 15) during execution also fold via the sponge construction: each absorption block folds into the accumulator (~30 field ops) instead of being proved independently. a 4 KiB particle hash: ~2,956 constraints folded (was ~54,464 with independent permutations, 18× savings).

### signal assembly

the output of a complete order() with proof-carrying is a [[signal]]:

```
signal = {
  ν:    neuron_id                          from order() argument
  l⃗:    [cyberlink]                        the batch (from computation results)
  π_Δ:  [(particle, F_p)]                  impulse (focus shift)
  σ:    accumulator                        the proof (from proof-carrying reduction)
  prev: H(previous signal)                ordering (hash chain)
  mc:   H(causal DAG root)                ordering (Merkle clock)
  vdf:  VDF(prev, T_min)                  ordering (physical time)
  step: u64                               ordering (logical clock)
}

σ is the proof-carrying accumulator. it proves:
  - all reduce() calls were valid (correct pattern dispatch)
  - all hemera hashes were computed correctly (folded sponge)
  - budget was sufficient and correctly metered
  - the result noun has the claimed identity H(result)

verification: one zheng decider call (10-50 μs), independent of computation size.
signal size: ~1-5 KiB (proof + impulse + 160 bytes ordering metadata)
proof cost: ZERO additional (accumulated during execution)
```

the signal is the unit of state change for [[BBG]]. it flows from device through [[structural-sync|structural sync]] (layers 1-5) to the network.

## stark integration

the reduction trace (sequence of pattern applications with register states) IS the stark witness. the trace layout is per-instantiation — column widths depend on F element size. see trace.md for the register layout and AIR constraints. see jets.md for optimized verification. see [[zheng]] recursion.md for HyperNova folding mechanics.
