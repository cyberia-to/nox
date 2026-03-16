# confluence

why evaluation order is irrelevant — and why that single property enables a planetary computation cache.

## the theorem

Layer 1 patterns form an orthogonal rewrite system:

- each pattern has a unique tag (0-15) — no two patterns match the same formula shape
- left-hand sides are linear — no variable appears twice in a pattern
- patterns are non-overlapping — the tag uniquely determines which rule fires

by the Huet-Levy theorem (1980), orthogonal term rewriting systems are confluent without requiring termination.

confluence means: if a term can be reduced in two different ways, both reductions eventually reach the same result. the evaluation strategy — eager, lazy, parallel, random — does not matter. the answer is the same.

## why this is extraordinary

most programming languages are evaluation-order-dependent. in C, `f(g(), h())` evaluates g() and h() in an implementation-defined order — and if either has side effects, the result depends on the order. in Haskell, lazy evaluation can observe different behavior than strict evaluation when non-termination is involved. even in pure functional languages, the exact normal form can depend on the reduction strategy in the presence of non-terminating subexpressions.

nox sidesteps all of this. the sixteen patterns are pure tree transformations with no side effects (Layer 1 has no I/O, no mutable state, no exceptions that depend on evaluation order). the orthogonality of the rules guarantees that however you choose to reduce, you get the same answer. this is a theorem, not a convention. it holds for any nox program, on any machine, under any scheduler.

## consequences

### content-addressed computation

because the result is independent of evaluation strategy, the computation's identity can be defined purely by its inputs:

```
key:   (H(subject), H(formula))
value: H(result)
```

this pair of hashes — the hash of what the program knows and the hash of what the program does — uniquely determines the hash of the output. any machine, anywhere, at any time, that reduces the same subject with the same formula will produce the same result. the cache entry is universal, permanent, and verifiable.

this is the foundation of the planetary computation cache. see content-addressing.md for the full implications.

### safe parallelism

confluence guarantees that parallel reduction is safe. patterns 2 (compose), 3 (cons), and all binary arithmetic/bitwise patterns (5-14) have independent sub-computations:

```
[2 [x y]]:    reduce(s,x) ∥ reduce(s,y)   — both use the same subject
[3 [a b]]:    reduce(s,a) ∥ reduce(s,b)   — independent tree construction
[5 [a b]]:    reduce(s,a) ∥ reduce(s,b)   — independent operand evaluation
```

the parallel results are identical to sequential results. no locks, no synchronization, no race conditions. the guarantee is structural — it follows from the mathematics of the rewrite system, not from careful programming.

the exception is pattern 4 (branch): the test must be evaluated before choosing a branch. this is deliberate — lazy evaluation of branches prevents infinite recursion in the untaken path. this controlled sequentiality within an otherwise parallel system is the right design: only branch where you must, parallelize everywhere else.

### reproducibility

any node in the [[cyber]] network that independently computes the same program on the same data will produce the same result. this is stronger than "eventually consistent" — it is "always identical." two nodes that never communicate, on different hardware, running different implementations, using different evaluation strategies, will compute the same hash for the same inputs.

this makes verification trustless. a node publishes `(H(subject), H(formula)) → H(result)`. any other node can verify this claim by re-running the computation, or by checking the [[stark]] proof. the result is either correct or it is not. there is no ambiguity, no "it depends on the implementation."

### the cybergraph is a function

a [[cyberlink]] is a nox computation. the cyberlink's identity (its content hash) determines its output. confluence guarantees that two nodes independently evaluating the same cyberlink produce the same result. the [[cybergraph]] — the sum of all cyberlinks — is a deterministic function of its inputs, verified by anyone, reproducible everywhere.

this property is what makes the cybergraph a shared, trustless knowledge structure. it is not a database that nodes must synchronize — it is a function that nodes independently evaluate and agree on by mathematical necessity.

## Layer 2 and confluence

hint (pattern 16) breaks confluence intentionally. two different provers may inject different valid witnesses for the same constraint. `reduce(s, [16 c], f)` may produce different results depending on what the prover injects.

this is the point. privacy requires non-determinism. if the computation is fully deterministic, the verifier can reproduce it and learn everything the prover knows. non-determinism — the prover choosing which witness to inject — is what creates the information asymmetry that zero-knowledge proofs exploit.

soundness is preserved: any witness must satisfy the Layer 1 constraint check. an invalid witness fails the constraint and is rejected. the non-determinism is bounded — any valid witness produces a correct result, even if different provers choose different valid witnesses.

the memoization scope follows: Layer 1 computations are fully memoizable (deterministic). Layer 2 computations are NOT memoizable (prover-specific). pure sub-expressions within a hint-containing computation remain memoizable — the taint applies to the hint root, not to its pure children.

## the mathematical structure

for those interested in the rewrite theory: the sixteen patterns form a left-linear, non-overlapping term rewriting system (TRS) over the term algebra of nouns. the sort structure is simple: all terms have sort `Noun`.

orthogonality follows from:
1. each rule's left-hand side begins with a distinct constructor (the pattern tag 0-15)
2. the body variables appear at most once (linearity)
3. no critical pairs exist (the tag makes rules non-overlapping)

by the theorem of Huet and Levy (1980), extended by Klop (1980) and Toyama (1987), any orthogonal TRS is confluent — even if it is non-terminating. this means: even for programs that loop forever, any partial results obtained along the way are consistent. there is no state where two evaluators have computed conflicting partial results.

this is a stronger guarantee than most systems provide. it is the mathematical bedrock on which content-addressed computation, safe parallelism, and trustless verification all rest.
