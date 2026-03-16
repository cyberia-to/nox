# hint

the boundary of knowledge — one instruction that provides privacy, search, and oracle access.

## one instruction

```
reduce(s, [16 constraint], f) =
  let (check, f1) = reduce(s, constraint, f - 1)
  let w = PROVER_INJECT()
  assert check(w) = 0
  (w, f1)
```

hint is one pattern among seventeen. but it is the entire mechanism through which private knowledge enters the system. every zero-knowledge proof, every private transaction, every oracle query, every search optimization — all flow through this single instruction.

the prover knows something the verifier does not. the prover injects that knowledge via hint. Layer 1 constraints verify it. the verifier checks the [[stark]] proof — which confirms the constraints were satisfied — without ever learning what was injected.

## the information asymmetry

Layer 1 is symmetric: both prover and verifier execute the same patterns, see the same values, compute the same results. hint breaks this symmetry. the prover has access to `PROVER_INJECT()` — an oracle that produces a value from outside the VM. the verifier never calls this oracle. the verifier sees only the [[stark]] proof that the constraint `check(w) = 0` was satisfied.

this asymmetry is the foundation of zero-knowledge computation. what the prover knows but the verifier does not:

```
identity:
  prover injects: the secret key behind a neuron address
  Layer 1 checks: Hemera(secret) = address
  verifier learns: the address is valid, nothing else
  what stays hidden: the secret key

private transfer:
  prover injects: record details (owner, value, nonce)
  Layer 1 checks: conservation law, ownership, nullifier freshness
  verifier learns: the transfer is valid
  what stays hidden: who sent, who received, how much

AI inference:
  prover injects: neural network weights
  Layer 1 checks: forward pass with those weights produces the claimed output
  verifier learns: the inference is correct
  what stays hidden: the model weights (intellectual property)

optimization:
  prover injects: an optimal solution
  Layer 1 checks: solution satisfies constraints AND is optimal
  verifier learns: the result is correct
  what stays hidden: the search strategy that found it
```

## the constraint discipline

hint is not "inject anything." the constraint function `check` is a Layer 1 computation — fully deterministic, fully provable. the injected witness `w` must satisfy `check(w) = 0`. if it does not, the [[stark]] proof is invalid and the verifier rejects it.

the prover has freedom in choosing WHICH valid witness to inject (this is the non-determinism). the prover has no freedom in injecting an INVALID witness (this is the soundness). the constraint is the discipline.

this means hint is only as powerful as the constraint you can write. proving identity requires a constraint that checks `Hemera(secret) = address`. proving a valid transfer requires constraints that check conservation laws and ownership. proving a correct inference requires constraints that check the forward pass. the art of zero-knowledge programming is the art of writing Layer 1 constraints that verify the property you care about without revealing the data you want to hide.

## hint and trident

[[trident]] (the high-level language that compiles to nox) exposes hint via the `divine()` function. a trident program calls `divine()` to request a witness from the prover, then uses regular code (which compiles to Layer 1 patterns) to verify the witness.

```
// trident pseudocode
fn prove_identity(address: Hash) -> bool {
    let secret = divine();                    // hint: prover injects secret
    let computed = hemera(secret);            // Layer 1: hash the secret
    assert_eq(computed, address);             // Layer 1: check hash matches
    true
}
```

the `divine()` call compiles to pattern 16. the `hemera()` call compiles to pattern 15. the `assert_eq` compiles to patterns 9 and 4. the entire proof logic is a mix of Layer 1 and Layer 2, compiled from a language that looks like ordinary code.

## hint and quantum computation

in the quantum compilation model, hint maps to a quantum oracle query. [[Grover's algorithm]] turns O(N) witness search into O(√N) oracle queries. the same instruction that provides classical zero-knowledge privacy also provides quantum search speedup.

the connection is structural: hint asks "give me a value that satisfies this constraint." in the classical case, the prover does the search (any strategy, outside the VM). in the quantum case, the search is done by the quantum computer via amplitude amplification. the constraint function (Layer 1) is the same in both cases — only the mechanism for finding the witness changes.

this means nox programs with hint are automatically quantum-ready: the constraint logic is written once, and the witness-finding strategy can be classical or quantum depending on the available hardware.

## hint and memoization

hint breaks confluence — different provers may inject different valid witnesses for the same constraint. this means hint-containing computations are NOT memoizable. the global computation cache excludes them:

```
Layer 1 computation: cache key = (H(object), H(formula)) — permanent, universal
Layer 2 computation: NOT cached — result depends on prover's private knowledge
```

pure sub-expressions within a hint-containing computation remain memoizable. the exclusion applies to the tainted root (the computation that transitively contains a hint), not to its pure children. a hint-containing proof that internally evaluates `add(7, 13)` can still use the cached result for that pure sub-expression.

## the minimum necessary non-determinism

nox could have been entirely deterministic — sixteen patterns, no hint. it would be a complete, provable, content-addressable virtual machine. but it would have no privacy. every computation would be publicly reproducible. every intermediate value visible. every secret exposed.

hint is the minimum necessary non-determinism. one instruction. one oracle call. one point where private knowledge enters and is verified without being revealed. everything else in nox is deterministic — the entire rest of the VM is a machine for checking the claims that enter through this single gate.

the elegance is in the economy: one instruction for all of zero-knowledge computation.
