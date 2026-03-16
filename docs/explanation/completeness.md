# completeness

why exactly these instruction groups — structural, field, bitwise, hash, hint — and why nothing else is needed.

## the five groups

nox has sixteen deterministic patterns organized into four groups, plus a non-deterministic prover protocol (hint). each group covers a distinct algebraic domain that the others cannot reach. removing any group cripples the system. adding more groups adds no capability.

```
STRUCTURAL (5)      tree algebra        Turing completeness          4-bit encoded
FIELD (6)           F_p arithmetic      proof-native computation     4-bit encoded
BITWISE (4)         Z/2^64 arithmetic   binary world interface       4-bit encoded
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

these five patterns are the computational core. they make nox Turing-complete — sufficient to express any computable function — and every other group is an extension built on top of them.

the structural patterns operate in tree space. they do not know what a field element is. they do not know what a hash is. they manipulate pure structure: positions in a tree, pairs of values, paths through branches. this is the universal layer — it works regardless of what atoms contain.

### axis: reading

axis navigates the subject tree by numeric address. the address is a natural number that encodes a path from root to leaf: 1 is the root, 2 is the left child, 3 is the right child, 4 is left-left, 5 is left-right, and so on. the binary representation of the axis number IS the path — after the leading 1 bit, each 0 means "go left" and each 1 means "go right."

this is the only way to read data from the subject. there are no variables, no names, no symbol tables. a program that needs "the third element of a list" computes the axis address and navigates there. the address is a value — it can be computed, stored, passed as an argument. data access is arithmetic on positions.

axis also serves a second role: `axis(subject, 0)` returns the cryptographic hash of the subject. axis zero is identity — the noun's fingerprint. this is how content-addressing enters at the structural level: any noun can know its own hash without invoking the hash pattern explicitly.

### quote: constants

quote returns its argument unchanged, without evaluating it. `[1 x]` produces `x` regardless of what the subject is. this is how literal values enter a computation — numbers, code templates, static data structures.

quote is the dual of axis. axis reads from the environment (the subject). quote ignores the environment entirely. between them, a nox program can reference any existing value (axis) or introduce any new value (quote).

### compose: recursion

compose is the engine of all computation. `[2 a b]` evaluates `a` against the subject to get a new subject, evaluates `b` against the subject to get a formula, then applies the formula to the new subject. this is function application — the universal mechanism for loops, recursion, subroutine calls, and every form of control flow.

compose is why five patterns suffice for Turing completeness. a formula can compose itself — evaluating a formula that produces another formula application, which produces another, indefinitely. this is recursion without a recursion primitive. the pattern does not know it is recursing; it simply evaluates a formula against a subject, and if the result is another computation, it evaluates that too.

every higher-level construct — while loops, map/fold, pattern matching, function dispatch — compiles down to compositions. compose is the sole source of computational depth in nox.

### cons: construction

cons builds a cell from two evaluated subexpressions. `[3 a b]` evaluates both `a` and `b` against the subject and pairs the results into `[result_a result_b]`. this is data construction — the only way to build new compound values.

cons is inherently parallel. its two subexpressions are independent — neither depends on the other's result. a parallel evaluator can dispatch both branches to separate threads. this is not an optimization hint; it is a mathematical fact. the two subexpressions share the same subject but cannot affect each other's evaluation. confluence guarantees that evaluating them in either order, or simultaneously, produces identical results.

### branch: decision

branch is conditional evaluation. `[4 test yes no]` evaluates `test` against the subject. if the result is 0, it evaluates `yes`. if the result is 1, it evaluates `no`. any other result is an error.

branch is the only pattern that discards computation. one of its two branches is never evaluated — the formula for the path not taken is syntactically present but semantically absent. this is how nox avoids wasted [[focus]]: the branch not chosen costs nothing.

### why five is enough

axis provides input. quote provides constants. cons provides output (construction). branch provides choice. compose provides depth (recursion). these five operations span the space of computable functions: read data, create data, combine data, choose between alternatives, and repeat. Church and Turing proved in 1936 that this is sufficient — any function computable by any mechanism whatsoever can be expressed as a combination of these primitives.

the structural patterns are inherited from [[Nock]], which inherited them from the combinatory logic tradition. S and K combinators (1924) proved two operations suffice for universality. lambda calculus (1936) proved one operation (application with binding) suffices. Nock showed that tree navigation and construction suffice. nox preserves this core unchanged — because it is already minimal.

## group 2: field arithmetic (patterns 5-10) — F_p algebra

```
5  add — (a + b) mod p
6  sub — (a - b) mod p
7  mul — (a × b) mod p
8  inv — a^(p-2) mod p
9  eq  — equality test
10 lt  — less-than comparison
```

these six patterns give nox native arithmetic over the [[Goldilocks field]] (p = 2^64 - 2^32 + 1). they are the reason nox produces proofs natively — without them, the system would be Turing-complete but proof-hostile.

### the field algebra

add, sub, and mul form a commutative ring. every field element can be added to, subtracted from, or multiplied with any other, and the result stays in the field (mod p ensures no overflow, no underflow, no undefined behavior). these three operations build any polynomial — and polynomials are the language of [[stark]] proofs.

inv completes the ring to a field. given any nonzero element a, `inv(a)` returns a^(p-2) mod p — the unique element such that `a × inv(a) = 1`. this is Fermat's little theorem: in a prime field, a^(p-1) = 1, so a^(p-2) is the multiplicative inverse. inv costs 64 field multiplications (square-and-multiply over the 64-bit exponent), making it the most expensive field pattern — but it enables division, which enables solving equations, which enables the full power of algebraic reasoning.

eq tests equality: returns 0 (true) if two field elements are identical, 1 (false) otherwise. lt tests ordering: returns 0 if a < b in the canonical integer representation of the field. together they provide the minimal comparison set — any predicate over field elements can be built from equality and ordering.

### why field arithmetic matters

the [[stark]] proof system operates over a finite field. every constraint in the proof is a polynomial equation over F_p. if the VM's arithmetic is field arithmetic, the execution trace IS the proof witness:

```
program computes:     a + b mod p = c
STARK constraint:     a + b mod p = c
```

same operation. same field. zero translation. the program runs and the proof writes itself — the sequence of field operations during execution is exactly the algebraic intermediate representation (AIR) that the prover proves and the verifier checks. there is no "circuit compilation" step. there is no "arithmetization" pass. the computation is already arithmetic.

this is the fundamental insight that separates nox from every other VM. conventional zkVMs (risc0, SP1, Valida) execute programs in one algebra (integers, bytes, registers) and then translate the execution into field constraints. that translation is where complexity, bugs, and performance loss live. nox eliminates the translation by making the execution algebra identical to the proof algebra.

### the cost model

each field operation (add, sub, mul) generates exactly 1 [[stark]] constraint. this is the theoretical minimum — one algebraic equation to verify one algebraic operation. inv generates ~64 constraints (the square-and-multiply chain). eq and lt generate ~32 constraints each (bit decomposition for comparison). the cost is honest and predictable: a programmer can count the field operations in a formula and know exactly how many constraints the proof will contain.

contrast this with Nock, which has only increment as its arithmetic primitive. decrement must be built by counting up from 0 to n-1 — an O(n) loop. addition is iterated increment: O(a+b). multiplication is iterated addition: O(a×b). multiplying two 64-bit numbers takes O(2^64) steps. in a proof system, each step is a constraint, so Nock arithmetic would produce astronomically large proofs. the field patterns collapse all of this: O(1) execution, O(1) constraints, for every arithmetic operation.

### why these six and no others

{+, -, ×, ÷} is the complete set of field operations. any algebraic expression over F_p decomposes into additions, subtractions, multiplications, and inversions. exponentiation is repeated multiplication. division is multiplication by inverse. square roots are exponentiations (Tonelli-Shanks over F_p). there is no field operation that escapes these four.

{=, <} is the minimal set of comparisons. equality handles exact matching. ordering handles range checks, bounds verification, and sorting. greater-than is `lt` with swapped arguments. not-equal is the complement of eq. between them, every comparison predicate is expressible.

a seventh field pattern would be redundant. a fifth arithmetic operation would decompose into the existing four. a third comparison would be syntactic sugar for a composition of eq and lt. six is the exact count: four for algebra, two for comparison, nothing wasted.

## group 3: bitwise (patterns 11-14) — Z/2^64 algebra

```
11 xor — exclusive or
12 and — bitwise and
13 not — bitwise complement
14 shl — left shift
```

these four patterns give nox native operations over 64-bit words. they exist because the world outside nox speaks binary — and because F_p and Z/2^64 are fundamentally different algebras that cannot simulate each other cheaply.

### the two-algebra problem

nox lives in F_p — a prime field where arithmetic wraps at p = 2^64 - 2^32 + 1. the external world lives in Z/2^64 — binary words where arithmetic wraps at 2^64. these two algebras share a representation (64-bit integers) but obey different laws. addition in F_p is not addition in Z/2^64 when the result exceeds p. XOR has no natural expression as field arithmetic. AND has no polynomial representation over a prime field.

without bitwise patterns, every interaction with binary data would require bit decomposition: split a 64-bit field element into 64 individual bits (each 0 or 1), perform Boolean logic on the bits, then reassemble the result. this decomposition costs ~64 [[stark]] constraints per operation — the algebraic cost of proving that 64 individual values are each 0 or 1 and that they sum to the original number.

the bitwise patterns absorb this cost into their constraint layout. each pattern includes the bit decomposition in its [[stark]] constraints (~64 constraints per operation) and exposes clean O(1) execution cost to the programmer. the programmer writes `xor(a, b)`. the proof system handles the decomposition internally. the cost model is honest: bitwise is ~64× more expensive than field arithmetic in proof size, because that is the real algebraic distance between F_p and Z/2^64.

### the Boolean basis

xor, and, and not form a functionally complete Boolean algebra. any Boolean function of any number of inputs can be expressed as a combination of these three operations:

- OR: `or(a, b) = xor(and(a, b), xor(a, b))`
- NAND: `nand(a, b) = not(and(a, b))`
- NOR: `nor(a, b) = not(or(a, b))`
- implication, equivalence, majority, multiplexer — all expressible

NAND alone is functionally complete (every Boolean function can be built from NAND gates). but xor+and+not is the practical choice: XOR is the fundamental operation of [[cryptography]] (stream ciphers, hash functions, error correction), AND is the fundamental operation of masking (bit extraction, flag testing), and NOT is the fundamental operation of complement. choosing these three means the most common binary operations are single patterns, not multi-pattern compositions.

### shift: positional manipulation

shl (shift left) moves bits toward higher positions, filling vacated positions with zeros. `shl(a, n)` multiplies a by 2^n in the binary interpretation. this single operation covers a surprising range of bit manipulation:

- left shift: `shl(a, n)` directly
- right shift: `and(shl(a, 64-n), mask)` — shift left by the complement, mask off the overflow
- bit extraction: `and(shl(a, 64-k), mask)` — shift the target bit to a known position, mask it
- rotation: `xor(shl(a, n), shl(a, 64-n))` with appropriate masking
- byte swapping, bit reversal, field packing — all compositions of shift, and, xor

a dedicated right shift pattern would save one operation per right shift. but it would consume a precious 4-bit encoding slot for something expressible as a two-pattern composition. the design chooses encoding density over convenience — consistent with the principle that patterns exist for algebraic necessity, not programmer comfort.

### what bitwise enables

the binary world is vast: network protocols encode headers as bit fields. file formats pack data into byte sequences. cryptographic primitives ([[Hemera]] internally, but also any future hash or cipher) operate on bits. error-correcting codes manipulate polynomial coefficients over F_2. compression algorithms work with variable-length bit strings.

without the bitwise group, nox could still compute all these functions (Turing completeness guarantees it), but the cost would be prohibitive. parsing a single network packet header — extracting flags, lengths, checksums from bit positions — would require hundreds of field operations simulating bit extraction. with bitwise patterns, it requires a handful of AND and SHL operations.

the bitwise group is the bridge between nox's native algebra (F_p) and the binary substrate of the physical world. it costs more in proof size than field arithmetic — honestly reflecting the algebraic distance — but it makes nox a practical system that can interact with existing protocols, formats, and standards without translating everything into pure field arithmetic first.

## group 4: hash (pattern 15) — cryptographic identity

```
15 hash — H(a) → 4 × F_p (256-bit Hemera digest)
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
