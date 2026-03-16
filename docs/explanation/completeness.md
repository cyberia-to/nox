# completeness

why exactly these instruction groups — structural, field, bitwise, hash, hint — and why nothing else is needed.

## the five groups

nox has sixteen deterministic patterns organized into four groups, plus a non-deterministic prover protocol (hint). each group covers a distinct algebraic domain that the others cannot reach. removing any group cripples the system. adding more groups adds no capability.

```
STRUCTURAL (5)      tree algebra        Turing completeness          4-bit encoded
FIELD (6)           F_p arithmetic      proof-native computation     4-bit encoded
BITWISE (4)         Z/2^32 arithmetic   binary world interface       4-bit encoded
HASH (1)            cryptographic       identity and commitment      4-bit encoded
HINT                non-deterministic   privacy and search           prover protocol
```

## group 1: structural (patterns 0-4) — tree algebra

```
0  axis    — navigate a noun tree
1  quote   — return a literal
2  compose — chain two computations (recursion)
3  cons    — build a cell (data construction)
4  branch  — conditional evaluation
```

these five patterns make nox Turing-complete. axis reads data. quote creates constants. cons builds structure. branch decides. compose enables recursion. five operations that span the space of computable functions: read, create, combine, choose, repeat. this core is inherited from [[Nock]] and the combinatory logic tradition — it is already minimal. see [[structural-patterns]] for the full explanation of each pattern.

## group 2: field arithmetic (patterns 5-10) — F_p algebra

```
5  add — (a + b) mod p
6  sub — (a - b) mod p
7  mul — (a × b) mod p
8  inv — a^(p-2) mod p
9  eq  — equality test
10 lt  — less-than comparison
```

six patterns for native arithmetic over the [[Goldilocks field]]. add, sub, mul form a ring. inv completes it to a field. eq and lt provide comparison. the reason these exist: the [[stark]] proof system operates over F_p, so if the VM's arithmetic is field arithmetic, the execution trace IS the proof witness — zero translation. see [[field-patterns]] for the full algebra and cost model.

## group 3: bitwise (patterns 11-14) — Z/2^32 algebra

```
11 xor — exclusive or
12 and — bitwise and
13 not — bitwise complement
14 shl — left shift
```

four patterns for native operations over 32-bit words. xor, and, not form a functionally complete Boolean algebra. shl provides positional manipulation. these exist because F_p and Z/2^32 are fundamentally different algebras — simulating bit operations as field arithmetic costs ~32× more in proof size. 32-bit words fit cleanly in Goldilocks ([0, 2^32) ⊂ [0, p)), avoiding the overflow gap that makes 64-bit words unrepresentable. heavy binary computation belongs in [[Bt]] (FRI-Binius, characteristic 2). see [[bitwise-patterns]] for the two-algebra problem and Boolean basis.

## group 4: hash (pattern 15) — cryptographic identity

```
15 hash — H(a) → 8 × F_p (64-byte Hemera digest)
```

one pattern. but it closes the entire identity loop.

hash gives nox intrinsic content-addressing. every noun can compute its own cryptographic fingerprint. `axis(s, 0)` returns `H(s)` — a noun can know its own identity. this is the primitive that makes the [[cybergraph]] possible: [[particles]] are identified by hash, [[cyberlinks]] connect hashes, the computation cache keys on hashes.

could hash be expressed as pure structural + field patterns? yes. [[Hemera]] (Poseidon2) is ~2800 field multiplications and additions. the hash pattern is simultaneously a Layer 1 pattern and a Layer 3 jet — the jet provides an optimized constraint layout (300 constraints instead of ~2800), but the semantics are identical.

the reason hash is a dedicated pattern rather than a library function: it appears in every meaningful operation. identity verification is a hash. Merkle trees are hashes. [[stark]] Fiat-Shamir challenges are hashes. content addressing is a hash. making hash a pattern means the most common expensive operation has the most optimized constraint layout. 83% of the stark verifier's cost is hash operations — this single pattern, jetted, accounts for the largest share of the 8.5× recursive verification speedup.

## group 5: hint — non-deterministic witness

```
hint — prover injects, constraints verify
```

not an opcode. a prover/verifier protocol. the entire mechanism of privacy, search, and oracle access.

hint is what separates nox from a transparent calculator. without hint, every computation is publicly reproducible — the verifier can re-run the program and learn everything the prover knows. hint creates the information asymmetry that makes [[zero knowledge proofs]] possible: the prover injects private knowledge, Layer 1 constraints verify it, the verifier checks the [[stark]] proof without learning the secret.

hint is the only non-deterministic mechanism. the sixteen deterministic patterns always produce the same output from the same input. hint produces a result that depends on what the prover injects. this breaks confluence intentionally, creating the gap between prover knowledge and verifier knowledge that ZK exploits.

could the system work without hint? yes — as a transparent, verifiable computation engine. all the other properties (confluence, content-addressing, memoization, proof-nativity) remain. but without hint, there are no private transactions, no ZK identity proofs, no ability to prove without revealing. hint is the minimum necessary non-determinism.

## the sufficiency argument

the five groups cover five algebraic domains:

```
trees     — universal computation (Turing complete)
F_p       — proof-native arithmetic (stark-compatible)
Z/2^32    — binary world interface (flags, masking, branching logic)
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
Bt       → bitwise (F_2 tower, native binary — heavy binary goes here, not nox)
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

## the four-bit core

sixteen deterministic patterns fit in four bits. the pattern tag is a single nibble — the encoding is maximally dense. a nox formula is a binary tree where each node's tag occupies exactly 4 bits. compact encoding means:

- shorter programs hash faster (less data through [[Hemera]])
- smaller proofs (fewer bits to commit in the [[stark]] trace)
- denser caching (more computation identities per unit of storage)
- cheaper transmission (less bandwidth per program over [[radio]])

everything beyond the sixteen patterns lives outside the encoding:

```
4-bit tag     16 deterministic patterns     the encoding, the wire format, the STARK trace
              frozen forever

runtime       jets                          recognized by formula hash, transparent optimization
              hash is simultaneously pattern 15 AND a jet (optimized constraint layout)
              poly_eval, merkle_verify, fri_fold, ntt — pure pattern compositions, jetted for speed

prover        hint                          prover/verifier protocol, not an opcode
              the prover injects a witness, constraints verify it
              never appears in the encoded formula — it is a runtime interaction
```

jets do not need opcodes. a jet is a formula tree made of the sixteen core patterns — the runtime recognizes it by `H(formula) == KNOWN_HASH` and substitutes optimized native code. the encoding on the wire is still 4-bit tagged core patterns. as more jets are added over time (recursive [[stark]] verification, new cryptographic primitives, AI inference kernels), the encoding never grows. jets are a runtime layer, not an encoding layer.

hint does not need an opcode either. it is a prover/verifier protocol — the prover signals "I will inject a witness here," the [[stark]] constraints verify the witness is valid, the verifier checks the proof without learning the secret. hint lives in the interaction between prover and verifier, not in the formula encoding.

the result: the wire format is 4 bits per node, forever. sixteen is the exact number — enough for algebraic completeness across five domains, few enough to fill a nibble with zero waste. adding a seventeenth deterministic pattern would require 5 bits, wasting half the encoding space on tags that will never be used. the four-bit boundary is both a mathematical optimum and a forcing function: it disciplines the design to include only what is algebraically necessary.
