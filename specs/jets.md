# jet specification

version: 0.4
status: canonical

## overview

jets are compositions of Layer 1 patterns recognized by formula hash and replaced with optimized implementations. every jet has an equivalent pure Layer 1 program producing identical output on all inputs. jets are OPTIMIZATION — semantics unchanged.

the jet mechanism is algebra-polymorphic. jet formula hashes are computed from the abstract pattern trees. the optimized implementation is per-instantiation — the same jet dispatches to different backends depending on the algebra (software, GFP hardware, or delegated to a specialized prover).

## semantic contract

every jet MUST have an equivalent pure Layer 1 expression producing identical output on all inputs.

this is testable: a harness compares jet output against pure-pattern output on random inputs. if a jet is removed, the system remains correct — only slower. jets are never load-bearing for correctness, only for performance.

## jet recognition

jets are recognized by formula hash. the VM maintains a jet registry mapping formula identities to optimized implementations.

```
jet_registry: H(formula_noun) → jet_implementation

recognition: at reduction time, before dispatching a formula,
the VM checks if H(formula) is in the jet registry.
if yes: execute the jet (same result, fewer trace rows).
if no:  dispatch normally via Layer 1 patterns.
```

the jet registry lives on the cybergraph. genesis jets are committed in the genesis BBG state — frozen by axiom A3 (append-only). every conforming implementation MUST recognize genesis jets.

genesis jet entries are generated, not hand-written: the build system constructs each jet's pure Layer 1 formula as a noun, computes its structural hash, and commits them as particles in the genesis state.

post-genesis jet registration through cyberlinks is a separate mechanism — design deferred. the semantic contract (pure Layer 1 equivalent) applies to all jets regardless of origin.

## per-instantiation dispatch

jet implementations are per-instantiation. the same formula hash dispatches to:
- software implementation (any instantiation)
- GFP hardware primitives (nox<Goldilocks> on GFP-equipped hardware)
- delegated prover (cross-algebra boundary jets)

each nox instantiation defines its own jet registry. formula hashes may differ across algebras if the same source operation compiles to different pattern trees.

## adding a new genesis jet

genesis jets are frozen at launch. the process before genesis:

1. **write the pure Layer 1 formula** — semantics defined by this formula, patterns 0-17 only
2. **compute the formula hash** — build system does this automatically
3. **implement the optimized version** — software, constraint (CCS), hardware (GFP)
4. **commit to genesis state** — jet particle in genesis BBG state
5. **write the test harness** — `∀ inputs: jet(input) == pure_formula(input)`
6. **document** — create a spec page in jets/ with the standard format

```
jet entry format:
  name, formula_hash, input, output, pure_cost, jet_cost,
  constraints, accelerates, hardware_mapping
```

## genesis jet registry — five algebras

individual jet specifications live in `jets/` (within specs/). see [[jets/README|jet registry index]] for the full table.

| group | algebra | jets | PCS | spec |
|-------|---------|------|-----|------|
| hash | all (Hemera anchor) | 1 | — | [[jets/hash]] |
| recursion | F_p | 4 | Brakedown | [[jets/recursion]] |
| binary-tower | F₂ | 8 | Binius | [[jets/binary-tower]] |
| polynomial-ring | R_q | 5 | Ikat | [[jets/polynomial-ring]] |
| isogeny-curves | F_q | 5 | Porphyry | [[jets/isogeny-curves]] |
| tropical-semiring | (min,+) | 6 | Assayer | [[jets/tropical-semiring]] |
| state | F_p | 6 (1 exact + 5 templates) | Brakedown | [[jets/state]] |
| decider | F_p | 1 | Brakedown | [[jets/decider]] |

boundary jets (quantize, dequantize, gadget_decompose, secret_hash, witness_commit) live within their parent algebra — not a separate group.

total genesis: 1 hash (universal) + 4 recursion + 8 binary-tower + 5 polynomial-ring + 5 isogeny-curves + 6 tropical-semiring + state jets (unlimited) + 1 decider. all committed in genesis BBG state.

## hardware mapping

```
GFP primitive                    jets it accelerates                              algebras
────────────────────────────     ─────────────────────────────────────             ────────
fma (field multiply-accumulate)  poly_eval, key_switch, noise_track, group_action  nebu, jali, genies
ntt (NTT butterfly)              ntt, ntt_batch, blind_rotate                      nebu, jali
p2r (Poseidon2 round)            hash, merkle_verify                               nebu
lut (lookup table)               activation_lut, trop comparisons                  kuro, trop
```

the stack is continuous: nox pattern → software jet → GFP hardware primitive. the same computation, three speeds, identical semantics at every level. all five algebras map to four GFP primitives.

## self-verification

the stark verifier for nox is itself a nox program. with Brakedown (Merkle-free PCS), the verifier is pure field arithmetic — ~825 constraints (CCS jet + batch) or ~89 (algebraic Fiat-Shamir). recursion to arbitrary depth, constant proof size at every level. the decider jet reduces all-history verification to 89 constraints. see zheng/specs/verifier.md for canonical costs.

## domain-specific jets

language operations recognized by the same formula-hash mechanism. open-ended — any frequently-used nox composition can become a jet:

```
language operation       nox composition              jet           GFP hardware
─────────────────────    ──────────────────────────   ──────────    ────────────
Arc: rank(g, steps)      iterated add/mul loops       matmul jet    fma array
Wav: fft(x)              butterfly add/mul network    ntt jet       ntt engine
Any: hash(x)             Poseidon2 field ops          hash jet      p2r pipeline
Ten: activation(x)       table lookup composition     lookup jet    lut engine
Ren: geometric_product   mul/add over components      geo_mul jet   fma array
Wav: polynomial_mul      NTT + pointwise + iNTT       ntt jet       ntt engine
```

domain-specific jets follow the same semantic contract. the genesis jet registry is per-instantiation. post-genesis jets inherit the same contract — design deferred.
