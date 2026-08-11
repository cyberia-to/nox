# nox

proof-native virtual machine for [[cyber]]. every execution produces a [[zheng]] proof as a byproduct — running a program and proving it ran correctly are the same act. there is no separate arithmetization step. the execution trace IS the algebraic constraint system.

## lineage

```
combinatory logic (1924)   S, K combinators            pure abstraction
  → lambda calculus (1936) Church's untyped lambda      computable functions
  → Nock (2016)            natural numbers + decrement  deterministic virtual machine for Urbit
  → nox (2026)             field elements + inverse     proof-native virtual machine for cyber
```

nox replaces Nock's natural numbers with [[Goldilocks field]] elements and decrement with field inverse. this single substitution makes the virtual machine algebraically native — every operation maps directly to field constraints with zero translation overhead.

## why nox is amazing

computation IS linking. `ask(ν, object, formula, τ, a, v, t)` has seven arguments — the seven fields of a [[cyberlink]]. ordering a computation and asserting knowledge are the same act. the [[cybergraph]] is simultaneously a knowledge base and a universal memo cache. every computation anyone ever did is reusable by everyone. the more the graph grows, the fewer computations actually execute. nox does not just compute — it remembers.

proof-native execution. most virtual machines bolt proofs onto execution after the fact — run the program, then arithmetize the trace, then prove. nox skips the middle step. the execution trace is already a valid [[zheng]] witness. `reduce(object, formula)` simultaneously computes the result and generates the proof artifact.

algebra polymorphism. the same 16 reduction patterns work over any field, any word width, any hash function. pattern semantics are universal — algebra is a parameter. a single nox program runs over Goldilocks ([[zheng]]), F₂ (Binius), or F_{p³} (recursive composition) by selecting a different instantiation. one spec, many proof systems.

merkle by construction. every `cons(a, b)` builds a Merkle tree — the hash is computed and stored at the parent node. `axis` traversal produces Merkle proofs as a side effect. content addressing is not a feature layered on top — it is the data model itself. and because every noun is content-addressed, every reduction result has a unique identity — the foundation of global memoization.

cybergraph-native. nox is tightly coupled with the [[cybergraph]] and [[bbg]]. the memo cache IS the graph state. the proof IS a [[cyberlink]]. the focus budget IS the token payment. this coupling is not a dependency — it is the source of compounding returns. every computation enriches the graph. every enrichment accelerates future computation.

minimal irreducible design. 18 patterns total: 16 deterministic compute (Layer 1) + call (16, non-deterministic witness injection) + look (17, deterministic BBG read) + 5 optimization jets. remove the jets — identical results, ~8.5× slower. remove call and look — no privacy, no state access, but still Turing-complete. remove Layer 1 — nothing remains. every pattern earns its place.

privacy at the boundary. the call pattern (16) injects untrusted witness data that Layer 1 constraints verify. this is where privacy enters: the prover knows the secret, the verifier checks the math. no trusted setup. no MPC ceremony. the architecture separates knowledge from verification.

lazy evaluation. the branch pattern (4) evaluates only the taken path. the other branch is never touched. this prevents infinite-recursion DoS attacks structurally — a property of the reduction semantics, not a runtime check.

unified IR. nox is simultaneously the intermediate representation (all [[cyber]] languages compile through it), the node runtime (production blockchain binary), and the composition tier (orchestrating programs across execution contexts). one representation from source to proof.

## architecture

```
ask(ν, object, formula, τ, a, v, t) → answer
```

the seven arguments of `ask` are the seven fields of a [[cyberlink]]. computation IS linking. the function:

1. compute `order_axon = H(formula, object)`
2. lookup: does `axon(formula, object)` have a verified result in the [[cybergraph]]?
   → yes: return cached result (zero computation — memoized)
   → no: `reduce(object, formula, focus=(τ,a))`, prove via [[zheng]]
3. link `order_axon → result` (with proof)
4. return result

the [[cybergraph]] is a universal, persistent, proven memo cache. every computation anyone ever did is reusable by everyone. the more the graph grows, the fewer computations actually execute

two [[cyberlinks]] per computation:

| Link | From | To | Who | What it records |
|------|------|----|-----|----------------|
| order | formula | object | neuron | "compute this" + payment |
| answer | order_axon | result | device | "here is the result" + proof |

the order axon `H(formula, object)` is itself a [[particle]] (axiom A6). anyone can query backlinks to it — "what results exist for this computation?" multiple devices can answer the same order. the [[coupling|ICBS]] market determines which answer the graph trusts

everything is a noun — a binary tree of [[Goldilocks field]] elements. programs are nouns. data is nouns. the result is a noun.

```
Layer 1: 16 deterministic compute patterns    Turing-complete + field arithmetic + bitwise + hash
Layer 2: call (16) + look (17)                witness injection + deterministic BBG read
Layer 3: 5 jets                               hash, poly_eval, merkle_verify, fri_fold, ntt
```

16 compute + call + look = 18 patterns total.

### the 18 patterns

| # | name | layer | domain | what it does |
|---|------|-------|--------|-------------|
| 0 | axis | 1 | structural | navigate tree by path. axis(0) = hash introspection |
| 1 | quote | 1 | structural | literal — code as data |
| 2 | compose | 1 | structural | chain computations. function application, recursion, control flow |
| 3 | cons | 1 | structural | build cell from two values |
| 4 | branch | 1 | structural | conditional. lazy — only evaluates taken path |
| 5 | add | 1 | field | field addition |
| 6 | sub | 1 | field | field subtraction |
| 7 | mul | 1 | field | field multiplication |
| 8 | inv | 1 | field | field inverse (Fermat) |
| 9 | eq | 1 | field | equality test |
| 10 | lt | 1 | field | less-than comparison |
| 11 | xor | 1 | bitwise | exclusive or |
| 12 | and | 1 | bitwise | bitwise and |
| 13 | not | 1 | bitwise | bitwise complement |
| 14 | shl | 1 | bitwise | shift left |
| 15 | hash | 1 | hash | structural hashing via [[hemera]] |
| 16 | call | 2 | witness | non-deterministic witness injection |
| 17 | look | 2 | state | deterministic read from BBG |

### canonical instantiation

```
F = Goldilocks       p = 2⁶⁴ - 2³² + 1
W = Z/2³²           32-bit words (fit cleanly in [0, p))
H = hemera           Poseidon2-Goldilocks sponge, 32-byte output
```

## companion repos

| repo | role | github |
|------|------|--------|
| [[nebu]] | field arithmetic | [nebu](https://github.com/cyberia-to/nebu) |
| [[hemera]] | hash function | [hemera](https://github.com/cyberia-to/hemera) |
| [[zheng]] | proof system | [zheng](https://github.com/cyberia-to/zheng) |
| [[trident]] | language compiler | [trident](https://github.com/cyberia-to/trident) |
| [[mudra]] | communication primitives | [mudra](https://github.com/cyberia-to/mudra) |
| [[bbg]] | authenticated state | [bbg](https://github.com/cyberia-to/bbg) |

## license

Cyber License: Don't trust. Don't fear. Don't beg.
