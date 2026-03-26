# self-verification

the system that verifies itself — recursive proof composition to arbitrary depth, constant proof size at every level.

## the closure

the [[stark]] verifier for nox is itself a nox program. every operation the verifier needs — field arithmetic, hashing, polynomial evaluation, Merkle path checking, FRI folding — is native to the sixteen patterns or their jet equivalents.

this closure is the deepest property of the design. the VM can verify proofs about its own executions. a proof-of-proof is a nox program that runs the verifier on a proof. the proof-of-proof is itself provable. recursion to arbitrary depth, constant proof size at every level.

```
program → trace → stark proof → verifier (nox program) → trace → stark proof → ...
```

the system closes on itself.

## why this matters

in most blockchain architectures, verification is a separate, trusted layer. the consensus protocol runs on one system; transaction verification runs on another; proof verification runs on a third. each layer must trust the ones below it. the trust chain ends at hardware and compilers that cannot be audited within the system.

nox collapses this chain. the verifier is a nox program. the proof that the verifier ran correctly is a [[stark]] proof. that proof can be verified by the same verifier. each level of verification reduces to the same mathematical framework — transition constraints over the [[Goldilocks field]], checked by [[sumcheck]], committed by [[WHIR]].

the trust chain does not end at an unauditable layer. it ends at the mathematics of finite field arithmetic and the security assumptions of the hash function ([[Hemera]]). these are public, analyzable, and falsifiable.

## the recursive structure

level 0: a nox program runs. it produces an execution trace. the trace is proved by the [[stark]] prover. output: a proof P₀ of ~60-157 KiB.

level 1: the verifier (a nox program) takes P₀ as its object and the verification formula as its formula. it runs (~825 constraints with CCS jet). it produces an execution trace. that trace is proved. output: a proof P₁ of ~2 KiB.

level 2: the verifier takes P₁ as its object. it runs. it produces P₂ of ~60-157 KiB.

at every level, the proof is the same size. the original computation could have been 10 patterns or 10 billion patterns — P₀ is the same size. P₁ is the same size. the proof compresses computation: O(N) execution → O(1) verification.

## aggregation

recursive verification enables proof aggregation. a block producer collects N transaction proofs and produces one proof that "all N transactions are valid."

```
step 1: verify P_tx1 → valid
step 2: verify P_tx2 → valid
...
step N: verify P_txN → valid
aggregate: prove that steps 1..N all succeeded → P_block
```

P_block is one proof. it covers all N transactions. the on-chain verifier checks one proof per block, regardless of how many transactions the block contains. O(N) transactions → O(1) on-chain verification.

this is the scalability mechanism of [[cyber]]. the chain does not re-execute transactions. it does not verify each proof individually. it checks one aggregated proof per block. the cost of consensus verification is independent of the volume of computation it certifies.

## the verifier's cost

with Brakedown (Merkle-free PCS), the zheng verifier is pure field arithmetic. canonical costs from zheng/specs/verifier.md:

```
generic (no jets):           ~8,000 constraints
CCS jet + batch Brakedown:      ~825 constraints
+ algebraic Fiat-Shamir:          ~89 constraints
```

per-fold cost: ~30 field ops + 1 hemera hash. the cost scales linearly with recursion depth, and each level produces a constant-size proof (~2 KiB).

## the verifier as a program

the nox verifier is written in nox. this means it is:

- content-addressable: `H(verifier_object, verifier_formula)` is a fixed identifier
- memoizable: the result of verifying a specific proof is cacheable
- auditable: the verifier code is a noun, inspectable by anyone
- upgradeable: a new verifier version is a new noun, deployed as a new computation

the verifier is not embedded in the consensus protocol as trusted code. it is a program among programs, held to the same proof requirements as any other computation. if the verifier has a bug, the bug is provable — it will produce an incorrect trace that the (meta-)verifier can detect.

this is the meaning of "trustless verification." the system does not ask you to trust the verifier. it asks you to trust the mathematics of finite fields and hash functions. the verifier is just another program, and its correctness is just another provable fact.

## the fixed point of trust

the recursive structure creates a fixed point. the verifier verifies proofs. the verifier's own execution is provable. the proof of the verifier's execution is verifiable by the verifier. at no point does the chain of verification leave the nox framework.

```
computation → proof → verification → proof → verification → ...
     ↑                                              │
     └──────────────── same framework ──────────────┘
```

this is the mathematical analogue of a self-sustaining system. the system generates proofs, and the system verifies proofs, and the verification is itself a proof. there is no external authority, no trusted third party, no unauditable substrate. the trust is in the math, and the math is in the machine, and the machine is in the math.
