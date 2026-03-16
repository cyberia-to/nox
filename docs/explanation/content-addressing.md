# content-addressed computation

every computation has a name — the seed of planetary memoization.

## the identity

because Layer 1 is confluent and programs are nouns, every computation has a canonical, permanent, universal identity:

```
key:   (H(subject), H(formula))
value: H(result)
```

the hash of what the program knows. the hash of what the program does. together they determine the hash of the output. this is a mathematical fact — not a caching strategy, not an optimization, not a convention. confluence guarantees it. any machine, anywhere, at any time, reducing the same subject with the same formula, will produce the same result.

## from function to fact

a traditional function call is ephemeral. you call `f(x)`, get the result, and it evaporates unless you explicitly store it. the computation exists for the duration of its execution and then disappears.

a content-addressed computation is a fact. once anyone has computed `(H(subject), H(formula)) → H(result)`, that relationship is permanently established. it is true now, true in a year, true on every machine. the fact can be recorded, shared, verified, and relied upon — just like a mathematical theorem.

this transforms computation from an event (something that happens) into a datum (something that exists). the result is not "the output of running this program" — it is "the unique value associated with this pair of hashes." the verb becomes a noun.

## the planetary cache

the network builds a shared cache where every node contributes:

```
node A computes: (H(s₁), H(f₁)) → H(r₁)
node B computes: (H(s₂), H(f₂)) → H(r₂)
node C looks up: (H(s₁), H(f₁)) → finds H(r₁) in cache
```

node C never runs the computation. it retrieves the cached result and verifies it against the [[stark]] proof (or re-computes to check). the cache is:

- universal: any node can contribute and consume, across network boundaries
- permanent: entries never change (confluence guarantees determinism)
- verifiable: result hashes are checkable against proofs or re-computation
- composable: the result of one computation can be the subject of another

as more nodes compute more programs, the cache grows. common computations — identity verification, link validation, rank updates, proof verification — are computed once and cached forever. the network converges toward a state where routine operations are memory lookups rather than recomputations.

## what gets cached, what does not

```
Layer 1: fully memoizable (deterministic)
Layer 2: NOT memoizable (hint results are prover-specific)
Layer 3: fully memoizable (jets are deterministic)
```

pure Layer 1 computations are the ideal cache citizens. their results are determined entirely by their inputs. cache them once, use them forever.

hint-containing computations (Layer 2) are excluded. different provers may inject different valid witnesses, producing different results for the same (subject, formula) pair. caching would be unsound — the cached result might not match what this particular prover would produce.

the boundary is precise: a computation is hint-tainted if it transitively contains a hint application. pure sub-expressions within a tainted computation remain cacheable. the taint applies to the root, not to the leaves. this maximizes caching without compromising soundness.

## content-addressing in the cybergraph

a [[cyberlink]] is a nox computation. its identity is `(H(from_particle), H(to_particle))` — the hash of the source and the hash of the target. the cyberlink's evaluation (ranking, validation, inference) is a nox reduction. the result is content-addressed.

this means the [[cybergraph]] is not a mutable database that must be synchronized. it is a deterministic function of its inputs. two nodes that independently evaluate the same cyberlinks produce the same results. agreement is not negotiated — it is computed.

the computation cache is the mechanism by which the cybergraph scales. as the graph grows, the fraction of computations that are cache hits increases. rank updates for stable regions of the graph are cached. identity verifications for known neurons are cached. link validations for established connections are cached. the network's computational load approaches a steady state where most operations are lookups.

## the hash chain

content-addressed computation is composable. the result of one computation becomes the subject of another:

```
step 1: (H(genesis_state), H(transition_1)) → H(state_1)
step 2: (H(state_1), H(transition_2)) → H(state_2)
step 3: (H(state_2), H(transition_3)) → H(state_3)
```

each step is independently cacheable. each step's result is verifiable. the chain of hashes is a complete, auditable history of the computation. this is how [[bbg]] (the state engine) works: the blockchain state is a sequence of content-addressed transitions, each provable, each cacheable.

## the fixed point

the planetary computation cache is the convergence point of several ideas:

- confluence guarantees that results are evaluation-order-independent
- nouns provide a universal data structure with canonical serialization
- [[Hemera]] provides a collision-resistant hash
- [[starks]] provide compact, verifiable proofs

together they create a system where computing something and proving you computed it are nearly the same cost — and where the result, once computed, is a permanent, shared, universal fact.

the cache is the seed of planetary intelligence. as more agents compute, more facts are established. as more facts accumulate, more computations become cache hits. the system accelerates itself — each computation makes future computations cheaper. this is the economic foundation of scalable collective intelligence: knowledge, once produced, persists and compounds.
