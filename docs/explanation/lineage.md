# lineage

from combinatory logic to nox — a century of searching for the smallest universal instruction set that is simultaneously a proof system.

## the search

the history of computation is a search for the minimum. what is the fewest rules that still express all computation? each answer trades generality for structure, approaching a limit where the instruction set is simultaneously a programming language, a proof system, and a data format.

```
combinatory logic (1924)   S, K combinators            pure abstraction
  → lambda calculus (1936) Church's untyped lambda      computable functions
  → Nock (2016)            natural numbers + decrement  deterministic VM for Urbit
  → nox (2026)             field elements + inverse     proof-native VM for cyber
```

but the search for the minimum is only half the story. nox also inherits from a parallel lineage — [[natural computing]], [[convergent computation]], [[focus flow computation]] — where computation is convergence to equilibrium, not derivation from axioms. nox is where these two lineages meet: the minimalist instruction set tradition and the convergent computation paradigm.

```
Turing machine (1936)        sequential symbol manipulation
  → von Neumann (1945)       stored program architecture
  → RISC (1980s)             reduced instruction set
  → Nock (2016)              minimal instruction set

natural computing            convergence, not derivation
  → convergent computation   formal foundation (computation = equilibrium)
  → focus flow computation   executable model (attention flows)
  → nox (2026)               field-native execution with conserved focus
```

## combinatory logic (1924)

Moses Schonfinkel discovered that two combinators — S and K — suffice to express any computable function. no variables needed. Haskell Curry developed this into a complete computational framework in the 1930s. the insight: computation is manipulation of structure, and two rules are enough to manipulate any structure.

S and K are universal but impractical. a simple function like "add two numbers" expands into thousands of combinator applications. the encoding is complete but hostile — no human or machine can efficiently reason about computation in raw S,K form. the search continues: can we find a system that is both minimal and usable?

## lambda calculus (1936)

Alonzo Church answered with variables and substitution. the lambda calculus has one operation — function application — and one mechanism — variable binding. it is the theoretical foundation of every programming language. Church proved it equivalent to Turing machines in computational power.

the lambda calculus is the right abstraction for mathematicians. it maps naturally to mathematical functions, type theory, and proof theory. but it carries complexity: variable capture, alpha-equivalence, substitution strategies. these are non-trivial to implement correctly, and they resist deterministic serialization. two syntactically different lambda terms can be alpha-equivalent — the same computation in different clothes. this matters when you need canonical representation.

## Nock (2016)

Curtis Yarvin ([[Urbit]]) took a radical step: replace variables with addresses into a binary tree. a Nock program is a noun — either an atom (natural number) or a cell (ordered pair of nouns). the environment is a noun. the code is a noun. the result is a noun. one data structure, no variables, no binding, no substitution.

Nock has twelve rules. the arithmetic primitive is decrement — given n, produce n-1. this is sufficient for Turing completeness (decrement plus conditional plus recursion), but it is maximally hostile to algebraic reasoning. every arithmetic operation decomposes into iterated decrement. multiplication of two 64-bit numbers takes O(2^64) steps. the system is deterministic and minimal, but the cost model is pathological.

the deeper insight of Nock is structural: computation as tree transformation. a program navigates a tree (axis), constructs trees (cons), and recursively applies transformations (compose). this is the core that nox inherits. the arithmetic, nox replaces entirely.

## nox (2026)

nox makes one fundamental mutation: replace natural numbers with [[Goldilocks field]] elements and decrement with field inverse.

```
Nock:  atom = natural number,   primitive = decrement
nox:   atom = F_p element,      primitive = field inverse
```

this changes everything. decrement over natural numbers is O(1) but leads to O(n) arithmetic. field inverse over [[Goldilocks field|Goldilocks]] is O(64) multiplications (Fermat's little theorem) but leads to O(1) arithmetic for add, sub, mul — and O(1) [[stark]] constraint verification. the tradeoff: pay 64× for inversion, gain constant-time everything else.

the consequence is proof-nativity. the [[Goldilocks field]] is the native field of the [[stark]] proof system. a nox execution trace — the sequence of field element operations — is directly the algebraic constraint system that the prover proves and the verifier checks. there is no compilation step from "program" to "circuit." the program IS the circuit. the execution IS the witness.

## the properties that emerge

the sixteen-pattern structure produces four properties that no previous system in the lineage achieves simultaneously:

### confluence

the patterns form an orthogonal rewrite system — each has a unique tag, no two overlap, no variable appears twice in a pattern's left-hand side. by Huet-Levy (1980), orthogonal systems are confluent: any two reduction sequences from the same term reach the same result. there is no "wrong" evaluation order.

this is the mathematical property that makes everything else possible. parallelism is free — two threads reducing different subexpressions cannot produce race conditions because there is nothing to race toward. content-addressed memoization is sound — `(H(subject), H(formula))` uniquely determines `H(result)`. the [[cybergraph]] is a deterministic function of its inputs. agreement between nodes is not negotiated — it is computed.

S,K combinators are confluent. the lambda calculus is confluent (Church-Rosser theorem). Nock is confluent. but none of them are confluent over a field — and that is what makes nox proofs native.

### cost determinism

the cost of a computation depends only on its syntactic structure, never on runtime values, cache state, or execution environment. if two nodes compute the same function on the same input, they spend the same [[focus]]. this is unique in the lineage — even Nock's cost model depends on the magnitude of natural numbers (decrement of a large number costs proportionally more).

cost determinism means: the network can price computation before executing it. a [[neuron]] can estimate the focus cost of a formula by static analysis. the [[stark]] prover can predict the trace size. there are no cost surprises.

### field-first arithmetic

every value is a field element. [[cryptography]] is a native instruction. a field multiplication is a single CPU operation. hashing is ~2800 field ops expressible in pure patterns. [[stark]] proofs verify computations using the same field arithmetic that performs them. there is no impedance mismatch between computation and verification.

no previous system in the lineage has this property. S,K operates on untyped terms. lambda calculus operates on abstract functions. Nock operates on natural numbers. nox operates on elements of a field that is simultaneously the computation substrate and the proof substrate.

### hash-universal identity

identity equals hash. two values are the same if and only if they hash to the same digest. this makes content-addressing intrinsic rather than bolted on. every [[particle]] in the [[cybergraph]] is identified by the hash of its content. every edge is authenticated by the hashes of its endpoints. deduplication is automatic. references are unforgeable.

combined with confluence, this produces content-addressed computation: `(H(subject), H(formula)) → H(result)` is a permanent, universal, verifiable fact. the planetary computation cache falls out as a direct consequence.

## the convergent computation connection

the Turing paradigm defines computation as derivation from axioms. [[convergent computation]] defines it as convergence to equilibrium. every Turing computation can be expressed as convergence — but convergent systems compute things formal derivation cannot reach, because they operate outside the proof-theoretic domain where [[Goedel]]'s theorems apply.

nox is the machine for [[focus flow computation]] — the executable model of convergent computation. the [[focus]] parameter in `reduce(subject, formula, focus)` is the conserved quantity from FFC. attention flows through the [[cybergraph]], and nox is the engine that transforms each unit of attention into verified computation.

```
Natural Computing              — the paradigm
  └─ Convergent Computation    — the formal foundation
       └─ Focus Flow Comp.     — the computational model
            └─ nox             — the executable machine
                 └─ Cybergraph — the knowledge substrate
```

## what was kept, what was changed

from Nock, nox inherits:
- nouns as the universal data structure (binary trees, no variables)
- axis addressing (navigating trees by numeric path)
- homoiconicity (code is data is code)
- deterministic evaluation (same input → same output, always)
- the structural patterns (axis, quote, compose, cons, branch)

from Nock, nox replaces:
- natural numbers → [[aurum]] elements
- decrement → field inverse
- crash semantics → typed error propagation (⊥_error, ⊥_unavailable)
- implicit cost → explicit focus metering

nox adds what Nock could not have:
- field arithmetic as native patterns (add, sub, mul, inv, eq, lt)
- bitwise operations (xor, and, not, shl) for binary protocol handling
- cryptographic hash as a pattern: [[Hemera]]
- non-deterministic witness injection (hint) for zero-knowledge proofs
- jets optimized for recursive [[stark]] verification and more

the result is sixteen patterns instead of twelve, but the six additional patterns (field arithmetic) are the entire reason the system can produce proofs natively. four more (bitwise) handle the binary world. one (hash) closes the identity loop. the increase in pattern count is the price of proof-nativity — and the return is that every computation in the network is automatically verifiable.

## the terminus

this is the terminus of the search. nox is simultaneously:
- a programming language (programs are nouns, evaluated by sixteen patterns)
- an algebraic constraint system (the execution trace IS the AIR)
- a content-addressable computation substrate (confluent reduction → canonical hash)
- a convergent computation engine (focus is the conserved quantity)

each previous step traded generality for structure. nox reaches the fixed point where structure and proof coincide. the instruction set is simultaneously the programming language, the proof system, and the identity scheme. there is nothing left to simplify that would not destroy one of these three roles.

## the name

Nox (Latin: "night"). the dark twin of computation — where execution and proof are indistinguishable, where privacy and verification coexist, where the program disappears into its proof. from Nock to nox: the same letter shift as from natural numbers to field elements, from counting to algebra, from light to the productive darkness where proofs are born.
