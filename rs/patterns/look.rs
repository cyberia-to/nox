//! pattern 17: look — deterministic BBG polynomial read
//! step 1: evaluate ns_formula → namespace field element (must be 0..9)
//! step 2: evaluate key_formula → key field element
//! step 3: extract C_t limbs from object at axes [4, 10, 22, 23]
//! step 4: call LookProvider.look(C_t[0], ns, key)
//! step 5: return value as field atom, or Unavailable

use nebu::Goldilocks;

use crate::data::{Reduction, Order, Data, NIL};
use crate::reduce::{Outcome, ErrorKind, pair_children, evaluate_binary_field, make_field};
use crate::call::CallProvider;
use crate::trace::{Tracer, TraceRow};
use crate::jets::registry::JetRegistry;

/// Axis addresses of the four 64-bit limbs of the BBG root within the object.
/// Object layout: `[[l0 | [l1 | [l2 | l3]]] | rest]`.
/// limb0=axis4, limb1=axis10, limb2=axis22, limb3=axis23.
pub(crate) const BBG_ROOT_LIMB_AXES: [u64; 4] = [4, 10, 22, 23];

/// Navigate to `addr` inside `object` and return the Order there.
/// Returns None if addr hits an atom mid-path or the order is inconsistent.
/// Mirrors the axis navigation logic from patterns/axis.rs without the
/// trace-row side effects (C_t extraction must not emit a row of its own).
fn axis_nav<const N: usize>(reduction: &Reduction<N>, object: Order, addr: u64) -> Option<Order> {
    match addr {
        1 => Some(object),
        _ => {
            let bits = 64 - addr.leading_zeros() - 1;
            let mut node = object;
            for i in (0..bits).rev() {
                match reduction.get(node)?.inner {
                    Data::Pair { left, right } => {
                        node = if (addr >> i) & 1 == 1 { right } else { left };
                    }
                    _ => return None,
                }
            }
            Some(node)
        }
    }
}

pub fn look<const N: usize, T: Tracer>(
    reduction: &mut Reduction<N>, object: Order, body: Order, budget: u64,
    hints: &dyn CallProvider<N>, tracer: &mut T, depth: u64,
    row: &mut TraceRow, registry: &JetRegistry<N>,
) -> Outcome {
    let (ns_formula, key_formula) = match pair_children(reduction, body) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    let (ns, key, budget) = match evaluate_binary_field(reduction, object, ns_formula, key_formula, budget, hints, tracer, depth, registry) {
        Ok(v) => v, Err(o) => return o,
    };

    // namespace must be 0..9 (the 10 public BBG evaluation dimensions)
    if ns.as_u64() > 9 {
        return Outcome::Error(ErrorKind::Unavailable);
    }

    // extract all 4 limbs of the BBG root for full circuit binding
    let mut limbs = [Goldilocks::ZERO; 4];
    for (i, &axis) in BBG_ROOT_LIMB_AXES.iter().enumerate() {
        let node = match axis_nav(reduction, object, axis) {
            Some(n) => n,
            None => return Outcome::Error(ErrorKind::Unavailable),
        };
        limbs[i] = match reduction.atom_value(node) {
            Some(v) => v,
            None => return Outcome::Error(ErrorKind::Unavailable),
        };
    }

    // r[4]=ct[0], r[5]=ns, r[6]=key, r[7]=value (below), r[11..14]=ct[1..4]
    row.r[4]  = limbs[0].as_u64();
    row.r[5]  = ns.as_u64();
    row.r[6]  = key.as_u64();
    row.r[11] = limbs[1].as_u64();
    row.r[12] = limbs[2].as_u64();
    row.r[13] = limbs[3].as_u64();

    match hints.look(limbs[0], ns, key) {
        Some(value) => {
            row.r[7] = value.as_u64();
            make_field(reduction, value, budget)
        }
        None => {
            // sentinel distinguishes "no value" from "value 0"
            row.r[7] = NIL as u64;
            Outcome::Error(ErrorKind::Unavailable)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::reduce::{reduce, Outcome, ErrorKind};
    use crate::call::{CallProvider, LookProvider, NullCalls};
    use crate::trace::NoTrace;
    use crate::data::{Reduction, Order};
    use nebu::Goldilocks;
    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    /// Build a 4-limb BBG object: `[[l0 | [l1 | [l2 | l3]]] | rest]`.
    /// limb0 is passed to LookProvider as `commitment`; limbs 1-3 go to r[11..14].
    fn make_obj<const N: usize>(ar: &mut Reduction<N>, l0: u64, l1: u64, l2: u64, l3: u64) -> Order {
        let al0  = ar.atom(g(l0)).unwrap();
        let al1  = ar.atom(g(l1)).unwrap();
        let al2  = ar.atom(g(l2)).unwrap();
        let al3  = ar.atom(g(l3)).unwrap();
        let inner     = ar.pair(al2, al3).unwrap();
        let mid       = ar.pair(al1, inner).unwrap();
        let root_pair = ar.pair(al0, mid).unwrap();
        let rest      = ar.atom(g(0)).unwrap();
        ar.pair(root_pair, rest).unwrap()
    }

    /// Build formula [17 [[1 ns] [1 key]]] — look with quoted ns and key.
    fn make_look<const N: usize>(ar: &mut Reduction<N>, ns: u64, key: u64) -> Order {
        let t17 = ar.atom(g(17)).unwrap();
        let t1  = ar.atom(g(1)).unwrap();
        let vns  = ar.atom(g(ns)).unwrap();
        let vkey = ar.atom(g(key)).unwrap();
        let ns_formula  = ar.pair(t1, vns).unwrap();
        let key_formula = ar.pair(t1, vkey).unwrap();
        let body = ar.pair(ns_formula, key_formula).unwrap();
        ar.pair(t17, body).unwrap()
    }

    #[test]
    fn look_null_provider_returns_unavailable() {
        let mut ar = Reduction::<1024>::new();
        let obj = make_obj(&mut ar, 0, 0, 0, 0);
        let formula = make_look(&mut ar, 0, 42);
        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Error(ErrorKind::Unavailable) => {}
            other => panic!("expected Unavailable, got {:?}", other),
        }
    }

    /// Two independent orders with the same (ns, key) and NullCalls both
    /// return Unavailable — determinism across independent executions.
    #[test]
    fn look_deterministic() {
        let run = || {
            let mut ar = Reduction::<1024>::new();
            let obj = make_obj(&mut ar, 0, 0, 0, 0);
            let formula = make_look(&mut ar, 3, 7);
            matches!(
                reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace),
                Outcome::Error(ErrorKind::Unavailable)
            )
        };
        assert!(run(), "first order: NullCalls look must return Unavailable");
        assert!(run(), "second order: NullCalls look must return Unavailable");
    }

    /// Same (ns, key, C_t) in two independent orders with the same provider
    /// must yield the same value — confluence requirement.
    #[test]
    fn look_deterministic_across_orders() {
        struct FixedLooks;
        impl LookProvider for FixedLooks {
            fn look(&self, _ct: Goldilocks, ns: Goldilocks, key: Goldilocks) -> Option<Goldilocks> {
                Some(ns + key)
            }
        }
        impl<const N: usize> CallProvider<N> for FixedLooks {
            fn provide(&self, _order: &mut Reduction<N>, _tag: Goldilocks, _object: Order) -> Option<Order> {
                None
            }
        }

        let run = || {
            let mut ar = Reduction::<1024>::new();
            let obj = make_obj(&mut ar, 100, 0, 0, 0); // C_t[0] = 100
            let formula = make_look(&mut ar, 7, 11);
            match reduce(&mut ar, obj, formula, 1000, &FixedLooks, &mut NoTrace) {
                Outcome::Ok(r, _) => ar.atom_value(r).unwrap().as_u64(),
                o => panic!("{:?}", o),
            }
        };
        assert_eq!(run(), run(), "look must be deterministic across orders");
        assert_eq!(run(), 18, "look(ns=7, key=11) under FixedLooks yields ns+key=18");
    }

    #[test]
    fn look_with_value_returns_atom() {
        struct TestLooks;
        impl LookProvider for TestLooks {
            fn look(&self, _ct: Goldilocks, _ns: Goldilocks, _key: Goldilocks) -> Option<Goldilocks> {
                Some(Goldilocks::new(99))
            }
        }
        impl<const N: usize> CallProvider<N> for TestLooks {
            fn provide(&self, _order: &mut Reduction<N>, _tag: Goldilocks, _object: Order) -> Option<Order> {
                None
            }
        }

        let mut ar = Reduction::<1024>::new();
        let obj = make_obj(&mut ar, 0, 0, 0, 0);
        let formula = make_look(&mut ar, 0, 42);
        match reduce(&mut ar, obj, formula, 1000, &TestLooks, &mut NoTrace) {
            Outcome::Ok(result, _) => {
                let v = ar.atom_value(result).unwrap();
                assert_eq!(v, Goldilocks::new(99));
            }
            other => panic!("expected Ok(99), got {:?}", other),
        }
    }

    /// Namespace outside 0..9 is rejected before calling the provider.
    #[test]
    fn look_namespace_out_of_range() {
        struct PanicLooks;
        impl LookProvider for PanicLooks {
            fn look(&self, _: Goldilocks, _: Goldilocks, _: Goldilocks) -> Option<Goldilocks> {
                panic!("provider must not be called for invalid namespace")
            }
        }
        impl<const N: usize> CallProvider<N> for PanicLooks {
            fn provide(&self, _: &mut Reduction<N>, _: Goldilocks, _: Order) -> Option<Order> { None }
        }

        for ns in [10u64, 100, u64::MAX] {
            let mut ar = Reduction::<1024>::new();
            let obj = make_obj(&mut ar, 0, 0, 0, 0);
            let formula = make_look(&mut ar, ns, 1);
            match reduce(&mut ar, obj, formula, 1000, &PanicLooks, &mut NoTrace) {
                Outcome::Error(ErrorKind::Unavailable) => {}
                other => panic!("ns={ns}: expected Unavailable, got {:?}", other),
            }
        }
    }

    /// When the object is an atom (no pairs for limb axes), look
    /// returns Unavailable rather than panicking.
    #[test]
    fn look_object_atom_returns_unavailable() {
        let mut ar = Reduction::<1024>::new();
        // object is a bare atom — axis(4) requires two pair levels
        let obj = ar.atom(g(0)).unwrap();
        let formula = make_look(&mut ar, 0, 1);
        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Error(ErrorKind::Unavailable) => {}
            other => panic!("expected Unavailable for atom object, got {:?}", other),
        }
    }

    /// Provider receives the C_t extracted from the object, not a default.
    #[test]
    fn look_passes_commitment_to_provider() {
        use core::sync::atomic::{AtomicU64, Ordering};
        struct CaptureLooks(AtomicU64);
        impl LookProvider for CaptureLooks {
            fn look(&self, ct: Goldilocks, _ns: Goldilocks, _key: Goldilocks) -> Option<Goldilocks> {
                self.0.store(ct.as_u64(), Ordering::Relaxed);
                Some(Goldilocks::ZERO)
            }
        }
        impl<const N: usize> CallProvider<N> for CaptureLooks {
            fn provide(&self, _: &mut Reduction<N>, _: Goldilocks, _: Order) -> Option<Order> { None }
        }

        let captured = CaptureLooks(AtomicU64::new(0));
        let mut ar = Reduction::<1024>::new();
        let obj = make_obj(&mut ar, 0xABCD, 0, 0, 0); // l0=0xABCD is passed as commitment
        let formula = make_look(&mut ar, 0, 0);
        let _ = reduce(&mut ar, obj, formula, 1000, &captured, &mut NoTrace);
        assert_eq!(captured.0.load(Ordering::Relaxed), 0xABCD, "provider must receive C_t from object axis");
    }
}
