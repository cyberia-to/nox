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

#[cfg(feature = "honeycrisp")]
mod hc_jets {
    extern crate alloc;
    use alloc::vec::Vec;

    use nebu::Goldilocks;
    use crate::data::{Reduction, Order, Data};
    use crate::reduce::{Outcome, ErrorKind, pair_children};
    use crate::call::CallProvider;
    use crate::trace::{Tracer, TraceRow};

    // ── NTT jet ──────────────────────────────────────────────────────────────

    pub fn honeycrisp_ntt_jet<const N: usize>(
        reduction: &mut Reduction<N>, object: Order, _body: Order, budget: u64,
        _hints: &dyn CallProvider<N>, _tracer: &mut dyn Tracer, _depth: u64,
        row: &mut TraceRow,
    ) -> Outcome {
        // object = [[n | formula] | [values_tree | omega]]
        let (lhs, rhs) = match pair_children(reduction, object) {
            Some(p) => p,
            None => return Outcome::Error(ErrorKind::Malformed),
        };
        let (n_id, _formula_id) = match pair_children(reduction, lhs) {
            Some(p) => p,
            None => return Outcome::Error(ErrorKind::Malformed),
        };
        let (tree_id, omega_id) = match pair_children(reduction, rhs) {
            Some(p) => p,
            None => return Outcome::Error(ErrorKind::Malformed),
        };

        let n = match reduction.atom_value(n_id) {
            Some(v) => v.as_u64() as usize,
            None => return Outcome::Error(ErrorKind::TypeError),
        };
        let omega = match reduction.atom_value(omega_id) {
            Some(v) => v,
            None => return Outcome::Error(ErrorKind::TypeError),
        };

        let size = 1usize << n;

        // Budget: n × size butterfly operations
        let cost = (n as u64).saturating_mul(size as u64);
        if budget < cost {
            return Outcome::Halt(budget);
        }
        let remaining = budget - cost;

        // Flatten the balanced binary tree into a Vec<Goldilocks>.
        let mut vals: Vec<Goldilocks> = Vec::with_capacity(size);
        if !flatten_tree(reduction, tree_id, &mut vals) {
            return Outcome::Error(ErrorKind::TypeError);
        }
        if vals.len() != size {
            return Outcome::Error(ErrorKind::TypeError);
        }

        // Convert to u64, run acpu NTT, convert back.
        let mut vals_u64: Vec<u64> = vals.iter().map(|g| g.as_u64()).collect();
        let omega_u64 = omega.as_u64();
        acpu::field::ntt_forward(&mut vals_u64, omega_u64);
        let vals_out: Vec<Goldilocks> = vals_u64.into_iter().map(Goldilocks::new).collect();

        // Write result back as a balanced binary tree into the order.
        let result_tree = match build_tree(reduction, &vals_out) {
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
        // object = [[k | formula] | [evals_tree | point]]
        let (lhs, rhs) = match pair_children(reduction, object) {
            Some(p) => p,
            None => return Outcome::Error(ErrorKind::Malformed),
        };
        let (k_id, _formula_id) = match pair_children(reduction, lhs) {
            Some(p) => p,
            None => return Outcome::Error(ErrorKind::Malformed),
        };
        let (evals_id, point_id) = match pair_children(reduction, rhs) {
            Some(p) => p,
            None => return Outcome::Error(ErrorKind::Malformed),
        };

        let k = match reduction.atom_value(k_id) {
            Some(v) => v.as_u64() as usize,
            None => return Outcome::Error(ErrorKind::TypeError),
        };

        // Flatten the balanced binary tree of evaluations into a Vec<Goldilocks>.
        let mut evals: Vec<Goldilocks> = Vec::new();
        if !flatten_tree(reduction, evals_id, &mut evals) {
            return Outcome::Error(ErrorKind::TypeError);
        }
        let expected = 1usize << k;
        if evals.len() != expected {
            return Outcome::Error(ErrorKind::TypeError);
        }

        // Decode k coordinates from the right-nested point list (big-endian, MSV first).
        let mut point: Vec<Goldilocks> = Vec::with_capacity(k);
        let mut cur = point_id;
        for _ in 0..k {
            match pair_children(reduction, cur) {
                Some((head, tail)) => match reduction.atom_value(head) {
                    Some(v) => { point.push(v); cur = tail; }
                    None => return Outcome::Error(ErrorKind::TypeError),
                },
                None => return Outcome::Error(ErrorKind::TypeError),
            }
        }

        let cost = expected as u64;
        if budget < cost {
            return Outcome::Halt(budget);
        }
        let remaining = budget - cost;

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

    // ── shared tree helpers ───────────────────────────────────────────────────

    fn flatten_tree<const N: usize>(reduction: &Reduction<N>, id: Order, out: &mut Vec<Goldilocks>) -> bool {
        let inner = match reduction.get(id) {
            Some(e) => e.inner,
            None => return false,
        };
        match inner {
            Data::Atom { .. } => match reduction.atom_value(id) {
                Some(v) => { out.push(v); true }
                None => false,
            },
            Data::Pair { left, right } => {
                flatten_tree(reduction, left, out) && flatten_tree(reduction, right, out)
            }
        }
    }

    fn build_tree<const N: usize>(reduction: &mut Reduction<N>, vals: &[Goldilocks]) -> Option<Order> {
        if vals.len() == 1 {
            return reduction.atom(vals[0]);
        }
        let mid = vals.len() / 2;
        let left  = build_tree(reduction, &vals[..mid])?;
        let right = build_tree(reduction, &vals[mid..])?;
        reduction.pair(left, right)
    }
}

/// Build the genesis registry using Honeycrisp (acpu) jets for NTT and poly_eval.
/// All other jets fall back to the CPU backend.
pub fn genesis_honeycrisp<const N: usize>() -> JetRegistry<N> {
    #[cfg(feature = "honeycrisp")]
    {
        use crate::jets::registry::compute_genesis_digests;
        use crate::jets::{merkle_verify, fri_fold, state, decider};

        let digests = compute_genesis_digests();
        let mut reg = JetRegistry::empty();

        // acpu-accelerated jets
        reg.insert_exact(digests.ntt,       hc_jets::honeycrisp_ntt_jet::<N>);
        reg.insert_exact(digests.poly_eval, hc_jets::honeycrisp_poly_eval_jet::<N>);

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
    #[cfg(not(feature = "honeycrisp"))]
    {
        super::cpu::genesis_cpu()
    }
}
