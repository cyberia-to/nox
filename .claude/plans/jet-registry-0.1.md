---
name: jet registry — 0.1.0
description: full jet registry with 3 backends for all 12 genesis jets
status: draft
date: 2026-05-18
---

# jet registry — 0.1.0

12 genesis jets: hash (1) + recursion (4) + state (6) + decider (1).
3 backends: CPU/Rust (reference), WGPU (cross-platform GPU), Honeycrisp (Apple Silicon AMX+Metal).
All backends ship in 0.1.0.

## directory layout

```
rs/jets/
  mod.rs            — update: expose registry, backends, new jet modules
  registry.rs       — JetRegistry, JetFn, genesis()
  formulas.rs       — existing + fri_fold, ntt, state jets, decider formulas
  poly_eval.rs      — existing CPU impl
  merkle_verify.rs  — existing CPU impl
  fri_fold.rs       — new CPU impl
  ntt.rs            — new CPU impl
  state.rs          — new: CYBERLINK exact + 5 template jets
  decider.rs        — new: HyperNova 89/825-constraint verifier
  backends/
    mod.rs
    cpu.rs          — genesis_cpu(): function pointers wrapping all CPU impls
    wgpu.rs         — genesis_wgpu(): WGSL kernel dispatch
    honeycrisp.rs   — genesis_honeycrisp(): AMX+Metal dispatch
rs/patterns/hash.rs — add feature-flag backend selection (hash is primitive, not formula-hash recognized)
```

## core types (registry.rs)

```rust
// key type: structural digest as [u64;4] for Ord/Hash without nebu dep at boundary
pub type DigestKey = [u64; 4];
pub fn digest_key(d: &[Goldilocks; 4]) -> DigestKey { d.map(|g| g.as_u64()) }

pub type JetFn<const N: usize> = fn(
    &mut Order<N>, NounId, NounId, u64,          // order, object, body, budget
    &dyn CallProvider<N>, &mut dyn Tracer, u64,   // hints, tracer, depth
    &mut TraceRow,
) -> Outcome;

pub type TemplatePredicate<const N: usize> = fn(&Order<N>, NounId) -> bool;

pub struct JetRegistry<const N: usize> {
    exact:     Vec<(DigestKey, JetFn<N>)>,                // sorted, binary search
    templates: Vec<(TemplatePredicate<N>, JetFn<N>)>,     // linear scan
}

impl<const N: usize> JetRegistry<N> {
    pub fn empty() -> Self { Self { exact: Vec::new(), templates: Vec::new() } }

    pub fn insert_exact(&mut self, key: DigestKey, f: JetFn<N>) {
        let pos = self.exact.partition_point(|(k, _)| k < &key);
        self.exact.insert(pos, (key, f));
    }

    pub fn insert_template(&mut self, pred: TemplatePredicate<N>, f: JetFn<N>) {
        self.templates.push((pred, f));
    }

    pub fn lookup_exact(&self, key: &DigestKey) -> Option<JetFn<N>> {
        self.exact.binary_search_by_key(key, |(k, _)| *k)
            .ok().map(|i| self.exact[i].1)
    }

    pub fn lookup_template(&self, order: &Order<N>, formula: NounId) -> Option<JetFn<N>> {
        self.templates.iter().find(|(pred, _)| pred(order, formula)).map(|(_, f)| *f)
    }

    pub fn genesis() -> Self { backends::select() }
}
```

## reduce_inner integration

Add `registry: &JetRegistry<N>` to `reduce()` and `reduce_inner()`.

Check BEFORE tag dispatch and BEFORE budget charge:

```rust
// after parsing (tag_ref, body) from formula:
let formula_key = order.digest(formula).map(|d| digest_key(d));
if let Some(key) = formula_key {
    if let Some(jet_fn) = registry.lookup_exact(&key) {
        // jet handles all budget metering
        return jet_fn(order, object, body, budget, hints, tracer as &mut dyn Tracer, depth, &mut row);
    }
    if let Some(jet_fn) = registry.lookup_template(order, formula) {
        return jet_fn(order, object, body, budget, hints, tracer as &mut dyn Tracer, depth, &mut row);
    }
}
// normal tag dispatch follows (existing code)
```

All existing tests pass `&JetRegistry::empty()`. All callers (CLI, bench) pass `&JetRegistry::genesis()`.

## hash jet — backend selection (patterns/hash.rs)

Hash (pattern 15) is a primitive tag — no fixed formula noun, no formula-hash lookup.
Backend selected at compile time via feature flags:

```rust
#[cfg(all(target_os = "macos", target_arch = "aarch64", feature = "honeycrisp"))]
fn run_poseidon2(input: &[Goldilocks; 8]) -> [Goldilocks; 16] { honeycrisp::poseidon2(input) }
#[cfg(all(feature = "wgpu", not(feature = "honeycrisp")))]
fn run_poseidon2(input: &[Goldilocks; 8]) -> [Goldilocks; 16] { wgpu::poseidon2(input) }
#[cfg(not(any(feature = "wgpu", feature = "honeycrisp")))]
fn run_poseidon2(input: &[Goldilocks; 8]) -> [Goldilocks; 16] { cpu::poseidon2(input) }
```

## budget metering per jet

Jets charge their own budget (no separate tag-cost is deducted before calling a jet):

| jet | budget charge |
|-----|--------------|
| hash | 25 (24 rounds + 1 squeeze) — pattern 15, not registry |
| poly_eval | 2^k (one unit per eval leaf, k = num_vars) |
| merkle_verify | depth × 25 (one hash per step) |
| fri_fold | N/2 (one fold per pair) |
| ntt | N × log2(N) (butterfly count) |
| CYBERLINK | 3200 (CCS encoding cost) |
| TRANSFER/INSERT/UPDATE/AGGREGATE/CONSERVE | 1–5 (per spec constraints column) |
| decider | 825 (conservative, pending algebraic FS verification) |

## jets: status and work needed

### already implemented (CPU, no registry wiring yet)
| jet | formula | CPU impl |
|-----|---------|---------|
| poly_eval | formulas.rs ✓ | poly_eval.rs ✓ |
| merkle_verify | formulas.rs ✓ | merkle_verify.rs ✓ |

### new CPU implementations needed

**fri_fold** (fri_fold.rs):
- formula: recursive tree fold. object = [evals_tree | r]. axis 2 = evals, axis 3 = r.
  split left/right halves of evals, recurse on each, combine: out[i] = (1-r)*left[i] + r*right[i].
  base case: depth=0 → single element, return it.
  formula shape: branch(eq(axis4, 0), axis6, recursive_combine)
  — axis 4 = depth (prepended to object by caller), axis 5 = self-ref, axis 6 = base_eval.
- CPU impl: iterative halving using Order tree traversal.

**ntt** (ntt.rs):
- formula: Cooley-Tukey recursive. object = [values_tree | [root_of_unity | direction]].
  even/odd split → recurse → butterfly combine with twiddle factor.
  direction: 0 = forward, 1 = inverse (divide by N at end).
  formula uses binop(7=mul) + binop(5=add) + binop(6=sub) for butterfly.
- CPU impl: iterative bit-reversal permutation + butterfly passes.

**state.rs** (6 jets):

CYBERLINK (exact match, formula = fixed nox cyberlink validation circuit):
- formula: constructs cyberlink: validates from/to graph nodes, checks auth, writes CYBERLINK edge.
  full circuit (~3200 constraint equivalent). fixed noun → single formula hash.
- CPU impl: direct Rust validation (parse fields, check sig, write edge to BBG state context).

5 templates (pattern match via TemplatePredicate):
- TRANSFER: pred matches formula shape `[READ(src) | [READ(src_bal) | [READ(tgt_bal) | ...]]]`
  with specific pattern of 2 reads + range check + 2 adds + 2 writes + assert_eq
- INSERT/UPDATE/AGGREGATE/CONSERVE: each matches specific composition of READ/WRITE/ASSERT_EQ/ADD/MUL
- CPU impls: direct field arithmetic, no pattern-level reduction

**decider.rs**:
- formula: nox program that reads HyperNova accumulator noun + Lens commitments, runs
  sumcheck replay (20 constraints), CCS evaluation (34), Brakedown spot-checks (35).
  Conservative: adds one Poseidon2 call (hemera pattern 15) for Fiat-Shamir = 825.
- CPU impl: direct Rust using hemera + lens crates.

### formulas.rs additions needed

`build_fri_fold_formula`, `build_ntt_formula`, `build_cyberlink_formula`, `build_decider_formula`
and their corresponding `*_formula_hash` functions + tests.

## backends

### backends/cpu.rs (genesis_cpu)
Wraps existing jet functions as JetFn<N> pointers. Computes formula digests at startup.

```rust
pub fn genesis_cpu<const N: usize>() -> JetRegistry<N> {
    let mut reg = JetRegistry::empty();
    let digests = compute_genesis_digests::<65536>(); // small order for digest computation
    reg.insert_exact(digests.poly_eval,      poly_eval::poly_eval_jet);
    reg.insert_exact(digests.merkle_verify,  merkle_verify::merkle_verify_jet);
    reg.insert_exact(digests.fri_fold,       fri_fold::fri_fold_jet);
    reg.insert_exact(digests.ntt,            ntt::ntt_jet);
    reg.insert_exact(digests.cyberlink,      state::cyberlink_jet);
    reg.insert_exact(digests.decider,        decider::decider_jet);
    reg.insert_template(state::is_transfer,  state::transfer_jet);
    reg.insert_template(state::is_insert,    state::insert_jet);
    reg.insert_template(state::is_update,    state::update_jet);
    reg.insert_template(state::is_aggregate, state::aggregate_jet);
    reg.insert_template(state::is_conserve,  state::conserve_jet);
    reg
}
```

`compute_genesis_digests` builds all formulas in a scratch Order::<65536> and extracts digests.

### backends/wgpu.rs (genesis_wgpu)
Feature flag: `wgpu`. Cargo.toml: `wgpu = { version = "22", optional = true }`.

GPU-accelerated jets: hash (Poseidon2), poly_eval (FMA), ntt (butterfly), fri_fold (fold).
State jets and decider remain CPU (single-element field arithmetic, no parallelism benefit).

Init path: `WgpuBackend::new()` creates `wgpu::Device` + `wgpu::Queue` + pre-compiled pipelines.
Each GPU jet: upload noun data to GPU buffer → dispatch compute shader → read back result.

WGSL kernels: `jets/backends/wgpu_kernels/{poseidon2,poly_eval,ntt,fri_fold}.wgsl`
Included at compile time via `include_str!`.

### backends/honeycrisp.rs (genesis_honeycrisp)
Feature flags: `honeycrisp`, cfg(target_os = "macos", target_arch = "aarch64").

AMX (Apple Matrix coprocessor) for FMA-heavy: poly_eval Horner chain, fri_fold.
Metal compute for parallelizable: hash Poseidon2 (batch), ntt.

Metal kernels: `jets/backends/honeycrisp_kernels/{poseidon2,ntt}.metal`
AMX bindings: via `amx` crate or direct inline-asm (`asm!` blocks for AMX instructions).

### backends/mod.rs — backend selection

```rust
pub fn select<const N: usize>() -> JetRegistry<N> {
    #[cfg(all(target_os = "macos", target_arch = "aarch64", feature = "honeycrisp"))]
    { return honeycrisp::genesis_honeycrisp(); }
    #[cfg(feature = "wgpu")]
    { return wgpu::genesis_wgpu(); }
    cpu::genesis_cpu()
}
```

## Cargo.toml additions

```toml
[features]
default = []
wgpu = ["dep:wgpu"]
honeycrisp = []   # amx via asm!, metal via objc crate

[dependencies]
wgpu = { version = "22", optional = true }
```

## implementation order

1. registry.rs: types + empty/insert/lookup/genesis_cpu (poly_eval + merkle_verify only)
2. reduce_inner wiring + all callers updated
3. backends/cpu.rs skeleton
4. fri_fold: formula + CPU impl + tests
5. ntt: formula + CPU impl + tests
6. state.rs: CYBERLINK formula + CPU; template predicates + CPU impls
7. decider.rs: formula + CPU (825-constraint conservative tier)
8. backends/wgpu.rs: GPU kernels for hash/poly_eval/ntt/fri_fold
9. backends/honeycrisp.rs: AMX/Metal kernels

## estimation

| phase | pomodoros |
|-------|-----------|
| registry + reduce wiring | 1 |
| fri_fold | 2 |
| ntt | 2 |
| state jets (6) | 3 |
| decider | 2 |
| WGPU backend | 4 |
| Honeycrisp backend | 4 |
| tests across all | 2 |
| **total** | **20 pomodoros (~3-4 sessions)** |
