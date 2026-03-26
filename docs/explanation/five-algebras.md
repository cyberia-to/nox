# five algebras

nox is one VM. the sixteen patterns never change. but computation over a planetary knowledge graph touches five fundamentally different algebraic structures — each with workloads where the others are structurally inefficient.

## why five

start from what a superintelligence must do:

| capability | algebra | why irreducible |
|-----------|---------|-----------------|
| prove truth | nebu (F_p) | universal reduction target — every proof lands here |
| execute efficiently | kuro (F₂) | binary operations 32× cheaper native than through F_p |
| compute on encrypted data | jali (R_q) | ring operations 3072× cheaper native than scalar decomposition |
| optimize decisions | trop (min,+) | semiring with no inverse — irreducible to field arithmetic |
| protect identity | genies (F_q) | commutative group action — no construction over Goldilocks |

remove any one and the system is incomplete. a correct system without encryption leaks data. an encrypted system without optimization makes bad decisions. an optimal system without identity protection exposes who decided what.

## one VM, five algebras

nox doesn't have five VMs. it has one VM parameterized by algebra:

```
nox<F, W, H>

F = field        (nebu, kuro, jali, genies — determines patterns 5-10)
W = word width   (determines patterns 11-14)
H = hash         (determines pattern 15)
```

the five structural patterns (axis, quote, compose, cons, branch) are identical across all instantiations. they operate on tree structure, not field elements. the computational patterns (add, mul, xor, hash) dispatch to the appropriate algebra.

## how non-field algebras enter

trop is a semiring, not a field. R_q is a ring, not F_p. how do they work in a field-parameterized VM?

**trop**: tropical operations decompose to existing patterns. min(a,b) = branch(lt(a,b), a, b). tropical jets accelerate common compositions (shortest path, assignment) but the underlying patterns are field operations. the proof happens in F_p — the tropical computation is the WITNESS, not the proof.

**jali**: R_q = F_p[x]/(x^n+1) decomposes via NTT into n copies of F_p. nox runs F_p operations. ring jets (ntt_batch, key_switch, blind_rotate) recognize structured compositions of F_p operations and commit them as ring-aware batches in zheng.

**genies**: F_q is a different prime field. nox<F_q> is a separate instantiation with its own jet registry. cross-algebra composition via HyperNova folds F_q sub-traces into the F_p accumulator.

## jets per algebra

each algebra contributes jets that accelerate its dominant workloads:

| algebra | jets | what they target |
|---------|------|-----------------|
| nebu | hash, poly_eval, merkle_verify, fri_fold, ntt | recursive proof verification |
| kuro | popcount, binary_matvec, quantize, activation_lut, gadget_decompose, ... | quantized inference, SpMV |
| jali | ntt_batch, key_switch, noise_track, blind_rotate | FHE bootstrapping |
| genies | group_action, isogeny_walk, vrf_eval, vdf_step | privacy primitives |
| trop | trop_matmul, trop_shortest, trop_hungarian, trop_viterbi, trop_transport | optimization witness generation |

33 jets total. remove them all: identical results, orders of magnitude slower.

## cross-algebra composition

a single nox program can mix algebras. FHE bootstrapping crosses three:

```
step 1: gadget decomposition    → kuro (F₂)
step 2: blind rotation          → jali (R_q)
step 3: key switching           → nebu (F_p)
step 4: modulus switching       → nebu (F_p)
```

each step proves via its native lens in zheng. HyperNova folds all sub-traces into one F_p accumulator. one decider, one proof. boundary cost: ~766 F_p constraints per algebra crossing.

## the decider: 89 constraints

the universal accumulator (~200 bytes) folds all history from genesis. the decider jet verifies it in 89 constraints — less than one hemera permutation (736). three optimizations stack: CCS jet encoding (4×), batched spot-checks via sumcheck (36×), algebraic Fiat-Shamir (zero hemera). verifying all chain history costs less than hashing 56 bytes.

## the correspondence

five algebras, five lenses in zheng, five jet families in nox, four GFP hardware primitives:

```
algebra → jets → lens → hardware
nebu    → 5    → Brakedown     → fma, ntt, p2r, lut
kuro    → 8    → Binius        → lut (SIMD packed)
jali    → 5    → Ring-aware    → fma, ntt
trop    → 5    → Tropical      → lut (comparisons)
genies  → 4    → Isogeny       → fma
```

the stack is continuous at every level. pattern → jet → lens → silicon. identical semantics, increasing speed. this continuity is the design invariant.
