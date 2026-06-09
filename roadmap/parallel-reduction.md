---
status: draft
date: 2026-05-15
author: parallel-reduction research
---

# Parallel reduction in nox — confluent budget threading

## Abstract

Nox is a confluent rewrite system. That's what distinguishes it from Nock,
where the reference interpreter is rigorously sequential and the budget
thread `f → f1 → f2` is baked into the specification. We want nox to be
parallel by default — confluence demands it, the prover needs it, the
hardware affords it. The blocker is that "budget" is a linear resource:
two evaluators of the same `(object, formula, budget)` triple must agree on
the **exact** `Result` variant **and** the **exact** remaining budget,
otherwise consensus breaks.

This proposal:

1. Formalizes the tension between confluence and sequential budget threading.
2. Shows why every "obvious" parallelization breaks in the case of
   sub-expressions with asymmetric cost under a tight budget.
3. Surveys four resolution schemes and proves which preserve
   sequential-equivalence.
4. Proposes a single semantics — **cost-bounded partitioning with
   refund-at-join** — and a two-mode implementation (Layer 1 strict,
   Layer 2 permissive).
5. Identifies the open theorems and the implementation work.

This is a specification-level proposal. The current sequential interpreter
remains correct; this proposal layers a parallel-safe semantics on top
with an equivalence theorem.

## 1. The core tension

### 1.1 Confluence (have)

Nox's Layer 1 patterns form an orthogonal term rewriting system: each
pattern tag uniquely determines a rewrite rule, left-hand sides are
linear, no two patterns overlap. By Huet-Lévy (1980), orthogonal TRSs
are confluent — independent of evaluation strategy or termination.

Pattern 16 (call) intentionally breaks confluence at the level of
*witness choice* (multiple witnesses may satisfy the same check). But
once a witness is fixed, the rewrite system remains orthogonal.

Pattern 17 (look) is deterministic — confluence is preserved.

Layer 3 (jets) preserves confluence by observational equivalence to
Layer 1 expansions.

So: **the rewrite system is confluent. Two evaluators on the same input
WILL reach the same normal form, in finite time, if budget permits.**

### 1.2 Budget (need)

Budget is a linear resource. Each reduction step decrements it. The
sequential semantics threads it through:

```
reduce(o, [op [a b]], f):
  c = cost(op)
  if f < c then Halt(f)
  (va, f1) = reduce(o, a, f - c)        -- f → f1
  (vb, f2) = reduce(o, b, f1)           -- f1 → f2
  return (op(va, vb), f2)
```

For consensus, two evaluators must agree on:
- The `Result` variant (`Ok` / `Halt` / `Error(kind)`)
- The result `Noun` (when `Ok`)
- The remaining budget (when `Ok` or `Halt`)
- The error kind (when `Error`)

The Noun agreement is given by confluence. The budget agreement requires
that **every evaluator threads f → f1 → f2 → ... in the same way**.
Sequential threading enforces this trivially. Parallel threading does not.

### 1.3 The conflict

Confluence says: it doesn't matter whether you reduce `a` before `b` or
in parallel — the result is the same. But sequential budget threading
DOES depend on order: `b` runs with `f - c - cost(a)`, not `f - c`. If
`b` is run in parallel with `f - c` directly, it gets MORE budget than
sequential gives it.

When budgets are loose (much greater than total cost), this doesn't
matter. When budgets are tight (close to total cost), parallel and
sequential can disagree on whether the program halted, and on the
remaining budget value.

## 2. Why naive parallelism breaks — three counter-examples

Let `c = cost(op) = 1` throughout; we focus on the budget threading for
sub-formulas. Let `cost(formula)` denote actual incurred cost (sequential).

### 2.1 Asymmetric branches, loose split

```
f = 102
formula = [op [a b]]  with cost(a) = 100, cost(b) = 1
sequential: a runs with 101 → succeeds, f1 = 1. b runs with 1 → succeeds, f2 = 0.
            Result: Ok(val, 0).
equal-split: a gets 50, halts (101 > 50, but cost(a)=100 > 50 already aborts).
              Result: Halt(50). DIVERGES from sequential.
```

### 2.2 Tight budget, race to halt

```
f = 50, c = 1, so f - c = 49
cost(a) = 30, cost(b) = 26
sequential: a runs with 49, succeeds, returns 19. b runs with 19, halts
            because some inner step needs 25 > 19. Result: Halt(x_b_seq).
parallel-full-budget: a runs with 49, succeeds (used 30, returns 19).
                       b runs with 49 (parallel sees full budget!), succeeds
                       (used 26, returns 23). Join: total work = 1+30+26=57 > 50.
                       Result: Halt(?). VALUE OF HALT DIFFERS from sequential.
```

This is the deep problem. The Halt variant agrees (both halt), but the
remaining-budget payload may differ.

### 2.3 Halt cascade with order dependence

```
f = 20, formula = [op [a b]]
cost(a) = 21 (a halts immediately, returns Halt(20))
cost(b) = 5 (b would succeed if it ran)
sequential: a halts at f-c=19. Halt(?) propagates. b NEVER RUNS.
            Result: Halt(value-of-a-halt).
parallel: a halts, b succeeds. At join: a halted → propagate. Halt(value-of-a-halt).
          Result matches if join takes a's halt deterministically. ✓
```

Case 2.3 is salvageable with a deterministic "first branch's halt wins"
join rule. Cases 2.1 and 2.2 require deeper changes.

## 3. The space of solutions

Five conceptually distinct schemes. Each fixes a different aspect of the
tension.

### Scheme S1 — Full-budget speculative, sum at join

Each branch runs with the parent's `f - c` budget as soft cap. At join,
sum the actually-incurred costs; halt if sum > `f - c`.

```
parallel_reduce(o, [op [a b]], f):
  if f < c then Halt(f)
  let (va, used_a) = parallel_reduce(o, a, f - c) ∥ (vb, used_b) = parallel_reduce(o, b, f - c)
  let total = c + used_a + used_b
  if total > f: Halt(f mod total)   -- or some defined halt-value
  else: Ok((va, vb), f - total)
```

**Soundness gap**: case 2.2 (tight budget). Sequential gives `b` budget
`f1 = f - c - used_a`. Parallel gives `b` budget `f - c`. If `b` contains
a step costing > `f1` but ≤ `f - c`, parallel succeeds where sequential
halts. They disagree.

**Fix attempt**: make `b`'s execution sensitive to the eventual join.
Impossible without sequential dependency.

**Verdict**: ❌ Not sequential-equivalent in general.

### Scheme S2 — Static equal partition

Pre-partition: each branch gets `⌊(f-c)/2⌋`. Run independently. Join with
sum of remaining.

**Soundness gap**: case 2.1. Asymmetric costs.

**Verdict**: ❌ Not sequential-equivalent.

### Scheme S3 — Reformulate "sequential" to match partitioning

Redefine sequential semantics so that it ALSO partitions the budget.
Then parallel and "sequential" agree by construction (they ARE the same
semantics).

```
reduce(o, [op [a b]], f):
  if f < c then Halt(f)
  let fa = (f - c) / 2
  let fb = (f - c) - fa
  let (va, ra) = reduce(o, a, fa)
  let (vb, rb) = reduce(o, b, fb)
  ... combine ...
```

This is a **new language**. Programs that worked under classical sequential
threading may halt under the new semantics if costs are uneven.

**Verdict**: ✅ Self-consistent. ❌ Different language from Nock-style
sequential. Lose ability to "borrow" budget from one branch to another.

### Scheme S4 — Cost-bound annotated formulas

Every formula `t` has a statically computable upper bound `bound(t) ≥
actual_cost(t)`. Partition by bound, refund unused at join.

```
parallel_reduce(o, [op [a b]], f):
  if f < c then Halt(f)
  if bound([op [a b]]) > f then Halt(f)         -- eager bound check
  let fa = bound(a), fb = bound(b)
  let (va, ra) = parallel_reduce(o, a, fa) ∥ (vb, rb) = parallel_reduce(o, b, fb)
  let used = (fa - ra) + (fb - rb) + c
  return (op(va, vb), f - used)
```

This works if `bound()` is exact for every formula. For nox patterns
0, 1, 3, 5-15, 17 the bound is structural:
- `bound(atom) = 0`
- `bound([0 _]) = 1` (axis)
- `bound([1 _]) = 1` (quote)
- `bound([3 [a b]]) = 1 + bound(a) + bound(b)`
- `bound([5..7,9..14 [a b]]) = COST_op + bound(a) + bound(b)`
- `bound([8 a]) = 64 + bound(a)`
- `bound([15 a]) = 25 + bound(a)`
- `bound([17 [n k]]) = 1 + bound(n) + bound(k)`

For pattern 4 (branch), bound is the **max** over arms (only one is
chosen): `bound([4 [t [y n]]]) = 1 + bound(t) + max(bound(y), bound(n))`.

For pattern 2 (compose) and pattern 16 (call), the bound is **dynamic**:
- `[2 [x y]]` reduces `x` and `y`, then `reduce(rx, ry)`. The third reduce
  depends on `ry`'s runtime value. No static bound.
- `[16 [tag check]]` injects a prover-supplied witness, whose cost is
  unknown to the formula author.

**Verdict**: ✅ Sequential-equivalent for Layer 1 patterns. ⚠️ Compose
and call need dynamic cost handling.

### Scheme S5 — Cost is part of the witness

Extend Scheme S4 by requiring compose results and call witnesses to
**declare** their cost. The reducer verifies the declaration.

```
For compose [2 [x y]]:
  rx = reduce(o, x), ry = reduce(o, y)
  ry must be of form [bound formula_body] — cost declaration in first slot
  reduce(rx, formula_body) with budget bound

For call [16 [tag check]]:
  witness = provider(tag, o)
  witness must be of form [witness_bound payload]
  -- cost is declared by the prover; verifier checks during proof gen
```

**Verdict**: ✅ Sequential-equivalent everywhere. ❌ Imposes new
formula structure conventions (potential incompatibility, schema burden).

## 4. The proposal — S4 with explicit dynamic-cost frontier

We adopt **Scheme S4** as the parallel semantics for Layer 1 patterns,
and explicitly mark pattern 2 (compose) and pattern 16 (call) as having
a sequential continuation phase.

### 4.1 Definition: `bound(t) : Formula → ℕ`

```
bound(atom)             = 0
bound([0 a])            = 1
bound([1 c])            = 1
bound([2 [x y]])        = 1 + bound(x) + bound(y) + DYNAMIC
bound([3 [a b]])        = 1 + bound(a) + bound(b)
bound([4 [t [y n]]])    = 1 + bound(t) + max(bound(y), bound(n))
bound([5 [a b]])        = 1 + bound(a) + bound(b)     -- add
bound([6 [a b]])        = 1 + bound(a) + bound(b)     -- sub
bound([7 [a b]])        = 1 + bound(a) + bound(b)     -- mul
bound([8 a])            = 64 + bound(a)                -- inv
bound([9 [a b]])        = 1 + bound(a) + bound(b)     -- eq
bound([10 [a b]])       = 64 + bound(a) + bound(b)    -- lt
bound([11 [a b]])       = 32 + bound(a) + bound(b)    -- xor
bound([12 [a b]])       = 32 + bound(a) + bound(b)    -- and
bound([13 a])           = 32 + bound(a)                -- not
bound([14 [a n]])       = 32 + bound(a) + bound(n)    -- shl
bound([15 a])           = 25 + bound(a)                -- hash
bound([16 [tag check]]) = 1 + bound(tag) + DYNAMIC
bound([17 [n k]])       = 1 + bound(n) + bound(k)
```

`DYNAMIC` is a sentinel meaning "not statically bounded". The frontier is
crossed at compose and call.

### 4.2 Parallel reduction rule (binary structural patterns)

For `op ∈ {3, 5, 6, 7, 9, 10, 11, 12, 14, 17}` (binary, both arms always
evaluated, no dynamic continuation):

```
parallel_reduce(o, [op [a b]], f):
  c = cost(op)
  if f < c then Halt(f)
  if bound(a) + bound(b) + c > f then
    -- not enough budget. fall back to sequential to determine the precise halt point.
    sequential_reduce(o, [op [a b]], f)
  else
    let (va, ra) = parallel_reduce(o, a, bound(a)) ∥ (vb, rb) = parallel_reduce(o, b, bound(b))
    let used = (bound(a) - ra) + (bound(b) - rb) + c
    Ok(op_apply(va, vb), f - used)
```

The fallback to sequential when `bound > budget` preserves the sequential
halt semantics for tight budgets. When `bound ≤ budget`, parallel is
sound: each branch has enough budget to complete (since `actual_cost ≤
bound`), so no branch can halt internally; the result is identical to
sequential by confluence + arithmetic of refund.

### 4.3 Unary parallel patterns (8, 13, 15)

```
parallel_reduce(o, [op a], f):
  c = cost(op)
  if f < c then Halt(f)
  if bound(a) + c > f then sequential_reduce(o, [op a], f)
  else
    let (va, ra) = parallel_reduce(o, a, bound(a))
    Ok(op_apply(va), f - ((bound(a) - ra) + c))
```

Single sub-formula; no parallelism within this rule, but `a` itself may
contain parallel sub-formulas.

### 4.4 Branch (4) — lazy with arm-bound

```
parallel_reduce(o, [4 [t [y n]]], f):
  c = 1
  bound_total = 1 + bound(t) + max(bound(y), bound(n))
  if f < c then Halt(f)
  if bound_total > f then sequential_reduce(o, [4 [t [y n]]], f)
  else
    let (vt, rt) = parallel_reduce(o, t, bound(t))
    let chosen = if vt == 0 then y else n
    let (vc, rc) = parallel_reduce(o, chosen, bound(chosen))
    Ok(vc, f - ((bound(t) - rt) + (bound(chosen) - rc) + 1))
```

The two reductions are sequential (must know `vt` to pick `chosen`).
Slack from the unchosen arm is refunded: `max(bound(y), bound(n)) -
bound(chosen) ≥ 0` extra budget returns to the caller.

### 4.5 Compose (2) — partial parallelism

```
parallel_reduce(o, [2 [x y]], f):
  if f < 1 then Halt(f)
  if bound(x) + bound(y) + 1 > f then sequential_reduce(o, [2 [x y]], f)
  else
    let (rx, rrx) = parallel_reduce(o, x, bound(x)) ∥ (ry, rry) = parallel_reduce(o, y, bound(y))
    let f_after_init = f - 1 - (bound(x) - rrx) - (bound(y) - rry)
    -- third reduce is sequential, its cost is dynamic
    sequential_reduce(rx, ry, f_after_init)
```

The first two sub-reductions parallelize via static bounds. The third
runs sequentially with whatever budget remains.

### 4.6 Call (16) — sequential after witness arrival

```
parallel_reduce(o, [16 [tag check]], f):
  if f < 1 then Halt(f)
  let (vtag, rt) = parallel_reduce(o, tag, bound(tag))  -- bounded, parallel-internal
  let witness = provider(vtag, o)
  if witness is None then Halt(remaining_budget)
  let witness_object = cell(witness, o)
  let f_after_witness = f - 1 - (bound(tag) - rt)
  sequential_reduce(witness_object, check, f_after_witness)
```

Tag evaluation parallelizes internally; everything after the provider
hand-off is sequential because the witness is dynamic.

### 4.7 Atomic patterns (0, 1, 17)

`bound = 1 + bound(args)`. No internal parallelism (axis is a tree walk;
quote returns a literal; look does one provider call). Sequential
implementation, no semantic difference.

## 5. The sequential-equivalence theorem

**Theorem (parallel-sequential equivalence)**. For every formula `t`,
object `o`, and budget `f`:

```
parallel_reduce(o, t, f).result_kind == sequential_reduce(o, t, f).result_kind
parallel_reduce(o, t, f).data         == sequential_reduce(o, t, f).data         when result_kind = Ok
parallel_reduce(o, t, f).remaining    == sequential_reduce(o, t, f).remaining    when result_kind ∈ {Ok, Halt}
parallel_reduce(o, t, f).error_kind   == sequential_reduce(o, t, f).error_kind   when result_kind = Error
```

**Proof sketch** (by structural induction on `t`):

*Base cases* (atoms, quote): cost is trivial, no sub-formulas.
sequential and parallel are identical.

*Inductive cases (Layer 1 binary patterns)*:
1. If `bound(t) > f`: both fall back to sequential. Trivially equal.
2. If `bound(t) ≤ f`: by definition `actual_cost(a) ≤ bound(a)` for both
   sub-formulas. Each sub-reduction in parallel has enough budget
   (`bound(a) ≥ actual_cost(a)`), so neither halts internally — both
   return `Ok`. The combined result is `op_apply(va, vb)`. Sequential
   with f ≥ bound(t) ≥ actual_cost(t) also returns `Ok`. By confluence,
   `va`, `vb`, and hence `op_apply` are identical.
3. Remaining-budget computation:
   - Sequential: `f → f - c → f - c - actual_cost(a) → f - c -
     actual_cost(a) - actual_cost(b)`.
   - Parallel: `f - c - (bound(a) - ra) - (bound(b) - rb)` where
     `ra = bound(a) - actual_cost(a)` and similarly for `rb`. So
     `bound(a) - ra = actual_cost(a)`. Same value.

*Branch (4)*: by IH on chosen arm. The unchosen arm is never reduced in
either semantics. Bound provides upper estimate for the chosen arm; only
chosen arm's `actual_cost` is consumed.

*Compose (2), call (16)*: parallel parts are bounded and use the binary
case; the dynamic continuation is invoked sequentially. By IH the
parallel parts produce identical results; the continuation runs in
identical sequential semantics; equivalence follows.

∎

The theorem requires `bound()` to be a true upper bound on actual cost.
Conservatively, this is satisfied by the formula above, since pattern 4
takes max-of-arms (loose) and all other patterns compute exact structural
sums (tight).

## 6. Implementation strategy

### 6.1 Two API surfaces

```rust
// existing — never changes
pub fn reduce<const N: usize, T: Tracer>(
    order: &mut Order<N>, ...
) -> Outcome;

// new — parallel-safe, identical Result
pub fn reduce_parallel<const N: usize, T: Tracer + Send>(
    order: &mut Order<N>, ...  // see 6.2 for the sharing problem
) -> Outcome;
```

`reduce_parallel` is observationally identical to `reduce`. Callers can
swap freely. The library guarantees Result-equivalence by the theorem
above.

### 6.2 The Order sharing problem

`Order<N>` is mutated during reduction (atoms/cells allocated for
results). Parallel threads would race on it. Three options:

**(a) Per-thread scratch + merge.** Each thread has its own scratch
`Order`. At join, allocations from each scratch are merged into a single
canonical `Order` via the hash-cons table (identical data bytes → same
slot). Merge is O(scratch size × hash cost).

**(b) Atomic Order.** Order's allocator becomes a lock-free hash table.
Read paths are uncontended; write paths take a CAS. Hash-consing already
provides the unique-allocation invariant; we just need to make
`alloc_raw` atomic.

**(c) Pre-allocate.** Before parallel phase, pre-compute the set of
allocations each branch will need. Allocate in parent; pass OrderIds into
branches. Requires static analysis of allocation patterns — likely
intractable for non-trivial formulas.

**Recommendation**: (b) for the rust impl; (a) for distributed evaluation
where threads can't share memory.

### 6.3 Trace row ordering

The parallel evaluator produces rows in non-deterministic order. For
witness-hash determinism, rows must be emitted in a canonical order.

Solution: each row carries a **structural index** — its position in the
reduce tree (e.g., a path string `"2.0.1"` meaning "left child of left
child of pattern 2 from root"). At end of parallel evaluation, the
collected rows are sorted by structural index before tracer emission.

This is one additional pass over the trace; ~O(N) work where N is row
count. Acceptable cost for parallelism.

### 6.4 Bound computation cost

`bound(t)` is itself a tree walk over the formula. For a one-shot
parallel reduction, this is O(formula size) — comparable to the
reduction itself. For repeated reductions of the same formula, the
bound can be cached at the OrderId level via the Order's hash-cons
machinery (one more entry per data: `cached_bound: u64`).

The cache invariant: `bound(data)` is a pure function of data structure,
so hash-consed identical sub-data share the same bound. Cache hit rate
should be high for any recursive program.

## 7. Open theorems and validation

### 7.1 Theorems to formalize

**T1 (sequential-equivalence)**: stated in §5. Should be machine-checked
(Coq/Lean) for confidence given the consensus implications.

**T2 (bound monotonicity)**: `bound(t)` is monotone under sub-data
substitution. Used in the proof of T1's compose case.

**T3 (parallel commutativity)**: for binary patterns with both arms
parallelizable, swapping the parallel execution order produces identical
witness traces (modulo the canonical sort). Used to argue
implementation-independence.

### 7.2 Counter-examples to construct

For each rejected scheme (S1, S2, S3), the spec MUST carry one concrete
counter-example showing where it diverges from sequential. This is the
"do not reinvent" record for future readers.

### 7.3 Test plan

- Round-trip tests: for 1000 random `(o, t, f)` triples,
  `reduce(o, t, f) == reduce_parallel(o, t, f)`.
- Stress tests: deeply nested compose chains (where dynamic cost
  dominates); tight-budget scenarios at the bound boundary.
- Adversarial: formulas designed to exhibit the worst-case bound vs
  actual-cost gap (bound = 10^6 but actual = 10).

## 8. Migration path

### 8.1 Phase 0 (this proposal accepted)

- Spec: add §parallel reduction to `reduction.md`, replacing the current
  "open spec gap" note at line 129.
- Code: no changes. Sequential remains canonical.

### 8.2 Phase 1 (bound function)

- Add `bound: Formula → ℕ` to `data/order.rs` as a cached field on
  `NounEntry`.
- Compute bound at data construction. Pattern 2 and pattern 16 sub-data
  bounds carry a "DYNAMIC" sentinel.
- Specs: `data/order.md` documents the bound field.

### 8.3 Phase 2 (parallel-safe Order)

- Refactor `Order::alloc_raw` to atomic-cas allocation.
- Tests: stress allocator with N threads doing concurrent atom/cell
  allocation. Verify hash-cons invariants hold.

### 8.4 Phase 3 (parallel reduce)

- Add `reduce_parallel()` API.
- Implement structural-index-sorted trace.
- Round-trip tests against `reduce()`.
- Document the equivalence theorem in `rs/lib.rs` doc comment.

### 8.5 Phase 4 (formal verification)

- Port the theorem to Lean. Machine-check T1 sequential-equivalence.
- Cross-check with zheng's witness generation: same witness hash on
  serial and parallel runs.

## 9. Implications beyond nox

### 9.1 zheng witness generation

zheng generates one constraint system per pattern row. Order-of-rows
doesn't affect the constraint system if rows are sorted canonically. So
zheng can consume parallel traces without modification, **once row
ordering is canonical**. Without canonical ordering, zheng would produce
different proofs for the same program — which is unacceptable.

### 9.2 Distributed evaluation

The parallel semantics extends naturally to distributed: a coordinator
splits sub-formulas across worker nodes, each worker reduces locally,
results are combined at the coordinator. Bound-based partitioning gives
the coordinator a static work estimate for scheduling.

### 9.3 The Nock comparison

Nock evaluators are sequential. Parallel Nock is folklore — every
implementation has its own ad-hoc rules, none are consensus-safe. Nox
formalizes parallel reduction at the spec level. This is the substantive
difference from Nock: not just "the patterns are different" but "the
semantics is parallel by construction".

### 9.4 Related work

- **Lévy parallel reduction** (1980, lambda calculus): optimal reduction
  via labeled redexes. Establishes that sharing of identical sub-terms
  enables parallelism without cost duplication. Nox's hash-consing
  provides exactly this sharing.
- **Lamping's algorithm** (1990, interaction nets): a concrete impl of
  Lévy's optimal reduction. Out of scope for nox (more complex than
  needed) but technically applicable.
- **Linear logic** (Girard, 1987): budget as a linear resource maps to
  TENSOR (parallel partition) and WITH (shared, additive). Our scheme S4
  is essentially TENSOR for binary patterns and WITH for branch.
- **Resource semantics in Coq** (FunLog, Iris): formal frameworks for
  proving resource-bounded programs. Could host the T1 machine-check.
- **Cairo, RISC-Zero, Plonky2**: all use fixed-cost-per-step sequential
  execution. None aspire to confluent parallelism. Nox occupies a unique
  position.

## 10. Decision request

This proposal asks for sign-off on:

1. **Adopting Scheme S4** (cost-bound partitioning with refund-at-join,
   sequential fallback for over-budget) as the parallel semantics.
2. **Pattern 2 and pattern 16 carry an explicit DYNAMIC marker** in their
   bound. They are not parallelizable past their first phase.
3. **Phasing**: spec change (§reduction.md) lands now; rust impl follows
   in 3 phases (bound caching, atomic Order, parallel reduce).
4. **Formal verification track**: T1 is goal-stated; machine-checked
   proof is non-blocking for v1 release but blocks v2.

Open questions to discuss:

- Should `bound` be carried in `Order::NounEntry` (always-on cache) or
  computed on-demand? Cache costs ~16 bytes per data (one u64 + status
  bit), gives O(1) bound lookup at the price of permanent memory.
- For the `Halt` payload in the over-budget fallback case: do we want
  the sequential halt's precise remaining-budget value (requires running
  sequentially in fallback) or a simpler `Halt(0)` (loses precision but
  parallelizable)?
- Cross-regime considerations: does the bound function need to be
  algebra-aware (different costs per regime per `vm.md` table)? If so,
  bound becomes `bound(t, regime)` and the cache becomes regime-keyed.

## Appendix A: rejected lemmas

For the historical record, here are sequence-equivalence claims that
LOOK plausible but FAIL:

**False claim 1**: "Sum of sub-formula costs equals total cost". False
when sub-formulas share state via compose (third reduce sees both).

**False claim 2**: "Confluence implies budget order-independence".
False; confluence is about normal-form uniqueness, not about resource
consumption path.

**False claim 3**: "If `f ≥ actual_cost(t)`, sequential and any
parallel strategy produce identical `Result`". True only if parallel
gives each branch ≥ actual_cost of that branch. False if parallel uses
fixed partition smaller than actual cost.

**False claim 4**: "Pattern 4 (branch) bound is `bound(t) + bound(y) +
bound(n)`". False because only one arm is reduced; sum overestimates.
Correct bound uses max-of-arms.

## Appendix B: bound function as Rust

```rust
pub fn bound<const N: usize>(order: &Order<N>, root: OrderId) -> Cost {
    match order.get(root).map(|e| e.inner) {
        Some(Noun::Atom { .. }) => Cost::Exact(0),
        Some(Noun::Cell { left, right }) => {
            let tag = match order.atom_value(left) {
                Some((v, _)) => v.as_u64(),
                None => return Cost::Exact(0),  // malformed; cost charged at reduce time
            };
            bound_for_tag(order, tag, right)
        }
        None => Cost::Exact(0),
    }
}

fn bound_for_tag<const N: usize>(order: &Order<N>, tag: u64, body: OrderId) -> Cost {
    let base = COSTS.get(tag as usize).copied().unwrap_or(1);
    match tag {
        0 | 1 => Cost::Exact(base),                                        // axis / quote
        2 | 16 => Cost::Dynamic(base),                                     // compose, call
        4 => {                                                              // branch — max of arms
            let (t, rest) = order.cell_pair(body)?;
            let (y, n) = order.cell_pair(rest)?;
            let bt = bound(order, t);
            let by = bound(order, y);
            let bn = bound(order, n);
            Cost::combine_max_arms(base, bt, by, bn)
        }
        3 | 5..=7 | 9..=14 | 17 => {                                      // binary structural
            let (a, b) = order.cell_pair(body)?;
            Cost::sum(base, bound(order, a), bound(order, b))
        }
        8 | 13 | 15 => Cost::sum(base, bound(order, body), Cost::Exact(0)), // unary
        _ => Cost::Exact(base),                                            // unknown → minimal
    }
}

enum Cost { Exact(u64), Dynamic(u64) }
```

## Appendix C: example traces under both semantics

For program `[5 [[1 3] [1 5]]]` with budget 100 (cost 3):

Sequential:
```
row 0: tag=5, budget_in=100, budget_out=97
row 1: tag=1, budget_in=99, budget_out=98 (left quote)
row 2: tag=1, budget_in=98, budget_out=97 (right quote)
```

Parallel (with canonical sort by structural index `()`, `(1.0)`, `(1.1)`):
```
row 0: tag=5,  budget_in=100, budget_out=97, struct_idx=()
row 1: tag=1,  budget_in=99,  budget_out=98, struct_idx=(1.0)
row 2: tag=1,  budget_in=98,  budget_out=97, struct_idx=(1.1)
```

Identical after sort. The budget_in/budget_out values reflect the
**partitioned-then-refunded** view: each branch's budget_in is its
`bound(branch)`; budget_out is `bound(branch) - actual_cost(branch)`.
For atomic quotes, both are 1 → 0, so the visible row contents may
differ from sequential's threading view. But the parent's final
remaining (97) and the result OrderId are identical.

This is the subtle point: **observable result identical; per-row budget
values differ on intermediate rows**. The witness hash includes per-row
budget, so the witness IS different. For consensus we need EITHER:

- (option α) only the parent budget matters; sub-row budgets are
  ignored in the witness hash; OR
- (option β) the witness format is the parallel-canonical one; all
  evaluators (including sequential interpreters) produce the parallel
  budget values to match.

This is the dual-mode budget bookkeeping decision. **Recommend option β**:
all reductions, sequential or parallel, follow the partitioned-budget
model. Sequential becomes a single-threaded execution of the same
semantics. Then witnesses are always identical regardless of execution
strategy.

This means **adopting the parallel semantics as THE semantics**, and
sequential becomes just a particular implementation choice. The
"sequential-equivalence theorem" then degrades to a statement about
implementations: any implementation that follows the spec produces the
same witness, regardless of whether it ran one thread or many.

This is the cleanest design and what we recommend.
