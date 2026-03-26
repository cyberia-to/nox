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

composite jets for compiling the [[cybergraph]] into a [[transformer]] and running fast inference inside [[nox]]. the [[focus flow computation]] specification (§6.6) derives transformer architecture analytically from graph structure — these jets make the compilation and inference path practical at scale.

motivated by Percepta's demonstration (March 2026) that a WASM interpreter embedded in transformer weights achieves 30K tok/s with 2D attention heads and convex hull KV-cache. their key insight: restricting head dimension to 2 turns attention lookup into a geometric query solvable in O(log n) via convex hull data structures. we adopt this for the compiled transformer fast path.

## key finding: no new languages needed

all seven operations decompose into existing [[cyb/languages]]:

| jet | operation | primary lang | secondary | proof path |
|-----|-----------|-------------|-----------|------------|
| 0 sparse_svd | π*-weighted truncated SVD | [[Ten]] (contraction) | [[Arc]] (adjacency) | Ten → Tri |
| 1 spectral_rank | effective dimensionality d* | [[Bel]] (entropy on Δⁿ) | [[Ten]] | Bel → research / Ten → Tri |
| 2 semcon_partition | subgraph extraction by edge type | [[Arc]] (subcategory) | — | Arc → Tri |
| 3 compile_weights | assemble transformer weights | [[Ten]] | [[Arc]] | composition of jets 0-2 |
| 4 hull_attention | 2D convex hull max-dot query | [[Ren]] (G(2,0,0)) | — | Ren → Tri |
| 5 tri_step | composite D+S+H operator | [[Ten]] (SpMV) | [[Arc]], [[Bel]] | Ten → Tri |
| 6 reconverge | incremental Δπ + STARK proof | [[Tok]] (conservation) | [[Tri]] (proof) | Tok → stark |

these are composite jets — compositions of existing language primitives recognized by formula hash and accelerated. they introduce no new algebraic domain. the one genuinely new primitive is hull_attention, which belongs to [[Ren]] (2D Euclidean geometric algebra).

## decomposition into language primitives

```
sparse_svd     = Ten(matmul, transpose) ∘ Arc(π*_weighted_adjacency)
spectral_rank  = Bel(shannon_entropy) ∘ Ten(normalize_spectrum)
semcon_partition = Arc(filter_edges_by_morphism_type)
compile_weights = Ten(assemble) ∘ sparse_svd ∘ semcon_partition
hull_attention  = Ren(convex_hull_supporting_point)     ← one new Ren op
tri_step       = Ten(spmv) × 3 + Ten(simplex_project)  ← existing "matmul jet → fma"
reconverge     = tri_step^k + Tok(verify_conservation) + Tri(stark_prove)
```

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

language decomposition: Arc extracts the π*-weighted adjacency (sparse graph → sparse matrix). Ten performs randomized SVD via iterated matrix-vector products. the same "matmul jet → fma" GFP primitive that already handles Arc:rank(g, steps).

- pure equivalent: millions of field ops (power iteration + QR)
- jet cost: O(|E| · d* · log d*)
- accelerates: embedding matrix E* = U_{:,1:d*} computation, the provably optimal initialization (Eckart-Young theorem)

## jet 1: spectral_rank

```
spectral_rank(Σ) → d*
  input:  singular value spectrum from jet 0
  output: effective dimensionality d* = exp(H(σ(Σ_π*)))
```

language decomposition: Ten normalizes the spectrum. Bel computes Shannon entropy H(σ) — the information-geometric measure of how many independent dimensions the graph spans. this is Bel's native domain: entropy on the probability simplex.

- pure equivalent: ~3N ops (normalize, log, entropy sum)
- jet cost: N (number of singular values)

## jet 2: semcon_partition

```
semcon_partition(A_eff, semcon_ids) → Vec<A_s>
  input:  effective adjacency matrix, semcon type assignments
  output: per-semcon adjacency submatrices
```

language decomposition: pure Arc — extract subcategories of the cybergraph by morphism type (semcon). each semcon s defines a subgraph from which attention weights W_Q^(s), W_K^(s) are derived. the number of distinct semcons determines h* (minimum head count).

- jet cost: O(|E| · h*)

## jet 3: compile_weights

```
compile_weights(E*, {A_s}, L*, d*) → TransformerWeights
  input:  embedding matrix, per-semcon adjacencies, layer count, dim
  output: complete compiled transformer weight set
```

composition of jets 0-2 plus Ten path co-occurrence statistics up to depth L*. layer count L* = diam(G) · ⌈log(1/ε)/log(1/κ)⌉ from the collective focus theorem.

- jet cost: O(|E| · d* · L*)

## jet 4: hull_attention — the one new primitive

```
hull_attention(q, hull_cache) → (value, updated_cache)
  input:  2D query vector, convex hull KV-cache
  output: max-dot-product value, updated cache with new key inserted
```

the core inference acceleration and the only genuinely new primitive. implements hard-max attention via convex hull supporting-point query in Ren's domain: G(2,0,0) — 2D Euclidean geometric algebra.

given direction q ∈ R², find the key on the convex hull that maximizes q · k. this is a supporting hyperplane query — a standard operation in computational geometry, native to Ren's Clifford algebra.

- pure equivalent: O(n) linear scan over all cached keys
- jet cost: O(log n) per query via convex hull binary search
- stark constraints: O(log n) — hull membership proof
- GFP primitive: fma (same as Ren:geometric_product)
- accelerates: every decoding step of the compiled transformer. on million-token traces: 200× speedup (demonstrated by Percepta)

cache maintenance: incremental convex hull update on key insertion — amortized O(log n).

extension: k-sparse softmax via nested convex hulls. retrieve top-k keys, softmax over those k. cost: O(k + log n). this bridges hard-max (k=1, pure execution) and full softmax (k=n, standard attention).

why 2D is sufficient: any 1D lookup (retrieve value at index i) can be encoded as a 2D max-dot-product query. keys k_j = (2j, -j²), query q = (i, 1): the unique maximizer is j = i. this embeds integer indexing into 2D geometry — Ren's domain. higher-dimensional heads (3D hulls) give O(log² n) but may be unnecessary.

## jet 5: tri_step

```
tri_step(φ, A_local, λ_d, λ_s, λ_h, τ) → φ'
  input:  current focus vector (local), local adjacency, kernel weights, temperature
  output: updated focus vector after one composite tri-kernel step
```

language decomposition: three Ten sparse matrix-vector products (diffusion, springs, heat) + Ten simplex projection. this is the existing "Arc: rank(g, steps) → matmul jet → fma" extended to the full tri-kernel composite.

$$φ' = \text{norm}[λ_d · D(φ) + λ_s · S(φ) + λ_h · H_τ(φ)]$$

operates on the local h-hop neighborhood only (locality theorem T4).

- jet cost: O(|E_local|)
- stark constraints: O(|E_local|)

## jet 6: reconverge

```
reconverge(π_current, Δlinks, bbg_root, ε) → (π_updated, π_Δ, proof)
  input:  current focus, new cyberlinks, state root, precision target
  output: updated focus, sparse delta, STARK proof of correctness
```

language decomposition: tri_step^k (Ten) until convergence + Tok conservation verification (Σπ = 1) + Tri STARK proof generation. this is the self-minting operation: a neuron creates cyberlinks, proves Δπ, and mints $CYB proportional to the proven shift.

- jet cost: O(|E_local| · log(1/ε) / log(1/κ))
- the proof IS the mining

## the compilation loop

```
graph state (bbg)
    ↓ sparse_svd (jet 0: Ten ∘ Arc)
    ↓ spectral_rank (jet 1: Bel ∘ Ten)
    ↓ semcon_partition (jet 2: Arc)
    ↓ compile_weights (jet 3: Ten ∘ Arc)
compiled transformer
    ↓ hull_attention (jet 4: Ren)  × L* layers
    ↓ tri_step (jet 5: Ten × 3)   per layer
fast inference response
    ↓ new cyberlinks from inference
    ↓ reconverge (jet 6: Ten + Tok + Tri)
updated π*, proof, reward
    → back to graph state
```

the loop is self-improving: every cyberlink added increases |E|, raises d*, may shrink diam(G) — producing a structurally better compiled model at next compilation. the cybergraph is a compounding inference quality asset.

## relationship to existing jets and languages

### jet groups

| jet group | count | target | languages used |
|-----------|-------|--------|---------------|
| verifier jets (recursive-jets) | 5 | proof composition | Tri |
| binary jets (binary-jets) | 8 | Bt prover | Bt |
| transformer jets (this proposal) | 7 | compiled inference | Ten, Arc, Bel, Ren, Tok, Tri |
| total | 20 | complete acceleration stack | |

verifier jets are pure Tri. binary jets are pure Bt. transformer jets are the first cross-language composite jets — they compose six of the fourteen proof languages. this validates the [[cyb/languages]] architecture: the languages are independently irreducible, but their compositions produce the complex operations needed for intelligence.

### new Ren operations needed

hull_attention requires one new operation in Ren:

```
Ren operation              nox composition              jet              GFP primitive
─────────────────          ──────────────────────────   ──────────       ────────────
geometric_product          mul/add over components      geo_mul jet      fma
hull_supporting_point      convex hull binary search    hull jet         fma + cmp
hull_insert                incremental hull update      hull_upd jet     fma + cmp
```

these should be added to the Ren language spec as native operations in G(2,0,0).

## why this validates the architecture

Percepta built a WASM interpreter inside transformer weights to make LLMs compute. they needed a new architecture (2D heads, custom KV-cache, execution trace encoding).

we need none of that. the existing language set already covers every algebraic domain their construction requires. their "programs into weights" vision is our §6.6 compile_weights — already specified. their "exponentially fast attention" is one new Ren primitive. their "execution traces" are our append-only cybergraph (axiom A3).

the languages doc states: "Add any plausible new language — say, a concurrent process calculus or an optimization language — and it turns out to reduce to a composition of existing ones via Nox." this proposal confirms it: a compiled transformer inference engine — the most complex composite operation we've specified — reduces to compositions of Ten, Arc, Bel, Ren, Tok, and Tri. no new language needed.

## open questions

- hull_attention in higher dimensions: 2D hulls give O(log n). 3D hulls give O(log² n). is 2D sufficient for all tri-kernel diffusion steps, or do some semcon heads benefit from 3D?
- Bel readiness: spectral_rank uses Bel (entropy on simplex). Bel is currently "research horizon." should this jet accelerate Bel's move to engineering-ready?
- compilation frequency: recompile per-epoch (slow, high quality) or incremental (fast, approximate)?
- training residual: after compilation, what does fine-tuning learn that the graph cannot encode? quantifying this gap determines the value of the compiled path vs pure focus flow
