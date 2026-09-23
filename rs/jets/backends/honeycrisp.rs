// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! Honeycrisp Apple Silicon AMX+Metal backend for genesis jets.
//!
//! Targets aarch64 macOS using Apple Matrix Coprocessor (AMX) for polynomial
//! arithmetic and Metal/MSL kernels for hash (Poseidon2) and NTT.
//! NTT and poly_eval are wired to acpu kernels.
//! All other jets fall back to the CPU backend.
//!
//! Availability check: returns true on aarch64 macOS when the honeycrisp
//! feature is enabled.

use crate::jets::registry::JetRegistry;

/// Returns true on aarch64 macOS (runtime guard for backend dispatch).
pub fn available() -> bool {
    cfg!(all(target_os = "macos", target_arch = "aarch64"))
}

#[cfg(all(feature = "honeycrisp", target_os = "macos", target_arch = "aarch64"))]
mod hc_jets {
    extern crate alloc;
    use alloc::vec::Vec;

    use nebu::Goldilocks;
    use crate::data::{Reduction, Order};
    use crate::reduce::{Outcome, ErrorKind};
    use crate::call::CallProvider;
    use crate::trace::{Tracer, TraceRow};

    // ── NTT jet ──────────────────────────────────────────────────────────────

    pub fn honeycrisp_ntt_jet<const N: usize>(
        reduction: &mut Reduction<N>, object: Order, _body: Order, budget: u64,
        _hints: &dyn CallProvider<N>, _tracer: &mut dyn Tracer, _depth: u64,
        row: &mut TraceRow,
    ) -> Outcome {
        let prepared = match crate::jets::admission::prepare_ntt(reduction, object, budget) {
            Ok(input) => input, Err(outcome) => return outcome,
        };
        let tree_id = prepared.input.tree;
        let omega_id = prepared.input.parameter;
        let omega = prepared.omega;
        let remaining = prepared.remaining;
        let vals = prepared.values;

        // Convert to u64, run acpu NTT, convert back.
        let mut vals_u64: Vec<u64> = vals.iter().map(|g| g.as_u64()).collect();
        let omega_u64 = omega.as_u64();
        acpu::field::ntt_forward(&mut vals_u64, omega_u64);
        let vals_out: Vec<Goldilocks> = vals_u64.into_iter().map(Goldilocks::new).collect();

        // Write result back as a balanced binary tree into the order.
        let result_tree = match crate::jets::ntt::build_tree(reduction, &vals_out) {
            Some(id) => id,
            None => return Outcome::Error(ErrorKind::Unavailable),
        };

        row.r[4] = tree_id as u64;
        row.r[5] = omega_id as u64;
        row.r[6] = result_tree as u64;

        Outcome::Ok(result_tree, remaining)
    }

    // ── poly_eval jet ─────────────────────────────────────────────────────────

    pub fn honeycrisp_poly_eval_jet<const N: usize>(
        reduction: &mut Reduction<N>, object: Order, _body: Order, budget: u64,
        _hints: &dyn CallProvider<N>, _tracer: &mut dyn Tracer, _depth: u64,
        row: &mut TraceRow,
    ) -> Outcome {
        let prepared = match crate::jets::admission::prepare_poly(reduction, object, budget) {
            Ok(input) => input, Err(outcome) => return outcome,
        };
        let evals_id = prepared.input.tree;
        let point_id = prepared.input.parameter;
        let remaining = prepared.remaining;
        let evals = prepared.values;
        let point = prepared.point;

        // Convert to u64, call acpu kernel, convert result back.
        let evals_u64: Vec<u64> = evals.iter().map(|g| g.as_u64()).collect();
        let point_u64: Vec<u64> = point.iter().map(|g| g.as_u64()).collect();
        let value_u64 = acpu::field::multilinear_eval(&evals_u64, &point_u64);
        let value = Goldilocks::new(value_u64);

        row.r[4] = evals_id as u64;
        row.r[5] = point_id as u64;
        row.r[6] = value.as_u64();

        match reduction.atom(value) {
            Some(r) => Outcome::Ok(r, remaining),
            None => Outcome::Error(ErrorKind::Unavailable),
        }
    }


}

/// Build the genesis registry using Honeycrisp (acpu) jets for NTT and poly_eval.
/// All other jets fall back to the CPU backend.
pub fn genesis_honeycrisp<const N: usize>() -> JetRegistry<N> {
    #[cfg(all(feature = "honeycrisp", target_os = "macos", target_arch = "aarch64"))]
    {
        use crate::jets::registry::compute_genesis_digests;
        use crate::jets::{merkle_verify, fri_fold, state, decider};

        let digests = compute_genesis_digests();
        let mut reg = JetRegistry::empty();

        // acpu-accelerated jets
        reg.insert_exact_guarded(digests.ntt, hc_jets::honeycrisp_ntt_jet::<N>, crate::jets::admission::ntt_admitted::<N>);
        reg.insert_exact_guarded(digests.poly_eval, hc_jets::honeycrisp_poly_eval_jet::<N>, crate::jets::admission::poly_admitted::<N>);

        // CPU fallbacks for remaining exact-match jets
        reg.insert_exact(digests.merkle_verify, merkle_verify::merkle_verify_jet::<N>);
        reg.insert_exact(digests.fri_fold,      fri_fold::fri_fold_jet::<N>);
        reg.insert_exact(digests.cyberlink,     state::cyberlink_jet::<N>);
        reg.insert_exact(digests.decider,       decider::decider_jet::<N>);

        // Template jets — CPU implementations
        reg.insert_template(state::is_transfer::<N>,  state::transfer_jet::<N>);
        reg.insert_template(state::is_insert::<N>,    state::insert_jet::<N>);
        reg.insert_template(state::is_update::<N>,    state::update_jet::<N>);
        reg.insert_template(state::is_aggregate::<N>, state::aggregate_jet::<N>);
        reg.insert_template(state::is_conserve::<N>,  state::conserve_jet::<N>);

        reg
    }
    #[cfg(not(all(feature = "honeycrisp", target_os = "macos", target_arch = "aarch64")))]
    {
        super::cpu::genesis_cpu()
    }
}
