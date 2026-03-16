# field patterns

six patterns for native arithmetic over the [[Goldilocks field]] — the reason nox produces proofs natively.

```
5  add — (a + b) mod p
6  sub — (a - b) mod p
7  mul — (a × b) mod p
8  inv — a^(p-2) mod p
9  eq  — equality test
10 lt  — less-than comparison
```

without these six patterns, nox would be Turing-complete but proof-hostile. with them, the execution trace IS the proof witness — same algebra, same field, zero translation.

## the field

the [[Goldilocks field]] is F_p where p = 2^64 - 2^32 + 1. every atom in nox is an element of this field — an integer from 0 to p-1. the choice of p is not arbitrary: Goldilocks admits fast reduction (the special form 2^64 - 2^32 + 1 allows modular reduction using only shifts and adds on 64-bit hardware), has multiplicative subgroups of size 2^32 (enabling efficient NTT for [[stark]] polynomial arithmetic), and fits in a single 64-bit machine word (every field operation maps to a handful of CPU instructions).

this is the field that the [[stark]] proof system operates over. every constraint in a [[stark]] proof is a polynomial equation over F_p. the choice of field is the choice of proof system — they are inseparable.

## the ring: add, sub, mul

add, sub, and mul form a commutative ring. every field element can be added to, subtracted from, or multiplied with any other, and the result stays in the field. mod p ensures no overflow, no underflow, no undefined behavior — the arithmetic is total and closed.

```
add(a, b) = (a + b) mod p
sub(a, b) = (a - b) mod p     equivalently: (a + (p - b)) mod p
mul(a, b) = (a × b) mod p
```

subtraction is addition by the additive inverse. `sub(a, b)` produces `a + (p - b) mod p`, which is always a valid field element. there is no concept of "negative numbers" — subtraction wraps around the field. `sub(0, 1)` produces p-1, the largest element.

these three operations build any polynomial. a polynomial over F_p is a sum of products of field elements — additions and multiplications composed. polynomials are the language of [[stark]] proofs: every constraint is a polynomial equation, every trace column is evaluated as a polynomial, every commitment is to a polynomial. the ring operations are the vocabulary of provable computation.

## the field: inv

inv completes the ring to a field. given any nonzero element a, `inv(a)` returns a^(p-2) mod p — the unique element such that `a × inv(a) = 1`.

```
inv(a) = a^(p-2) mod p       for a ≠ 0
inv(0) = ⊥_error             division by zero
```

this is Fermat's little theorem: in a prime field, a^(p-1) = 1 for all nonzero a, so a^(p-2) is the multiplicative inverse. the implementation uses square-and-multiply over the 64-bit exponent p-2, costing 64 field multiplications — making inv the most expensive field pattern.

inv enables division: `div(a, b) = mul(a, inv(b))`. division enables solving equations: given `a × x = b`, the solution is `x = mul(b, inv(a))`. solving equations enables the full power of algebraic reasoning — linear systems, polynomial root finding, Lagrange interpolation, all the machinery that [[stark]] proofs require.

inv is also the pattern that nox inherited instead of Nock's increment. where Nock builds all arithmetic from increment (a single +1 operation, making decrement O(n) and multiplication O(n²)), nox builds all arithmetic from field operations. the tradeoff: inv costs 64 multiplications, but add, sub, and mul cost 1 each. the total cost of arithmetic in nox is O(1) per operation. in Nock it is O(n) to O(n²). this is the price of proof-nativity — and the return is that every arithmetic operation generates a bounded number of [[stark]] constraints.

## comparisons: eq, lt

eq tests equality: returns 0 if two field elements are identical, 1 otherwise.

```
eq(a, b) = 0    if a == b
eq(a, b) = 1    if a ≠ b
```

lt tests ordering: returns 0 if a < b in the canonical integer representation.

```
lt(a, b) = 0    if a < b    (as integers in [0, p-1])
lt(a, b) = 1    if a ≥ b
```

the return convention (0 for true, 1 for false) matches [[Nock]] and integrates with branch: `branch(test, yes, no)` evaluates `yes` when test returns 0. true is zero. this is counterintuitive for programmers used to C conventions but consistent within the nox/Nock tradition.

together eq and lt provide the minimal comparison set. every predicate over field elements can be built from them:

- not-equal: the complement of eq (swap the branches)
- greater-than: `lt(b, a)` — swap the arguments
- less-or-equal: `branch(lt(a, b), yes, branch(eq(a, b), yes, no))`
- range check: composition of lt comparisons

lt is more expensive than arithmetic in the [[stark]] trace — ~64 constraints (bit decomposition to compare magnitudes). eq is cheap: 1 constraint (the verifier checks whether the difference is zero or has an inverse). lt requires reasoning about the integer representation of field elements, which means decomposing into bits.

## why field arithmetic matters

the [[stark]] proof system operates over a finite field. every constraint in the proof is a polynomial equation over F_p. if the VM's arithmetic is field arithmetic, the execution trace IS the proof witness:

```
program computes:     a + b mod p = c
STARK constraint:     a + b mod p = c
```

same operation. same field. zero translation. the program runs and the proof writes itself — the sequence of field operations during execution is exactly the algebraic intermediate representation (AIR) that the prover proves and the verifier checks. there is no "circuit compilation" step. there is no "arithmetization" pass. the computation is already arithmetic.

this is the fundamental insight that separates nox from every other VM. conventional zkVMs (risc0, SP1, Valida) execute programs in one algebra (integers, bytes, registers) and then translate the execution into field constraints. that translation is where complexity, bugs, and performance loss live. nox eliminates the translation by making the execution algebra identical to the proof algebra.

## the cost model

```
pattern    execution cost    STARK constraints    notes
add        O(1)              1                    single field equation
sub        O(1)              1                    single field equation
mul        O(1)              1                    single field equation
inv        O(64)             1                    verifier checks a × a⁻¹ = 1
eq         O(1)              1                    difference is zero or has inverse
lt         O(1)              ~64                  bit decomposition for comparison
```

the asymmetry between execution cost and verification cost is fundamental. inv costs 64 multiplications to compute (square-and-multiply) but only 1 constraint to verify (check a × a⁻¹ = 1). the verifier does not repeat the computation — it checks the result. add, sub, mul each generate exactly 1 constraint. eq generates 1 constraint (checking whether the difference is zero). lt requires ~64 constraints (bit decomposition to compare magnitudes in the integer representation).

contrast this with Nock, which has only increment as its arithmetic primitive. decrement must be built by counting up from 0 to n-1 — an O(n) loop. addition is iterated increment: O(a+b). multiplication is iterated addition: O(a×b). multiplying two 64-bit numbers takes O(2^64) steps. in a proof system, each step is a constraint, so Nock arithmetic would produce astronomically large proofs. the field patterns collapse all of this: O(1) execution, O(1) constraints, for every arithmetic operation.

## why six and no others

{+, -, ×, ÷} is the complete set of field operations. any algebraic expression over F_p decomposes into additions, subtractions, multiplications, and inversions. exponentiation is repeated multiplication. division is multiplication by inverse. square roots are exponentiations (Tonelli-Shanks over F_p). there is no field operation that escapes these four.

{=, <} is the minimal set of comparisons. equality handles exact matching. ordering handles range checks, bounds verification, and sorting. greater-than is `lt` with swapped arguments. not-equal is the complement of eq. between them, every comparison predicate is expressible.

a seventh field pattern would be redundant. a fifth arithmetic operation would decompose into the existing four. a third comparison would be syntactic sugar for a composition of eq and lt. six is the exact count: four for algebra, two for comparison, nothing wasted.
