# proof-native computation

execution IS proof — why there is no circuit compilation step, and why this matters more than anything else in the design.

## the identity

in every other proof system, there is a translation step. you write a program. then a compiler transforms it into an arithmetic circuit (R1CS, Plonkish, AIR). the circuit is a different representation — different structure, different optimization concerns, different debugging surface. the programmer thinks in one world; the prover proves in another.

in nox, the program IS the circuit. the execution trace — the sequence of register states across all reduction steps — is directly the algebraic intermediate representation (AIR) that the [[stark]] prover proves and the verifier checks.

```
nox execution trace          →    stark witness
register state at each step  →    trace row
pattern tag                  →    constraint selector
pattern semantics            →    transition constraint polynomial
```

there is no separate compilation. there is no intermediate representation that could diverge from the program's semantics. the program runs. the trace records what happened. the trace IS the proof witness. the [[stark]] verifies the trace.

## why the field choice is everything

this identity is possible because of the field choice. nox arithmetic IS [[Goldilocks field]] arithmetic. the execution trace IS a table of Goldilocks elements. the [[stark]] proof IS over Goldilocks. there is no impedance mismatch at any layer.

```
program:  add(a, b)  →  (a + b) mod p
trace:    row[t] = { pattern: 5, operand_a: a, operand_b: b, result: (a+b) mod p }
constraint: result_{t+1} = operand_a_{t} + operand_b_{t}  (degree 1 over F_p)
```

the program addition is the same operation as the constraint addition. the field element in the program is the same field element in the proof. there is no conversion, no embedding, no approximation.

contrast this with a VM that operates on 256-bit integers but proves over a 64-bit field. the prover must decompose each 256-bit operation into multiple 64-bit constraints. the translation is correct but expensive, and every translation step is a potential source of bugs. nox eliminates the translation entirely.

## the sixteen constraints

each of the sixteen patterns becomes an AIR transition constraint:

```
pattern 5  (add):  result_{t+1} = a_t + b_t              degree 1
pattern 7  (mul):  result_{t+1} = a_t × b_t              degree 2
pattern 8  (inv):  result_{t+1} × a_t = 1                degree 2
pattern 15 (hash): Poseidon2 round constraints            degree 7
pattern 4  (branch): selector × yes + (1-selector) × no  degree 2
```

the constraint selector — `pattern_tag_t = N` — gates each pattern's constraints. only the active pattern's constraints apply per row. [[SuperSpartan]]'s CCS (Customizable Constraint System) handles mixed degrees natively — no degree padding, no uniform arithmetization.

sixteen patterns means sixteen constraint families. this is manageable. a conventional VM with hundreds of opcodes produces hundreds of constraint families, each requiring separate verification logic. nox's minimalism at the instruction level translates directly to simplicity at the proof level.

## the trace

the execution trace has 16 registers and 2^n rows (padded to a power of 2):

```
r0:  pattern tag (0-16)           which rule fired
r1:  subject hash                 H(current subject)
r2:  formula hash                 H(current formula)
r3:  operand A                    first evaluated operand
r4:  operand B                    second evaluated operand
r5:  result                       output value
r6:  focus before                 budget entering this step
r7:  focus after                  budget leaving this step
r8-r10: type tags                 for A, B, result
r11-r14: auxiliary                 pattern-specific data
r15: status                       0=ok, 1=halt, 2=error
```

each row is one reduction step. the trace is a complete record of what the program did — every operation, every intermediate value, every focus decrement. the [[stark]] prover commits to this trace as a multilinear polynomial, and the verifier checks the transition constraints via [[sumcheck]].

## what this means for programmers

for the programmer writing in [[trident]] (the high-level language): the cost model is transparent. every trident operation compiles to a known number of nox patterns. each pattern has a known focus cost and a known constraint count. the programmer can predict the proving cost of their program at compile time.

there are no optimization surprises. a JIT compiler cannot change the constraint count. an interpreter cannot introduce constraints the programmer did not expect. the cost is structural — it follows from the program's shape, not from an optimizer's decisions.

for the auditor: the proof covers exactly what the program did. if the program has a bug, the bug is in the trace. if the trace satisfies the constraints, the program ran correctly. there is no gap between "what was proved" and "what ran" because they are the same thing.

## what this means for the network

for the [[cyber]] network: every computation submitted by a [[neuron]] comes with a proof. the proof is ~60-157 KiB regardless of how large the computation was. the verifier checks it in O(log n) time. the cost of verification is constant with respect to the computation's size.

this is the enabler of scalable consensus. the network does not re-execute computations to verify them. it checks proofs. a million-step computation produces the same size proof as a thousand-step computation. the network's verification throughput is independent of the complexity of the computations it processes.

## the deeper point

proof-nativity is the single design decision from which most of nox's properties flow:

- content-addressing works because the trace deterministically follows from the computation
- memoization works because the proof certifies the result
- parallelism works because confluence is a theorem about the constraint system
- privacy works because hint creates an information asymmetry within the same constraint framework
- recursive verification works because the verifier is a nox program operating in the same field

every other proof system has a "compilation gap" — the distance between the program and the proof. nox closes this gap to zero. the program and the proof are the same mathematical object, viewed from different angles. this is the deepest insight of the design, and it is the reason nox exists as a separate VM rather than using an existing instruction set with a bolted-on proof system.
