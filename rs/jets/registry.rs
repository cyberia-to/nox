// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! jet registry — formula-hash dispatch to optimized implementations
//!
//! Every genesis jet is identified by the structural digest of its pure Layer 1
//! formula data. At reduction time, reduce_inner checks H(formula) against this
//! registry before tag dispatch. If matched, the jet runs instead of the L1
//! program — identical results, orders of magnitude faster.
//!
//! Two lookup mechanisms:
//!   exact:     H(formula) → JetFn   (fixed formula, binary-search Vec)
//!   templates: predicate(formula) → JetFn  (structural pattern match, linear scan)
//!
//! Template predicates are used for state jets (TRANSFER/INSERT/UPDATE/AGGREGATE/
//! CONSERVE) which recognize a class of formulas matching a CCS pattern rather
//! than one specific data.
//!
//! Register layout convention for jets (single-row):
//!   r[0] = tag of formula's outer pattern (from reduce_inner)
//!   r[1] = object Order
//!   r[2] = formula Order
//!   r[3] = result Order (filled by reduce_inner from Outcome)
//!   r[4..8] = jet-specific operands (filled by jet)
//!   r[8] = budget before jet (from reduce_inner)
//!   r[9] = budget after jet (filled by reduce_inner from Outcome)
//!   r[10] = error kind if any (filled by reduce_inner from Outcome)

extern crate alloc;
use alloc::vec::Vec;

use crate::data::{Digest, Reduction, Order};
use crate::call::CallProvider;
use crate::trace::{Tracer, TraceRow};
use crate::reduce::Outcome;

/// Structural digest packed as four raw u64s for Ord + comparison.
/// Matches [Goldilocks; 4] but uses as_u64() so sorting is natural.
pub type DigestKey = [u64; 4];

/// Convert a structural [Goldilocks; 4] digest to a DigestKey.
#[inline]
pub fn digest_key(d: &Digest) -> DigestKey {
    [d[0].as_u64(), d[1].as_u64(), d[2].as_u64(), d[3].as_u64()]
}

/// Jet function pointer. Receives (reduction, object, body, budget, hints, tracer,
/// depth, row) and returns an Outcome. Jets handle their own budget metering.
///
/// `body` is the right half of the formula data (formula's body in [tag | body]
/// decomposition). Jets read data from `object`, not `body`, following the
/// calling convention encoded in their pure L1 formula.
pub type JetFn<const N: usize> = fn(
    &mut Reduction<N>,
    Order,                      // object (contains data per formula convention)
    Order,                      // body (formula right half — jet may ignore)
    u64,                         // budget (full, before any jet charge)
    &dyn CallProvider<N>,
    &mut dyn Tracer,
    u64,                         // depth
    &mut TraceRow,
) -> Outcome;

/// Predicate for template jets: returns true if `formula` matches the template.
pub type TemplatePredicate<const N: usize> = fn(&Reduction<N>, Order) -> bool;

/// Jet registry — maps formula digests to optimized implementations.
pub struct JetRegistry<const N: usize> {
    /// Exact matches sorted by DigestKey for binary search.
    exact: Vec<(DigestKey, JetFn<N>)>,
    /// Template matches scanned linearly (few entries, so linear is fine).
    templates: Vec<(TemplatePredicate<N>, JetFn<N>)>,
}

impl<const N: usize> JetRegistry<N> {
    /// Empty registry — no jets registered. Zero allocation.
    pub fn empty() -> Self {
        Self { exact: Vec::new(), templates: Vec::new() }
    }

    /// Insert an exact-match jet. Maintains sorted order for binary search.
    pub fn insert_exact(&mut self, key: DigestKey, f: JetFn<N>) {
        let pos = self.exact.partition_point(|(k, _)| k < &key);
        self.exact.insert(pos, (key, f));
    }

    /// Insert a template-match jet. Appended; linear scan order.
    pub fn insert_template(&mut self, pred: TemplatePredicate<N>, f: JetFn<N>) {
        self.templates.push((pred, f));
    }

    /// Look up an exact jet by formula digest key.
    #[inline]
    pub fn lookup_exact(&self, key: &DigestKey) -> Option<JetFn<N>> {
        self.exact.binary_search_by_key(key, |(k, _)| *k)
            .ok()
            .map(|i| self.exact[i].1)
    }

    /// Look up a template jet by inspecting the formula data.
    pub fn lookup_template(&self, reduction: &Reduction<N>, formula: Order) -> Option<JetFn<N>> {
        self.templates.iter()
            .find(|(pred, _)| pred(reduction, formula))
            .map(|(_, f)| *f)
    }

    /// Build the genesis registry using the best available backend.
    /// Platform/feature selection order: Honeycrisp → WGPU → CPU.
    pub fn genesis() -> Self {
        super::backends::select()
    }

    /// Number of exact-match jets registered.
    pub fn exact_count(&self) -> usize { self.exact.len() }

    /// Number of template jets registered.
    pub fn template_count(&self) -> usize { self.templates.len() }
}

// ── genesis digest computation ────────────────────────────────────────────────

/// Structural digests for all genesis jets, computed from their pure L1 formulas.
pub struct GenesisDigests {
    pub poly_eval:      DigestKey,
    pub merkle_verify:  DigestKey,
    pub fri_fold:       DigestKey,
    pub ntt:            DigestKey,
    pub cyberlink:      DigestKey,
    pub decider:        DigestKey,
}

/// Compute all genesis formula digests in a scratch Reduction.
/// Uses a large enough Reduction to build all formulas without collision.
pub fn compute_genesis_digests() -> GenesisDigests {
    use crate::jets::formulas;
    // Reduction::<4096> is ~400 KB; genesis formulas total ~200-300 unique nodes.
    let mut ar = Reduction::<4096>::new();
    let poly_eval = formulas::build_poly_eval_formula(&mut ar)
        .map(|id| digest_key(ar.digest(id).unwrap()))
        .unwrap_or([0; 4]);
    let merkle_verify = formulas::build_merkle_verify_formula(&mut ar)
        .map(|id| digest_key(ar.digest(id).unwrap()))
        .unwrap_or([0; 4]);
    let fri_fold = formulas::build_fri_fold_formula(&mut ar)
        .map(|id| digest_key(ar.digest(id).unwrap()))
        .unwrap_or([0; 4]);
    let ntt = formulas::build_ntt_formula(&mut ar)
        .map(|id| digest_key(ar.digest(id).unwrap()))
        .unwrap_or([0; 4]);
    let cyberlink = formulas::build_cyberlink_formula(&mut ar)
        .map(|id| digest_key(ar.digest(id).unwrap()))
        .unwrap_or([0; 4]);
    let decider = formulas::build_decider_formula(&mut ar)
        .map(|id| digest_key(ar.digest(id).unwrap()))
        .unwrap_or([0; 4]);
    GenesisDigests { poly_eval, merkle_verify, fri_fold, ntt, cyberlink, decider }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Reduction;

    #[test]
    fn empty_registry_returns_none() {
        let reg = JetRegistry::<1024>::empty();
        assert!(reg.lookup_exact(&[0u64; 4]).is_none());
    }

    #[test]
    fn insert_and_lookup_exact() {
        let mut reg = JetRegistry::<1024>::empty();
        fn dummy_jet<const N: usize>(
            _: &mut Reduction<N>, _: Order, _: Order, b: u64,
            _: &dyn CallProvider<N>, _: &mut dyn Tracer, _: u64,
            _: &mut TraceRow,
        ) -> Outcome {
            Outcome::Halt(b)
        }
        let key = [1u64, 2, 3, 4];
        reg.insert_exact(key, dummy_jet);
        assert!(reg.lookup_exact(&key).is_some());
        assert!(reg.lookup_exact(&[0u64; 4]).is_none());
    }

    #[test]
    fn sorted_insert_maintains_binary_search() {
        let mut reg = JetRegistry::<1024>::empty();
        fn jet_a<const N: usize>(
            _: &mut Reduction<N>, _: Order, _: Order, b: u64,
            _: &dyn CallProvider<N>, _: &mut dyn Tracer, _: u64, _: &mut TraceRow,
        ) -> Outcome { Outcome::Halt(b) }
        fn jet_b<const N: usize>(
            _: &mut Reduction<N>, _: Order, _: Order, b: u64,
            _: &dyn CallProvider<N>, _: &mut dyn Tracer, _: u64, _: &mut TraceRow,
        ) -> Outcome { Outcome::Halt(b + 1) }
        // Insert out of order
        reg.insert_exact([5, 0, 0, 0], jet_b);
        reg.insert_exact([2, 0, 0, 0], jet_a);
        assert!(reg.lookup_exact(&[2, 0, 0, 0]).is_some());
        assert!(reg.lookup_exact(&[5, 0, 0, 0]).is_some());
        assert!(reg.lookup_exact(&[3, 0, 0, 0]).is_none());
    }

    #[test]
    fn template_predicate_matches() {
        let mut reg = JetRegistry::<1024>::empty();
        fn always_match<const N: usize>(_: &Reduction<N>, _: Order) -> bool { true }
        fn jet<const N: usize>(
            _: &mut Reduction<N>, _: Order, _: Order, b: u64,
            _: &dyn CallProvider<N>, _: &mut dyn Tracer, _: u64, _: &mut TraceRow,
        ) -> Outcome { Outcome::Halt(b) }
        reg.insert_template(always_match, jet);
        let ar = Reduction::<1024>::new();
        assert!(reg.lookup_template(&ar, 0).is_some());
    }

    #[test]
    fn genesis_digests_are_deterministic() {
        let d1 = compute_genesis_digests();
        let d2 = compute_genesis_digests();
        assert_eq!(d1.poly_eval, d2.poly_eval);
        assert_eq!(d1.merkle_verify, d2.merkle_verify);
    }
}
