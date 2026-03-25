# reduction specification

version: 0.2
status: canonical

## overview

reduction is the execution model of nox. a formula is applied to an object under a focus budget, producing a result. the reduction rules are algebra-independent — they work identically across all nox<F, W, H> instantiations. pattern dispatch costs and constraint counts are per-instantiation (see patterns.md).

## interface

the top-level invocation is `ask` — the seven fields of a [[cyberlink]]:

```
ask : (ν, Object, Formula, τ, a, v, t) → Answer

  ν        : Neuron  — who orders the computation
  Object   : Noun    — the environment, the data, the context
  Formula  : Noun    — the code (cell of form [tag body])
  τ        : Token   — denomination of payment
  a        : Amount  — how much to pay (focus budget)
  v        : Valence — prediction about result quality {-1, 0, +1}
  t        : Time    — block height
```

`ask` checks the [[cybergraph]] memo cache before executing. if `axon(Formula, Object)` has a verified result → return it. otherwise → `reduce`, prove, link.

## reduction signature

the internal execution engine:

```
reduce : (Object, Formula, Focus) → Result

  Object   : Noun    — the environment, the data, the context
  Formula  : Noun    — the code (cell of form [tag body])
  Focus    : F       — resource budget (element of the instantiated field),
                       decremented per pattern
                       comparison (f < cost) uses integer ordering on canonical representatives
                       the Halt guard prevents subtraction from ever wrapping

Result = (Noun, Focus')     — success with remaining focus
       | Halt               — focus exhausted (f < cost of next pattern)
       | ⊥_error            — type/semantic error (bitwise on hash, inv(0), axis on atom)
       | ⊥_unavailable      — referenced content not retrievable (network partition)
```

in the canonical instantiation (nox<Goldilocks>), Focus is an F_p element with comparison on [0, p).

## focus metering

every reduce() call costs 1 focus, deducted before the pattern executes. if remaining focus is less than 1 (or less than the multi-step cost for axis/inv/hash), reduction halts.

```
reduce(s, formula, f) =
  if f < cost then Halt          — cost is 1 for most patterns
  let (tag, body) = formula
  ... dispatch by tag, deducting cost from f ...
```

focus is the same resource that weights cyberlinks in the cybergraph. a neuron spends focus to think (run nox programs) and to speak (create cyberlinks). the budget is unified — attention and computation are the same currency.

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
    16 → hint(s, body, f)
    _  → ⊥_error                   — unknown pattern tag
```

if formula is an atom (not a cell), reduction produces ⊥_error.

## confluence

Layer 1 patterns form an orthogonal rewrite system:
- each pattern has a unique tag (non-overlapping left-hand sides)
- left-hand sides are linear (no variable appears twice)
- patterns are non-overlapping (tag uniquely determines the rule)

by the Huet-Levy theorem (1980), orthogonal term rewriting systems are confluent without requiring termination.

confluence holds for the term rewriting system (the pure reduction rules). with finite focus, the full reduce() function is confluent only when focus is sufficient for all reduction paths to reach a normal form. with insufficient focus, different evaluation strategies may halt at different points — one path may succeed where another exhausts focus. the result noun, when produced, is always the same; whether it is produced depends on evaluation strategy and available focus.

consequence: for any (object, formula) pair with sufficient focus, the result depends only on what the program IS, never on how it was evaluated. parallel reduction, lazy reduction, eager reduction, any mixture — the answer is the same.

consequence: content-addressed memoization is sound. `(H(object), H(formula))` uniquely determines `H(result)` for successful completions. the memo table caches only successful results (status = 0).

Layer 2 (`hint`) breaks confluence intentionally — multiple valid witnesses may satisfy the same constraints. soundness is preserved: any witness that passes the Layer 1 constraint check is valid. hint is the deliberate injection point for non-determinism.

Layer 3 (jets) preserves confluence — jets are observationally equivalent to their Layer 1 expansions. replacing a jet with its pure equivalent produces identical results.

## parallel reduction

confluence enables safe parallelism. specific patterns have independent sub-computations:

```
Pattern 2 (compose):  [2 [x y]]
  reduce(s,x) ∥ reduce(s,y)  — INDEPENDENT
  Then: reduce(result_x, result_y)

Pattern 3 (cons):     [3 [a b]]
  reduce(s,a) ∥ reduce(s,b)  — INDEPENDENT
  Then: cell(result_a, result_b)

Patterns 5-7, 9-12:   [op [a b]]
  reduce(s,a) ∥ reduce(s,b)  — INDEPENDENT
  Then: apply op

Pattern 4 (branch):   [4 [t [c d]]]
  reduce(s,t) first — MUST evaluate test before choosing
  Then: ONE of reduce(s,c) or reduce(s,d)  — NOT parallel (lazy)
```

all binary arithmetic and bitwise patterns can evaluate both operands in parallel. branch is the only pattern that enforces sequential evaluation (test before choice).

NOTE on focus and parallelism: the formal reduction rules thread focus sequentially (f → f1 → f2), which contradicts parallel evaluation of sub-expressions. for parallelism to work, the focus budget must be partitioned between parallel branches (e.g. split f equally, or pre-compute sub-expression costs). the partitioning scheme is not yet specified. confluence guarantees the result is identical regardless of evaluation order, but the focus accounting must produce the same final value. this is an open specification gap.

## global memoization via cybergraph

the [[cybergraph]] is the memo table. the cache key is the axon — the directed edge from formula to object:

```
Key:   axon(formula, object) = H(formula, object)
Value: result particle linked to the axon
```

before executing, `ask` checks whether `axon(formula, object)` already has a verified result linked to it in the graph. if yes → zero computation, return the cached result. if no → reduce, prove, link `axon → result`.

```
ask(ν, object, formula, τ, a, v, t) → answer

  1. order_axon = H(formula, object)
  2. lookup axon in cybergraph
     → verified result exists: return cached (zero compute)
     → no result: reduce(object, formula, focus=(τ,a)), prove
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
- Layer 2: NOT memoizable (hint results are prover-specific)
- Layer 3: fully memoizable (jets are deterministic)

computations containing hint anywhere in their reduction tree are excluded from the global cache. pure sub-expressions within a hint-containing computation remain memoizable — the exclusion applies to the hint-tainted root, not to its pure children.

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
reduce(s, [5 [a b]], f) =
  let (v_a, f1) = reduce(s, a, f - 1)
  if v_a is error → return error
  let (v_b, f2) = reduce(s, b, f1)
  if v_b is error → return error
  ((v_a + v_b) mod p, f2)
```

Halt propagates identically — if a sub-expression exhausts focus, the parent halts.

## pattern 16: hint interface

hint is the non-deterministic witness injection point. the prover provides a value; Layer 1 constraints validate it. the verifier never calls the provider — it checks the zheng proof.

### provider signature

```
trait HintProvider {
    fn provide(&self, tag: F, subject: NounRef) -> HintResult;
}

enum HintResult {
    Value(NounRef),    // prover provides a witness noun
    Halt,              // prover has no witness — clean halt
}
```

### reduction rule

```
hint formula structure: cell(16, cell(tag_formula, check_formula))

reduce(s, [16 [tag_f check_f]], f):
  1. tag = reduce(s, tag_f, f - 1)           // evaluate tag expression
     if tag is error/halt → propagate
  2. witness = provider.provide(tag, s)       // ask prover
     if witness == Halt → return Halt(f')     // clean halt, focus preserved
  3. result = reduce([witness s], check_f, f')  // validate: check runs on [witness subject]
     if result is error → return Error(HintRejected)
  4. return result
```

### tag

tag is a field element identifying WHICH hint the prover should provide. conventions:

```
0x00  unspecified (prover decides)
0x01  private key / secret witness
0x02  optimization solution
0x03  search result / oracle query
0x04  decryption share
```

tags are conventions, not enforced by the VM. any field value is a valid tag. the prover interprets the tag to decide what witness to provide.

### check formula

the check formula validates the witness using Layer 1 patterns only. the witness enters as head of the subject: `[witness original_subject]`. the check can access both the witness (via axis 2) and the original subject (via axis 3).

if check succeeds → result is the check's output (the validated computation).
if check fails (error) → HintRejected. the witness was invalid.

### properties

- **synchronous**: hint is a function call, not an event
- **no hint = halt**: not an error. the prover simply doesn't know. focus preserved for caller
- **hint rejected = error**: the witness failed validation. prover bug
- **not memoizable**: different provers provide different valid witnesses. hint-tainted computation trees excluded from global cache. pure sub-expressions within remain memoizable
- **confluence broken intentionally**: multiple valid witnesses may satisfy the same check. this is correct — it is what makes zero-knowledge possible
- **verifier never calls provide()**: the zheng proof covers steps 1 and 3. the witness enters the trace as a value; constraints verify the check formula

### focus cost

hint dispatch: 1 focus (same as all patterns). tag evaluation: cost of tag_f. check evaluation: cost of check_f. total: 1 + cost(tag_f) + cost(check_f). if hint halts, only 1 + cost(tag_f) consumed.

## Result encoding

Result is not a noun. it is the return type of reduce(). in the content-addressed protocol:

```
success:     (status=0, H(result), focus_remaining)   — noun identity + focus
halt:        (status=1, focus_remaining)               — no result noun
error:       (status=2, error_kind)                    — no result noun
```

unavailable is an error (status=2) with error_kind=3. it is not a separate status code — the trace only encodes three status values (0, 1, 2). the Result type in the reduction semantics distinguishes ⊥_error from ⊥_unavailable for error reporting, but the trace encoding folds them into status=2 with the error_kind discriminant.

the trace encodes Result in r15 (status) and r12 (error kind). the instance includes status and H(result) for success cases (H(result) = 0 when status ≠ 0, see trace.md). errors are transient computation outcomes, not persistent data — they have no content-addressed storage entry.

## focus accounting

**rule: every reduce() call costs 1 focus.**

this is the entire cost model. when reduce(s, formula, f) is entered, 1 focus is deducted for dispatch (reading the tag, selecting the pattern). sub-expression reduce() calls deduct their own costs recursively. the total focus consumed by a computation is the total number of reduce() calls in its evaluation tree.

two patterns have multi-step overhead beyond the dispatch cost. the overhead is per-instantiation:

canonical (nox<Goldilocks, Z/2^32, Hemera>):
- axis: 1 (O(1) polynomial evaluation via PCS opening — replaces legacy depth traversal)
- inv: 64 (square-and-multiply chain — 64 sequential multiplications)
- hash: 300 (Poseidon2 permutation — 72 rounds + absorption/squeeze)

all other patterns cost exactly 1 per reduce() call.

```
example: reduce([1,2], [5 [[0 2] [0 3]]], 100)

reduce #1: dispatch pattern 5 (add), deduct 1 → f=99
reduce #2: reduce(s, [0 2], 99)
  dispatch pattern 0 (axis), deduct 1 → f=98
  axis(cell(1,2), 2) = 1
reduce #3: reduce(s, [0 3], 98)
  dispatch pattern 0 (axis), deduct 1 → f=97
  axis(cell(1,2), 3) = 2
apply: 1 + 2 = 3
result: (3, 97)
```

3 reduce() calls = 3 focus consumed. matches test vector.

```
example: reduce([1,2], [4 [[9 [[0 2] [0 3]]] [[1 100] [1 200]]]], 100)

reduce #1: dispatch pattern 4 (branch), deduct 1 → f=99
reduce #2: reduce(s, [9 [[0 2] [0 3]]], 99)
  dispatch pattern 9 (eq), deduct 1 → f=98
  reduce #3: reduce(s, [0 2], 98) → axis → 1, f=97
  reduce #4: reduce(s, [0 3], 97) → axis → 2, f=96
  eq(1, 2) = 1 (not equal)
branch: t=1 ≠ 0, take no-branch
reduce #5: reduce(s, [1 200], 96)
  dispatch pattern 1 (quote), deduct 1 → f=95
  result: 200
result: (200, 95)
```

5 reduce() calls = 5 focus consumed. matches test vector.

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

with polynomial nouns, hemera drops to ~3 calls per execution: (1) domain separation wrap for noun identity, (2) Fiat-Shamir seed for the proof, (3) Brakedown binding for the PCS commitment. the legacy model required hundreds of hemera calls for recursive tree hashing (one permutation per cell in the noun). polynomial commitment replaces recursive hashing with O(N) field operations + 1 hemera call per identity.

hemera hash operations (pattern 15) during execution also fold via the sponge construction: each absorption block folds into the accumulator (~30 field ops) instead of being proved independently. a 4 KiB particle hash: ~2,956 constraints folded (was ~54,464 with independent permutations, 18× savings).

### signal assembly

the output of a complete ask() with proof-carrying is a [[signal]]:

```
signal = {
  ν:    neuron_id                          from ask() argument
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
  - focus was sufficient and correctly metered
  - the result noun has the claimed identity H(result)

verification: one zheng decider call (10-50 μs), independent of computation size.
signal size: ~1-5 KiB (proof + impulse + 160 bytes ordering metadata)
proof cost: ZERO additional (accumulated during execution)
```

the signal is the unit of state change for [[BBG]]. it flows from device through [[structural-sync|structural sync]] (layers 1-5) to the network.

## stark integration

the reduction trace (sequence of pattern applications with register states) IS the stark witness. the trace layout is per-instantiation — column widths depend on F element size. see trace.md for the register layout and AIR constraints. see jets.md for optimized verification. see [[zheng]] recursion.md for HyperNova folding mechanics.
