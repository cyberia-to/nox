# nox

proof-native virtual machine for [[cyber]]. every execution produces a STARK proof as a byproduct — running a program and proving it ran correctly are the same act. there is no separate arithmetization step. the execution trace IS the algebraic constraint system.

## lineage

```
combinatory logic (1924)   S, K combinators            pure abstraction
  → lambda calculus (1936) Church's untyped lambda      computable functions
  → Nock (2016)            natural numbers + decrement  deterministic virtual machine for Urbit
  → nox (2026)             field elements + inverse     proof-native virtual machine for cyber
```

nox replaces Nock's natural numbers with [[Goldilocks field]] elements and decrement with field inverse. this single substitution makes the virtual machine algebraically native — every operation maps directly to field constraints with zero translation overhead.

## why nox is amazing

**proof-native execution.** most virtual machines bolt proofs onto execution after the fact — run the program, then arithmetize the trace, then prove. nox skips the middle step. the execution trace is already a valid STARK witness. `reduce(subject, formula)` simultaneously computes the result and generates the proof artifact.

**algebra polymorphism.** the same 16 reduction patterns work over any field, any word width, any hash function. pattern semantics are universal — algebra is a parameter. a single nox program runs over Goldilocks (STARKs), F₂ (Binius), or F_{p³} (recursive composition) by selecting a different instantiation. one spec, many proof systems.

**merkle by construction.** every `cons(a, b)` builds a Merkle tree — the hash is computed and stored at the parent node. `axis` traversal produces Merkle proofs as a side effect. content addressing is not a feature layered on top — it is the data model itself.

**minimal irreducible design.** 16 deterministic patterns, 1 non-deterministic hint, 5 optimization jets. remove the jets — identical results, ~8.5× slower. remove the hint — no privacy, no ZK, but still Turing-complete. remove Layer 1 — nothing remains. every pattern earns its place.

**privacy at the boundary.** the hint pattern (16) injects untrusted witness data that Layer 1 constraints verify. this is where privacy enters: the prover knows the secret, the verifier checks the math. no trusted setup. no MPC ceremony. the architecture separates knowledge from verification.

**lazy evaluation.** the branch pattern (4) evaluates only the taken path. the other branch is never touched. this prevents infinite-recursion DoS attacks structurally — a property of the reduction semantics, not a runtime check.

**unified IR.** nox is simultaneously the intermediate representation (all [[cyber]] languages compile through it), the node runtime (production blockchain binary), and the composition tier (orchestrating programs across execution contexts). one representation from source to proof.

## architecture

```
reduce(subject, formula, focus) → result
```

everything is a noun — a binary tree of [[Goldilocks field]] elements. programs are nouns. data is nouns. the result is a noun.

```
Layer 1: 16 deterministic patterns    Turing-complete + field arithmetic + bitwise + hash
Layer 2: 1 non-deterministic hint     witness injection, privacy boundary, verified by Layer 1
Layer 3: 5 jets                       hash, poly_eval, merkle_verify, fri_fold, ntt
```

### the 16 patterns

| # | name | domain | what it does |
|---|------|--------|-------------|
| 0 | axis | structural | navigate tree by path. axis(0) = hash introspection |
| 1 | quote | structural | literal — code as data |
| 2 | compose | structural | chain computations. function application, recursion, control flow |
| 3 | cons | structural | build cell from two values |
| 4 | branch | structural | conditional. lazy — only evaluates taken path |
| 5 | add | field | field addition |
| 6 | sub | field | field subtraction |
| 7 | mul | field | field multiplication |
| 8 | inv | field | field inverse (Fermat) |
| 9 | eq | field | equality test |
| 10 | lt | field | less-than comparison |
| 11 | xor | bitwise | exclusive or |
| 12 | and | bitwise | bitwise and |
| 13 | not | bitwise | bitwise complement |
| 14 | shl | bitwise | shift left |
| 15 | hash | hash | structural hashing via [[hemera]] |

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
| [[mudra]] | crypto primitives | [mudra](https://github.com/cyberia-to/mudra) |
| [[bbg]] | authenticated state | [bbg](https://github.com/cyberia-to/bbg) |

## license

Cyber License: Don't trust. Don't fear. Don't beg.
