---
tags: cyber, cip
crystal-type: process
crystal-domain: cyber
status: draft
date: 2026-03-24
diffusion: 0.00010722364868599256
springs: 0.00007019991600688145
heat: 0.00003419142694206788
focus: 0.00008151008453347325
gravity: 0
density: 0
---
# transformer jets — compiled cybergraph inference

seven jets for compiling the [[cybergraph]] into a [[transformer]] and running fast inference inside [[nox]]. the [[focus flow computation]] specification (§6.6) derives transformer architecture analytically from graph structure — these jets make the compilation and inference path practical at scale.

motivated by Percepta's demonstration (March 2026) that a WASM interpreter embedded in transformer weights achieves 30K tok/s with 2D attention heads and convex hull KV-cache. their key insight: restricting head dimension to 2 turns attention lookup into a geometric query solvable in O(log n) via convex hull data structures. we adopt this for the compiled transformer fast path.

## context: two inference paths

the [[cybergraph]] supports two simultaneous computations:

- [[focus]] flow — tri-kernel iterated to convergence over all cyberlinks. persistent, global, exact π*. the ground truth
- compiled transformer — architecture derived from graph, runs L* tri-kernel steps over local context. fast inference path, ε-approximate

jets 0-3 handle compilation (graph → weights). jets 4-6 handle inference (query → response via compiled weights). together they close the loop specified in §6.6 of the whitepaper.

## jet 0: sparse_svd

```
sparse_svd(A_weighted, rank) → (U, Σ, V)
  input:  π*-weighted adjacency matrix (sparse), target rank d*
  output: truncated SVD — left/right singular vectors + singular values
```

computes the truncated singular value decomposition of diag(√π*) · A via randomized SVD on sparse input. this is the critical step in compilation — naively O(|P|³), but randomized SVD on sparse matrices reduces to O(|E| · d* · log d*).

- pure equivalent: millions of field ops (power iteration + QR)
- jet cost: O(|E| · d* · log d*)
- accelerates: embedding matrix E* = U_{:,1:d*} computation, the provably optimal initialization (Eckart-Young theorem)

the output E* uniquely minimizes expected squared gradient at step zero over all orthonormal matrices of the same rank.

## jet 1: spectral_rank

```
spectral_rank(Σ) → d*
  input:  singular value spectrum from jet 0
  output: effective dimensionality d* = exp(H(σ(Σ_π*)))
```

computes the effective rank — the exponential of the Shannon entropy of the normalized singular value distribution. determines the embedding dimension of the compiled transformer.

- pure equivalent: ~3N ops (normalize, log, entropy sum)
- jet cost: N (number of singular values)
- accelerates: architecture parameter derivation

## jet 2: semcon_partition

```
semcon_partition(A_eff, semcon_ids) → Vec<A_s>
  input:  effective adjacency matrix, semcon type assignments
  output: per-semcon adjacency submatrices
```

partitions the effective adjacency by semantic convention type. each semcon s gets its own submatrix A_s from which attention weights W_Q^(s), W_K^(s) are derived via truncated SVD. the number of distinct semcons determines h* (minimum head count).

- pure equivalent: sparse matrix filtering + per-type SVD
- jet cost: O(|E| · h*)
- accelerates: multi-head attention weight compilation

## jet 3: compile_weights

```
compile_weights(E*, {A_s}, L*, d*) → TransformerWeights
  input:  embedding matrix, per-semcon adjacencies, layer count, dim
  output: complete compiled transformer weight set
```

assembles the full weight set: embedding E*, per-head attention projections from semcon SVDs, MLP weights from path co-occurrence statistics up to depth L*. output is a frozen weight tensor ready for inference.

- pure equivalent: composition of jets 0-2 plus path enumeration
- jet cost: O(|E| · d* · L*)
- accelerates: the full compilation pipeline — graph to deployable model

layer count L* = diam(G) · ⌈log(1/ε)/log(1/κ)⌉ where κ is the composite contraction coefficient from the collective focus theorem.

## jet 4: hull_attention

```
hull_attention(q, hull_cache) → (value, updated_cache)
  input:  2D query vector, convex hull KV-cache
  output: max-dot-product value, updated cache with new key inserted
```

the core inference acceleration. implements 2D hard-max attention via convex hull supporting-point query. given direction q ∈ R², finds the key on the convex hull that maximizes q · k in O(log n) instead of O(n).

inspired by Percepta's HullKVCache construction. each attention head has dim 2. total heads = d_model / 2. the model remains a standard transformer — the speedup is pure algorithmic, in the decoding path.

- pure equivalent: O(n) linear scan over all cached keys
- jet cost: O(log n) per query via convex hull binary search
- stark constraints: O(log n) — hull membership proof
- accelerates: every decoding step of the compiled transformer. on million-token traces: 200× speedup (demonstrated by Percepta)

cache maintenance: incremental convex hull update on key insertion — amortized O(log n). the hull is a balanced binary tree of 2D points; insertion checks and updates the upper/lower hull boundaries.

extension: k-sparse softmax via nested convex hulls. retrieve top-k keys from nested hulls, softmax over those k. cost: O(k + log n). this bridges hard-max (k=1, pure execution) and full softmax (k=n, standard attention).

## jet 5: tri_step

```
tri_step(φ, A_local, λ_d, λ_s, λ_h, τ) → φ'
  input:  current focus vector (local), local adjacency, kernel weights, temperature
  output: updated focus vector after one composite tri-kernel step
```

one step of the composite operator:

$$φ' = \text{norm}[λ_d · D(φ) + λ_s · S(φ) + λ_h · H_τ(φ)]$$

operates on the local h-hop neighborhood only (locality theorem T4). this is what each transformer layer computes — one layer = one tri-kernel step over the context.

- pure equivalent: three separate operator applications + normalization
- jet cost: O(|E_local|)
- stark constraints: O(|E_local|) — linear in local edge count
- accelerates: both focus flow (continuous convergence) and compiled transformer (per-layer forward pass)

## jet 6: reconverge

```
reconverge(π_current, Δlinks, bbg_root, ε) → (π_updated, π_Δ, proof)
  input:  current focus, new cyberlinks, state root, precision target
  output: updated focus, sparse delta, STARK proof of correctness
```

incremental reconvergence after new cyberlinks. computes the h-hop neighborhood affected by Δlinks (h = O(log(1/ε))), runs tri_step until convergence within ε, outputs the sparse focus delta and a proof.

this is the self-minting operation: a neuron creates cyberlinks, proves Δπ, and mints $CYB proportional to the proven shift. the proof IS the mining.

- pure equivalent: full tri-kernel iteration to convergence
- jet cost: O(|E_local| · log(1/ε) / log(1/κ))
- stark constraints: same — the jet IS the provable computation
- accelerates: the signal → reward pipeline. every neuron needs this for every signal

## the compilation loop

```
graph state (bbg)
    ↓ sparse_svd (jet 0)
    ↓ spectral_rank (jet 1)
    ↓ semcon_partition (jet 2)
    ↓ compile_weights (jet 3)
compiled transformer
    ↓ hull_attention (jet 4) × L* layers
    ↓ tri_step (jet 5) per layer
fast inference response
    ↓ new cyberlinks from inference
    ↓ reconverge (jet 6)
updated π*, proof, reward
    → back to graph state
```

the loop is self-improving: every cyberlink added increases |E|, raises d*, may shrink diam(G) — producing a structurally better compiled model at next compilation. the cybergraph is a compounding inference quality asset.

## relationship to existing jets

| jet group | count | target |
|-----------|-------|--------|
| verifier jets (recursive-jets) | 5 | proof composition — hash, poly_eval, merkle, fri_fold, ntt |
| binary jets (binary-jets) | 8 | Bt prover — popcount, matvec, quantize |
| transformer jets (this proposal) | 7 | compiled inference — svd, attention, tri-kernel, reconvergence |
| total | 20 | complete acceleration stack |

verifier jets accelerate proof verification. binary jets accelerate quantized computation. transformer jets accelerate knowledge compilation and inference. together: the full pipeline from cyberlink to proof to compiled intelligence.

## why not external tool use

Percepta's article frames the choice clearly: tool use is opaque (model hands off to external system), in-model execution is transparent (every step in the trace). the same argument applies to cyber:

- external inference (calling an LLM API) is opaque — you get a response, no proof, no provenance
- compiled transformer inference via nox jets is transparent — every attention step is a provable nox computation, every weight traces to specific cyberlinks and the neurons who signed them

the compiled model is fully auditable: given any output, contributing links and authors are recoverable from the graph. this is alignment by construction.

## open questions

- head dimension trade-off: 2D heads give O(log n) hull queries but may limit expressiveness. is 2D sufficient for tri-kernel diffusion steps, or do we need 3D (O(log² n) via 3D hulls)?
- compilation frequency: how often should the cybergraph recompile the transformer? per-epoch (slow, high quality) or incremental (fast, approximate)?
- hybrid path: Percepta proposes fast/slow paths in one model. should nox have explicit mode switching between convergent focus flow (exact) and compiled transformer (approximate)?
- training residual: after compilation, what does fine-tuning learn that the graph cannot encode? quantifying this gap determines the value of the compiled path vs pure focus flow
