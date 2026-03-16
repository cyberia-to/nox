# completeness

why exactly these instruction groups — structural, field, bitwise, hash, hint — and why nothing else is needed.

## the five groups

nox has seventeen patterns organized into five groups. each group exists because it covers a distinct algebraic domain that the others cannot reach. removing any group cripples the system. adding more groups adds no capability.

```
STRUCTURAL (5)      tree algebra        Turing completeness
FIELD (6)           F_p arithmetic      proof-native computation
BITWISE (4)         Z/2^64 arithmetic   binary world interface
HASH (1)            cryptographic       identity and commitment
HINT (1)            non-deterministic   privacy and search
```

## group 1: structural (patterns 0-4) — tree algebra

```
0  axis    — navigate a noun tree
1  quote   — return a literal
2  compose — chain two computations (recursion)
3  cons    — build a cell (data construction)
4  branch  — conditional evaluation
```

these five patterns make nox Turing-complete. axis reads data. quote creates constants. cons builds structure. branch decides. compose enables recursion — it evaluates two subexpressions and applies one to the other, which is the mechanism for loops, function calls, and all control flow.

this is the core inherited from [[Nock]]. five rules for navigating and constructing binary trees, sufficient to express any computable function. every other group is an extension of this core — adding new operations over the same noun substrate.

the structural patterns operate in tree space. they do not know what a field element is. they do not know what a hash is. they manipulate structure: reading positions, building pairs, choosing paths. this is the universal layer — it works regardless of what atoms contain.

## group 2: field arithmetic (patterns 5-10) — F_p algebra

```
5  add — (a + b) mod p
6  sub — (a - b) mod p
7  mul — (a × b) mod p
8  inv — a^(p-2) mod p
9  eq  — equality test
10 lt  — less-than comparison
```

these six patterns give nox native arithmetic over the [[Goldilocks field]]. add, sub, mul form a ring. inv completes it to a field (every nonzero element has a multiplicative inverse). eq and lt provide comparison.

why field arithmetic? because the [[stark]] proof system operates over a finite field. every constraint in the proof is a polynomial equation over F_p. if the VM's arithmetic is field arithmetic, then the execution trace IS the proof witness — no translation required. the program computes `a + b mod p`. the constraint checks `result = a + b mod p`. same operation, same field, zero impedance.

why these six operations specifically? because {+, -, ×, ÷} is the complete set of field operations, and {=, <} is the minimal set of comparisons. you can build any polynomial from add, sub, and mul. you can solve any equation with inv. you can branch on any condition with eq and lt. there is no field operation that cannot be expressed as a composition of these six.

could the system work without field arithmetic, using only structural patterns? yes — in the same way Nock works with only decrement. you could implement addition as iterated increment, multiplication as iterated addition. but the cost would be catastrophic: O(p) operations for a single addition, where p ≈ 2^64. and every operation would require ~64 [[stark]] constraints instead of 1. the field patterns are the reason nox can prove computations efficiently.

## group 3: bitwise (patterns 11-14) — Z/2^64 algebra

```
11 xor — exclusive or
12 and — bitwise and
13 not — bitwise complement
14 shl — left shift
```

these four patterns give nox native operations over 64-bit words. xor, and, and not form a complete Boolean algebra (any Boolean function can be expressed as a combination of these three). shl provides positional manipulation.

why bitwise? because the world outside nox speaks binary. network protocols encode data as bit sequences. cryptographic primitives operate on bits. file formats, compression, error correction — all binary. without bitwise patterns, nox would need to decompose every byte into 8 field elements and simulate bit operations as field arithmetic. this is possible but costs ~64 [[stark]] constraints per bit operation (the bit decomposition overhead).

the bitwise patterns accept this cost once — they include the bit decomposition in their constraint layout — and expose clean O(1) execution cost to the programmer. the [[stark]] cost (~64 constraints each) is higher than field arithmetic (~1 constraint), reflecting the real algebraic cost of bit operations in a prime field. this cost model is honest: bitwise is more expensive than arithmetic because F_p and Z/2^64 are different algebras.

why only four operations? because {xor, and, not} is a functionally complete Boolean basis (NAND alone suffices, but xor+and+not is more practical), and shl covers both left shift (direct) and right shift (via `shl(a, 64-n)` plus mask). right shift, rotate, and other bit manipulations are expressible as compositions.

## group 4: hash (pattern 15) — cryptographic identity

```
15 hash — H(a) → 4 × F_p (256-bit Hemera digest)
```

one pattern. but it closes the entire identity loop.

hash gives nox intrinsic content-addressing. every noun can compute its own cryptographic fingerprint. `axis(s, 0)` returns `H(s)` — a noun can know its own identity. this is the primitive that makes the [[cybergraph]] possible: [[particles]] are identified by hash, [[cyberlinks]] connect hashes, the computation cache keys on hashes.

could hash be expressed as pure structural + field patterns? yes. [[Hemera]] (Poseidon2) is ~2800 field multiplications and additions. the hash pattern is simultaneously a Layer 1 pattern and a Layer 3 jet — the jet provides an optimized constraint layout (300 constraints instead of ~2800), but the semantics are identical.

the reason hash is a dedicated pattern rather than a library function: it appears in every meaningful operation. identity verification is a hash. Merkle trees are hashes. [[stark]] Fiat-Shamir challenges are hashes. content addressing is a hash. making hash a pattern means the most common expensive operation has the most optimized constraint layout. 83% of the stark verifier's cost is hash operations — this single pattern, jetted, accounts for the largest share of the 8.5× recursive verification speedup.

## group 5: hint (pattern 16) — non-deterministic witness

```
16 hint — prover injects, Layer 1 verifies
```

one pattern. the entire mechanism of privacy, search, and oracle access.

hint is what separates nox from a transparent calculator. without hint, every computation is publicly reproducible — the verifier can re-run the program and learn everything the prover knows. hint creates the information asymmetry that makes [[zero knowledge proofs]] possible: the prover injects private knowledge, Layer 1 constraints verify it, the verifier checks the [[stark]] proof without learning the secret.

hint is the only non-deterministic pattern. the other sixteen are fully deterministic — same input always produces the same output. hint produces a result that depends on what the prover injects. this breaks confluence intentionally, creating the gap between prover knowledge and verifier knowledge that ZK exploits.

could the system work without hint? yes — as a transparent, verifiable computation engine. all the other properties (confluence, content-addressing, memoization, proof-nativity) remain. but without hint, there are no private transactions, no ZK identity proofs, no ability to prove without revealing. hint is the minimum necessary non-determinism.

## the sufficiency argument

the five groups cover five algebraic domains:

```
trees     — universal computation (Turing complete)
F_p       — proof-native arithmetic (stark-compatible)
Z/2^64    — binary world interface (protocols, formats)
H(·)      — cryptographic identity (content addressing)
oracle    — non-determinism (privacy, ZK)
```

any computation that [[cyber]] needs falls into one of these domains:

- identity verification → hash + hint (prove H(secret) = address)
- state transitions → structural + field (tree transformation with arithmetic)
- network protocols → bitwise (packet encoding, flag testing)
- ranking → field (focus flow is field arithmetic over the graph)
- [[stark]] verification → field + hash (polynomial evaluation, Merkle paths)
- private transactions → hint + field + hash (witness injection, conservation checks)
- AI inference → field + structural (matrix operations as noun transformations)

the nine [[cyb/architecture|computation languages]] (Nox, Bt, Rs, Trident, Arc, Seq, Ask, Wav, Ten) all compile through nox as their structural IR. each language maps to a subset of the five groups:

```
Nox      → structural (direct)
Bt       → bitwise (F_2 tower maps to Z/2^64)
Rs       → bitwise + field (system-level word operations)
Trident  → field + hash (proof-oriented programs)
Arc      → structural + field (graph adjacency as nested pairs)
Seq      → structural (partial orders as trees)
Ask      → structural + field (Datalog unification over nouns)
Wav      → field (signal processing as field arithmetic)
Ten      → field (tensor contraction as field multiplication)
```

## why nothing else is needed

no sixth group is necessary because:

- floating point: unnecessary. all quantities are field elements. where approximate arithmetic is needed (AI inference), fixed-point representation over F_p suffices. [[stark]] proofs cannot verify floating point anyway.
- string operations: unnecessary. strings are lists of atoms (character codes). string manipulation is tree manipulation (structural patterns).
- I/O: unnecessary inside the VM. nox is a pure computation engine. I/O happens at the boundary — [[radio]] handles networking, [[bbg]] handles storage. the VM transforms nouns; the environment provides and consumes them.
- exceptions: unnecessary. errors propagate as values (⊥_error, ⊥_unavailable). no stack unwinding, no try/catch. error handling is tree navigation.
- concurrency primitives: unnecessary. confluence guarantees safe parallelism. no locks, no channels, no atomic operations. the rewrite system's mathematics provides all the concurrency safety needed.

## the four-bit argument

sixteen patterns fit in four bits. this is deliberate. the pattern tag is a single nibble. the encoding is maximally dense — a nox formula is a binary tree where each node's tag occupies exactly 4 bits. compact encoding means:

- shorter programs hash faster (less data through [[Hemera]])
- smaller proofs (fewer bits to commit in the [[stark]] trace)
- denser caching (more computation identities per unit of storage)
- cheaper transmission (less bandwidth per program over [[radio]])

adding a seventeenth deterministic pattern would require 5 bits for the tag, wasting half the encoding space. hint (pattern 16) already occupies position 10000 in binary — the first 5-bit tag. this is acceptable because hint is rare (used only at ZK injection points). a system with 32 patterns would waste encoding density on patterns that are used less than 1% of the time.

sixteen is the sweet spot: enough for algebraic completeness, few enough for encoding density, exactly the number that fits a nibble.
