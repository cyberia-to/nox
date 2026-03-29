//! pattern 17: look — deterministic external lookup
//! step 1: evaluate ns_formula to get namespace
//! step 2: evaluate key_formula to get key
//! step 3: call LookProvider to read BBG state
//! step 4: return value as atom, or error if unavailable
//!
//! unlike call (pattern 16), look is deterministic — the same namespace
//! and key always produce the same result. no prover witness needed.

use crate::noun::{Order, NounId};
use crate::reduce::{Outcome, ErrorKind, cell_pair, evaluate_field, make_field};
use crate::call::CallProvider;

pub fn look<const N: usize>(
    order: &mut Order<N>, object: NounId, body: NounId, budget: u64, hints: &dyn CallProvider<N>,
) -> Outcome {
    let (ns_formula, key_formula) = match cell_pair(order, body) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    let (ns, budget) = match evaluate_field(order, object, ns_formula, budget, hints) {
        Ok(v) => v, Err(o) => return o,
    };
    let (key, budget) = match evaluate_field(order, object, key_formula, budget, hints) {
        Ok(v) => v, Err(o) => return o,
    };
    match hints.look(ns, key) {
        Some(value) => make_field(order, value, budget),
        None => Outcome::Error(ErrorKind::Unavailable),
    }
}

#[cfg(test)]
mod tests {
    use crate::reduce::reduce;
    use crate::call::NullCalls;
    use crate::noun::{Order, Tag};
    use crate::reduce::{Outcome, ErrorKind};
    use nebu::Goldilocks;

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    /// look with NullCalls returns Unavailable — no BBG state attached
    #[test]
    fn look_null_provider_returns_unavailable() {
        let mut ar = Order::<1024>::new();
        // object = atom 0 (dummy)
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        // body = [ns_formula key_formula] where both are quote-literals
        // ns_formula = [1 0]  (quote 0 — namespace 0)
        let one = ar.atom(g(1), Tag::Field).unwrap();
        let zero = ar.atom(g(0), Tag::Field).unwrap();
        let ns_formula = ar.cell(one, zero).unwrap();
        // key_formula = [1 42] (quote 42 — key 42)
        let forty_two = ar.atom(g(42), Tag::Field).unwrap();
        let key_formula = ar.cell(one, forty_two).unwrap();
        // body = [ns_formula key_formula]
        let body = ar.cell(ns_formula, key_formula).unwrap();
        // formula = [17 body]
        let tag = ar.atom(g(17), Tag::Field).unwrap();
        let formula = ar.cell(tag, body).unwrap();

        match reduce(&mut ar, obj, formula, 1000, &NullCalls) {
            Outcome::Error(ErrorKind::Unavailable) => {}
            other => panic!("expected Unavailable, got {:?}", other),
        }
    }

    /// look with a provider that returns a value succeeds
    #[test]
    fn look_with_value_returns_atom() {
        use crate::call::{CallProvider, LookProvider};
        use crate::noun::{Order, NounId};

        struct TestLooks;
        impl LookProvider for TestLooks {
            fn look(&self, _ns: Goldilocks, _key: Goldilocks) -> Option<Goldilocks> {
                Some(Goldilocks::new(99))
            }
        }
        impl<const N: usize> CallProvider<N> for TestLooks {
            fn provide(&self, _order: &mut Order<N>, _tag: Goldilocks, _object: NounId) -> Option<NounId> {
                None
            }
        }

        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        let one = ar.atom(g(1), Tag::Field).unwrap();
        let zero = ar.atom(g(0), Tag::Field).unwrap();
        let ns_formula = ar.cell(one, zero).unwrap();
        let forty_two = ar.atom(g(42), Tag::Field).unwrap();
        let key_formula = ar.cell(one, forty_two).unwrap();
        let body = ar.cell(ns_formula, key_formula).unwrap();
        let tag = ar.atom(g(17), Tag::Field).unwrap();
        let formula = ar.cell(tag, body).unwrap();

        match reduce(&mut ar, obj, formula, 1000, &TestLooks) {
            Outcome::Ok(result, _budget) => {
                let (v, _) = ar.atom_value(result).unwrap();
                assert_eq!(v, Goldilocks::new(99));
            }
            other => panic!("expected Ok with value 99, got {:?}", other),
        }
    }
}
