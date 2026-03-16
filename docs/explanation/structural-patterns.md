# structural patterns

the five patterns that make nox Turing-complete — tree algebra inherited from [[Nock]] and the combinatory logic tradition.

```
0  axis    — navigate a noun tree
1  quote   — return a literal
2  compose — chain two computations (recursion)
3  cons    — build a cell (data construction)
4  branch  — conditional evaluation
```

these five patterns are the computational core. every other group — field, bitwise, hash — is an extension built on top of them. the structural patterns operate in tree space. they do not know what a field element is. they do not know what a hash is. they manipulate pure structure: positions in a tree, pairs of values, paths through branches. this is the universal layer — it works regardless of what atoms contain.

## axis: reading

axis navigates the object tree by numeric address. the address is a natural number that encodes a path from root to leaf: 1 is the root, 2 is the left child, 3 is the right child, 4 is left-left, 5 is left-right, and so on. the binary representation of the axis number IS the path — after the leading 1 bit, each 0 means "go left" and each 1 means "go right."

```
         1
        / \
       2   3
      / \ / \
     4  5 6  7
```

this is the only way to read data from the object. there are no variables, no names, no symbol tables. a program that needs "the third element of a list" computes the axis address and navigates there. the address is a value — it can be computed, stored, passed as an argument. data access is arithmetic on positions.

this design eliminates an entire class of bugs. there is no variable shadowing (no variables). there is no scope confusion (no scopes). there is no capture problem (no binding). the address either points to a valid position in the tree or it does not — and if it does not, the result is a type error (⊥_error), not silent corruption.

axis also serves a second role: `axis(object, 0)` returns the cryptographic hash of the object. axis zero is identity — the noun's fingerprint. this is how content-addressing enters at the structural level: any noun can know its own hash without invoking the hash pattern explicitly.

## quote: constants

quote returns its argument unchanged, without evaluating it. `[1 x]` produces `x` regardless of what the object is. this is how literal values enter a computation — numbers, code templates, static data structures.

quote is the dual of axis. axis reads from the environment (the object). quote ignores the environment entirely. between them, a nox program can reference any existing value (axis) or introduce any new value (quote). these two patterns — read and literal — are the only sources of data in a computation. everything else transforms data that axis and quote provide.

quote is also the mechanism for code generation. since code is data (nouns are nouns), quoting a formula produces a formula-as-data that can be stored, transmitted, and later evaluated by compose. this is homoiconicity in action: the program can construct programs, and quote is the boundary between "this is code to run" and "this is data to keep."

## compose: recursion

compose is the engine of all computation. `[2 a b]` evaluates `a` against the object to get a new object, evaluates `b` against the object to get a formula, then applies the formula to the new object. this is function application — the universal mechanism for loops, recursion, subroutine calls, and every form of control flow.

```
compose(object, [2 a b]):
  1. new_object = reduce(object, a)
  2. formula     = reduce(object, b)
  3. result      = reduce(new_object, formula)
```

compose is why five patterns suffice for Turing completeness. a formula can compose itself — evaluating a formula that produces another formula application, which produces another, indefinitely. this is recursion without a recursion primitive. the pattern does not know it is recursing; it simply evaluates a formula against an object, and if the result is another computation, it evaluates that too.

every higher-level construct — while loops, map/fold, pattern matching, function dispatch — compiles down to compositions. compose is the sole source of computational depth in nox. without it, every computation would be flat: a single pass over the object, producing a result in bounded time. compose adds unbounded depth, which is exactly what separates Turing-complete systems from finite automata.

the [[focus]] parameter prevents unbounded computation from becoming unbounded cost. each compose step consumes focus. when focus reaches zero, computation halts with a Halt result — not an error, but a clean resource boundary. compose provides the depth; focus provides the limit.

## cons: construction

cons builds a cell from two evaluated subexpressions. `[3 a b]` evaluates both `a` and `b` against the object and pairs the results into `[result_a result_b]`. this is data construction — the only way to build new compound values.

```
cons(object, [3 a b]):
  left  = reduce(object, a)
  right = reduce(object, b)
  result = [left right]
```

cons is inherently parallel. its two subexpressions are independent — neither depends on the other's result. a parallel evaluator can dispatch both branches to separate threads. this is not an optimization hint; it is a mathematical fact. the two subexpressions share the same object but cannot affect each other's evaluation. [[confluence]] guarantees that evaluating them in either order, or simultaneously, produces identical results.

cons is also the mechanism for returning multiple values. a function that needs to return three things wraps them in nested cons: `[3 [3 a b] c]` produces `[[result_a result_b] result_c]`. there is no tuple type, no record type, no struct — just nested pairs. every compound data structure in nox is built from cons: lists are right-nested pairs, trees are balanced pairs, records are positional pairs. one construction primitive serves all.

## branch: decision

branch is conditional evaluation. `[4 test yes no]` evaluates `test` against the object. if the result is 0, it evaluates `yes`. if the result is 1, it evaluates `no`. any other result is an error.

```
branch(object, [4 test yes no]):
  condition = reduce(object, test)
  if condition == 0: reduce(object, yes)
  if condition == 1: reduce(object, no)
  otherwise: ⊥_error
```

branch is the only pattern that discards computation. one of its two branches is never evaluated — the formula for the path not taken is syntactically present but semantically absent. this is how nox avoids wasted [[focus]]: the branch not chosen costs nothing.

the binary choice (0 or 1, nothing else) is deliberate. multi-way dispatch is built by nesting branches. this keeps the pattern semantics minimal — one test, two paths — and the [[stark]] constraint simple: verify the condition is 0 or 1, verify the result matches the chosen branch. a multi-way branch would require variable-length constraint patterns, complicating the proof system for marginal convenience.

## why five is enough

axis provides input. quote provides constants. cons provides output (construction). branch provides choice. compose provides depth (recursion). these five operations span the space of computable functions: read data, create data, combine data, choose between alternatives, and repeat.

```
axis    → input          (read from environment)
quote   → constants      (introduce new values)
cons    → construction   (build compound values)
branch  → decision       (choose between paths)
compose → depth          (recursion, loops, function calls)
```

Church and Turing proved in 1936 that this is sufficient — any function computable by any mechanism whatsoever can be expressed as a combination of these primitives. the structural patterns are inherited from [[Nock]], which inherited them from the combinatory logic tradition. S and K combinators (1924) proved two operations suffice for universality. lambda calculus (1936) proved one operation (application with binding) suffices. Nock showed that tree navigation and construction suffice. nox preserves this core unchanged — because it is already minimal.

the structural patterns produce one [[stark]] constraint each (or zero for quote, which is resolved at parse time). they are the cheapest patterns in the system — pure tree manipulation with no arithmetic overhead. this makes sense: structure is free, arithmetic costs.
