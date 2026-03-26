# the triple

object, formula, budget — the three inputs to every nox computation. the first two come from [[Nock]]. the third is new, and it changes everything.

## reduce(object, formula, budget) → result

every nox reduction takes exactly three inputs.

object is what the program knows — the environment, the data, the context. in a function call, the object is the argument. in a contract execution, the object is the current state. in a [[cyberlink]] evaluation, the object is the graph neighborhood.

formula is what the program does — the code, the instructions, the transformation. a formula is a noun of the form `(tag . body)` where tag selects one of seventeen patterns. the formula transforms the object into a result.

budget is what the program costs — a resource counter. every pattern deducts from budget. when budget reaches zero, computation halts. the protocol decides what token denominates the budget.

## object-formula duality

in nox, programs are nouns. data is nouns. the distinction between code and data is purely contextual — the same noun can be an object in one reduction and a formula in another.

```
reduce(object, formula, budget)

object = noun     (the data)
formula = noun     (the code)
result  = noun     (the output)
```

this duality is structural, not semantic. the VM does not "know" which noun is code and which is data. it takes any noun as an object, any noun as a formula, and attempts to reduce. if the formula has a valid pattern tag and the operands match, reduction proceeds. if not, it produces ⊥_error.

the consequence: self-modifying programs, interpreters, compilers, and proof verifiers are all ordinary nox computations. a nox program that takes another nox program as its object and executes it is just pattern 2 (compose) — evaluate the formula-noun to get a new formula, then apply it to the object. there is no special "eval" mechanism. the universality is built in.

## budget: the third element

[[Nock]] has object and formula. nox adds budget. this addition is what makes nox suitable for a decentralized network where computation must be bounded and priced.

without budget, a formula can loop forever. `[2 [[1 [2 [[0 1] [0 1]]]] [1 [2 [[0 1] [0 1]]]]]]` — compose applied to a self-referencing formula — never terminates. in a single-machine system, you kill the process. in a decentralized network, who decides when to stop? budget solves this: every pattern costs, the budget is finite, termination is guaranteed.

```
reduce(s, [5 [a b]], f) =
  if f < 1 then Halt
  let (v_a, f1) = reduce(s, a, f - 1)
  let (v_b, f2) = reduce(s, b, f1)
  ((v_a + v_b) mod p, f2)
```

budget is not gas (Ethereum). gas is an economic mechanism bolted onto a virtual machine that was designed without it. budget is a semantic parameter of reduction — it appears in the type signature, it affects the result (Halt vs value), it is part of the computation's identity.

## focus as attention

in [[cyber]], focus is the same resource that weights [[cyberlinks]] in the [[cybergraph]]. a [[neuron]] has a focus budget. it spends focus to think (run nox programs) and to speak (create [[cyberlinks]]). the budget is unified — attention and computation are the same currency.

this is the resource theory of nox. a neuron that runs an expensive computation pays the same focus it would spend creating thousands of cyberlinks. the network does not distinguish between "processing" and "communicating" — both consume the same resource, both are metered by the same mechanism, both contribute to the same focus-weighted graph.

the budget also determines [[cyberank]] influence. a neuron's links are weighted by the focus spent on them. a neuron that exhausts its focus on computation has less influence in the knowledge graph. a neuron that prioritizes linking has less computation available. the tradeoff is fundamental — it forces neurons to allocate attention between thinking and speaking, between private computation and public knowledge.

## the triple and the stark

the [[stark]] proof covers the entire triple. the proof says: "this formula was applied to this object under this focus budget, and the result was this noun with this remaining focus." the verifier checks:

```
(H(object), H(formula), focus_initial) → (H(result), focus_remaining)
```

focus appears in the proof. a computation that halts (focus exhausted) has a different proof than one that completes. the prover cannot lie about the budget — the trace records every focus decrement, and the [[stark]] verifier checks them all.

this means focus is publicly auditable. when a [[neuron]] claims to have spent focus on a computation, the proof demonstrates exactly how much was consumed. the network can verify that the neuron's focus allocation matches its claims. no trust required — the math checks.

## why not two inputs?

many VMs separate code and data — the program counter reads instructions from one region while operating on data in another. why does nox combine them into one space (object) and control them with one pointer (formula)?

because content-addressing requires it. a computation's canonical identity is `(H(object), H(formula))`. if the "code" and "data" lived in separate namespaces, you would need separate hashes, separate identity schemes, separate caches. by making everything nouns in the same space, the identity collapses to a single pair of hashes. the computation cache is one table. the content-addressing scheme is one function.

and because homoiconicity requires it. when code is data, a program can construct and examine other programs. a compiler transforms a source noun into a target noun. a proof verifier takes a proof noun and checks it. these are ordinary computations — same triple, same reduction, same caching. the uniformity is the power.
