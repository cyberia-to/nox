//! pattern 17: look — deterministic external lookup (BBG polynomial read)
//! step 1: evaluate ns_formula to get namespace
//! step 2: evaluate key_formula to get key
//! step 3: call LookProvider to read BBG state
//! step 4: return value as field atom, or Unavailable

use crate::noun::{Order, NounId, NIL};
use crate::reduce::{Outcome, ErrorKind, cell_pair, evaluate_binary_field, make_field};
use crate::call::CallProvider;
use crate::trace::{Tracer, TraceRow};

pub fn look<const N: usize, T: Tracer>(
    order: &mut Order<N>, object: NounId, body: NounId, budget: u64,
    hints: &dyn CallProvider<N>, tracer: &mut T, depth: u64,
    row: &mut TraceRow,
) -> Outcome {
    let (ns_formula, key_formula) = match cell_pair(order, body) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    let (ns, key, budget) = match evaluate_binary_field(order, object, ns_formula, key_formula, budget, hints, tracer, depth) {
        Ok(v) => v, Err(o) => return o,
    };
    row.r[4] = ns.as_u64();
    row.r[5] = key.as_u64();
    match hints.look(ns, key) {
        Some(value) => {
            row.r[6] = value.as_u64();
            make_field(order, value, budget)
        }
        None => {
            // sentinel distinguishes "no value" from "value 0"
            row.r[6] = NIL as u64;
            Outcome::Error(ErrorKind::Unavailable)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::reduce::{reduce, Outcome, ErrorKind};
    use crate::call::{CallProvider, LookProvider, NullCalls};
    use crate::trace::NoTrace;
    use crate::noun::{Order, NounId, Tag};
    use nebu::Goldilocks;

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    fn make_look<const N: usize>(ar: &mut Order<N>, ns: u64, key: u64) -> crate::noun::NounId {
        let t17 = ar.atom(g(17), Tag::Field).unwrap();
        let t1 = ar.atom(g(1), Tag::Field).unwrap();
        let vns = ar.atom(g(ns), Tag::Field).unwrap();
        let vkey = ar.atom(g(key), Tag::Field).unwrap();
        let ns_formula = ar.cell(t1, vns).unwrap();
        let key_formula = ar.cell(t1, vkey).unwrap();
        let body = ar.cell(ns_formula, key_formula).unwrap();
        ar.cell(t17, body).unwrap()
    }

    #[test]
    fn look_null_provider_returns_unavailable() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        let formula = make_look(&mut ar, 0, 42);
        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Error(ErrorKind::Unavailable) => {}
            other => panic!("expected Unavailable, got {:?}", other),
        }
    }

    /// With NullCalls two independent orders for the same (ns, key) both return
    /// Unavailable — the result is consistent (deterministic) across orders.
    #[test]
    fn look_deterministic() {
        let run = || {
            let mut ar = Order::<1024>::new();
            let obj = ar.atom(g(0), Tag::Field).unwrap();
            let formula = make_look(&mut ar, 3, 7);
            match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
                Outcome::Error(ErrorKind::Unavailable) => true,
                other => panic!("expected Unavailable, got {:?}", other),
            }
        };
        assert!(run(), "first order: NullCalls look must return Unavailable");
        assert!(run(), "second order: NullCalls look must return Unavailable");
    }

    /// Same (ns, key) in two independent orders with the same provider
    /// must yield the same value — confluence requirement.
    #[test]
    fn look_deterministic_across_orders() {
        struct FixedLooks;
        impl LookProvider for FixedLooks {
            fn look(&self, ns: Goldilocks, key: Goldilocks) -> Option<Goldilocks> {
                // pure function of (ns, key)
                Some(ns + key)
            }
        }
        impl<const N: usize> CallProvider<N> for FixedLooks {
            fn provide(&self, _order: &mut Order<N>, _tag: Goldilocks, _object: NounId) -> Option<NounId> {
                None
            }
        }

        let run = || {
            let mut ar = Order::<1024>::new();
            let obj = ar.atom(g(0), Tag::Field).unwrap();
            let formula = make_look(&mut ar, 7, 11);
            match reduce(&mut ar, obj, formula, 1000, &FixedLooks, &mut NoTrace) {
                Outcome::Ok(r, _) => ar.atom_value(r).unwrap().0.as_u64(),
                o => panic!("{:?}", o),
            }
        };
        assert_eq!(run(), run(), "look must be deterministic across orders");
        assert_eq!(run(), 18, "look(7, 11) under FixedLooks should yield 18");
    }

    #[test]
    fn look_with_value_returns_atom() {
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
        let formula = make_look(&mut ar, 0, 42);
        match reduce(&mut ar, obj, formula, 1000, &TestLooks, &mut NoTrace) {
            Outcome::Ok(result, _) => {
                let (v, _) = ar.atom_value(result).unwrap();
                assert_eq!(v, Goldilocks::new(99));
            }
            other => panic!("expected Ok(99), got {:?}", other),
        }
    }
}
