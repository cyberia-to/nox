// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! CPU/Rust reference backend for genesis jets.
//!
//! Registers jet function pointers by formula digest.
//! All jet functions match JetFn<N> directly (non-generic over Tracer).

use crate::jets::registry::{JetRegistry, compute_genesis_digests};
use crate::jets::{poly_eval, merkle_verify, fri_fold, ntt, state, decider};

/// Build the CPU genesis registry. Called at startup; digests computed once.
pub fn genesis_cpu<const N: usize>() -> JetRegistry<N> {
    let digests = compute_genesis_digests();
    let mut reg = JetRegistry::empty();

    // ── exact-match jets ─────────────────────────────────────────────────────
    reg.insert_exact(digests.poly_eval,     poly_eval::poly_eval_jet::<N>);
    reg.insert_exact(digests.merkle_verify, merkle_verify::merkle_verify_jet::<N>);
    reg.insert_exact(digests.fri_fold,      fri_fold::fri_fold_jet::<N>);
    reg.insert_exact(digests.ntt,           ntt::ntt_jet::<N>);
    reg.insert_exact(digests.cyberlink,     state::cyberlink_jet::<N>);
    reg.insert_exact(digests.decider,       decider::decider_jet::<N>);

    // ── template jets ────────────────────────────────────────────────────────
    reg.insert_template(state::is_transfer::<N>,  state::transfer_jet::<N>);
    reg.insert_template(state::is_insert::<N>,    state::insert_jet::<N>);
    reg.insert_template(state::is_update::<N>,    state::update_jet::<N>);
    reg.insert_template(state::is_aggregate::<N>, state::aggregate_jet::<N>);
    reg.insert_template(state::is_conserve::<N>,  state::conserve_jet::<N>);

    reg
}
