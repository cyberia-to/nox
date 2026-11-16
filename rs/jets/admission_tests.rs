use super::{admission::*, backends, formulas, registry::*};
use crate::call::NullCalls;
use crate::data::{Order, Reduction};
use crate::reduce::{reduce, reduce_with_registry, ErrorKind, Outcome};
use crate::trace::{NoTrace, TraceRow, VecTrace};
use nebu::Goldilocks;
fn atom<const N: usize>(r: &mut Reduction<N>, v: u64) -> Order {
    r.atom(Goldilocks::new(v)).unwrap()
}
fn object<const N: usize>(
    r: &mut Reduction<N>,
    n: u64,
    reference: Order,
    tree: Order,
    parameter: Order,
) -> Order {
    let exponent = atom(r, n);
    let l = r.pair(exponent, reference).unwrap();
    let h = r.pair(tree, parameter).unwrap();
    r.pair(l, h).unwrap()
}
fn all_backends<const N: usize>() -> alloc::vec::Vec<JetRegistry<N>> {
    let mut backends = alloc::vec![backends::cpu::genesis_cpu()];
    #[cfg(all(feature = "honeycrisp", target_os = "macos", target_arch = "aarch64"))]
    backends.push(super::backends::honeycrisp::genesis_honeycrisp());
    #[cfg(not(all(feature = "honeycrisp", target_os = "macos", target_arch = "aarch64")))]
    let _ = &mut backends;
    backends
}
fn copy(r: &Reduction<1024>) -> Reduction<1024> {
    let mut out = Reduction::new();
    for id in 0..r.count() {
        let copied = if let Some(value) = r.atom_value(id) {
            out.atom(value)
        } else {
            out.pair(r.head(id).unwrap(), r.tail(id).unwrap())
        };
        assert_eq!(copied, Some(id));
    }
    out
}

fn status<const N: usize>(r: &Reduction<N>, o: &Outcome) -> (u8, [u64; 4], u64) {
    match o {
        Outcome::Ok(id, b) => (0, digest_key(r.digest(*id).unwrap()), *b),
        Outcome::Halt(b) => (1, [0; 4], *b),
        Outcome::Error(e) => (2, [0; 4], *e as u64),
    }
}
fn assert_pure_fallback(r: &Reduction<1024>, obj: Order, formula: Order, budgets: &[u64]) {
    let key = digest_key(r.digest(formula).unwrap());
    for registry in all_backends() {
        for &budget in budgets {
            assert!(registry.lookup_exact_for(&key, r, obj, budget).is_none());
            let mut pure = copy(r);
            let mut accelerated = copy(r);
            let mut pt = VecTrace::default();
            let mut jt = VecTrace::default();
            let p = reduce(&mut pure, obj, formula, budget, &NullCalls, &mut pt);
            let j = reduce_with_registry(
                &mut accelerated,
                obj,
                formula,
                budget,
                &NullCalls,
                &mut jt,
                &registry,
            );
            assert_eq!(
                status(&pure, &p),
                status(&accelerated, &j),
                "budget{budget}"
            );
            assert_eq!(
                pt.0.iter().map(|r| r.r).collect::<alloc::vec::Vec<_>>(),
                jt.0.iter().map(|r| r.r).collect::<alloc::vec::Vec<_>>(),
                "declined jet changed trace or budget{budget}"
            );
        }
    }
}
#[test]
fn hostile_exponents_decline_before_shifts_or_allocations() {
    let mut r = Reduction::<1024>::new();
    let poly = formulas::build_poly_eval_formula(&mut r).unwrap();
    let ntt = formulas::build_ntt_formula(&mut r).unwrap();
    let leaf = atom(&mut r, 7);
    let p = atom(&mut r, 1);
    for exponent in [17, 32, 63, 64, 65, 1u64 << 32, 18446744069414584320] {
        for f in [poly, ntt] {
            let obj = object(&mut r, exponent, f, leaf, p);
            let key = digest_key(r.digest(f).unwrap());
            let count = r.count();
            for registry in all_backends() {
                assert!(registry.lookup_exact_for(&key, &r, obj, u64::MAX).is_none());
                let direct = registry.lookup_exact(&key).unwrap();
                assert!(matches!(
                    direct(
                        &mut r,
                        obj,
                        0,
                        u64::MAX,
                        &NullCalls,
                        &mut NoTrace,
                        0,
                        &mut TraceRow::default()
                    ),
                    Outcome::Error(ErrorKind::Unavailable)
                ));
            }
            assert_eq!(r.count(), count);
            assert_pure_fallback(&r, obj, f, &[0, 1, 2, 3, 5]);
        }
    }
}
#[test]
fn unbalanced_equal_leaf_count_and_depth_zero_pairs_follow_pure_semantics() {
    let mut r = Reduction::<1024>::new();
    let f = formulas::build_poly_eval_formula(&mut r).unwrap();
    let a = atom(&mut r, 1);
    let b = atom(&mut r, 2);
    let c = atom(&mut r, 3);
    let d = atom(&mut r, 4);
    let cd = r.pair(c, d).unwrap();
    let bcd = r.pair(b, cd).unwrap();
    let unbalanced = r.pair(a, bcd).unwrap();
    let point = r.pair(a, b).unwrap();
    let point = r.pair(a, point).unwrap();
    let obj = object(&mut r, 2, f, unbalanced, point);
    assert_pure_fallback(&r, obj, f, &[0, 1, 5, 20, 100, 1000]);
    let obj = object(&mut r, 0, f, cd, point);
    assert_pure_fallback(&r, obj, f, &[0, 1, 10, 100]);
}
#[test]
fn poly_self_reference_is_part_of_the_computation() {
    let mut r = Reduction::<1024>::new();
    let f = formulas::build_poly_eval_formula(&mut r).unwrap();
    let a = atom(&mut r, 3);
    let b = atom(&mut r, 7);
    let tree = r.pair(a, b).unwrap();
    let one = atom(&mut r, 1);
    let nine = atom(&mut r, 9);
    let replacement = r.pair(one, nine).unwrap();
    let point = r.pair(one, nine).unwrap();
    let obj = object(&mut r, 1, replacement, tree, point);
    assert_pure_fallback(&r, obj, f, &[0, 1, 10, 50, 1000]);
    let out = reduce(&mut r, obj, f, 1000, &NullCalls, &mut NoTrace);
    match out {
        Outcome::Ok(id, _) => assert_eq!(r.atom_value(id).unwrap().as_u64(), 9),
        other => panic!("{other:?}"),
    }
}
#[test]
fn actual_ntt_anchor_does_not_masquerade_as_recursive_transform() {
    let mut r = Reduction::<1024>::new();
    let f = formulas::build_ntt_formula(&mut r).unwrap();
    let a = atom(&mut r, 3);
    let b = atom(&mut r, 5);
    let pair = r.pair(a, b).unwrap();
    let omega = atom(&mut r, 2);
    let obj = object(&mut r, 1, f, pair, omega);
    assert_pure_fallback(&r, obj, f, &[0, 1, 5, 100]);
    match reduce(&mut r, obj, f, 1000, &NullCalls, &mut NoTrace) {
        Outcome::Ok(id, _) => {
            assert_eq!(r.atom_value(r.head(id).unwrap()).unwrap().as_u64(), 13);
        }
        other => panic!("{other:?}"),
    }
    let tree = r.pair(pair, pair).unwrap();
    let obj = object(&mut r, 2, f, tree, omega);
    assert_pure_fallback(&r, obj, f, &[0, 1, 5, 100]);
}
#[test]
fn bounded_shared_dag_expands_logical_positions_and_ignores_point_tail() {
    let mut r = Reduction::<1024>::new();
    let f = formulas::build_poly_eval_formula(&mut r).unwrap();
    let leaf = atom(&mut r, 7);
    let zero = atom(&mut r, 0);
    let mut tree = leaf;
    let mut point = r.pair(leaf, leaf).unwrap();
    for _ in 0..MAX_ACCELERATED_DEPTH {
        tree = r.pair(tree, tree).unwrap();
        point = r.pair(zero, point).unwrap();
    }
    let obj = object(&mut r, u64::from(MAX_ACCELERATED_DEPTH), f, tree, point);
    let key = digest_key(r.digest(f).unwrap());
    for registry in all_backends() {
        assert!(registry.lookup_exact_for(&key, &r, obj, 0).is_none());
        assert!(registry
            .lookup_exact_for(&key, &r, obj, MAX_ACCELERATED_WORDS as u64)
            .is_some());
        let direct = registry.lookup_exact(&key).unwrap();
        let count = r.count();
        assert!(matches!(
            direct(
                &mut r,
                obj,
                0,
                0,
                &NullCalls,
                &mut NoTrace,
                0,
                &mut TraceRow::default()
            ),
            Outcome::Halt(0)
        ));
        assert_eq!(r.count(), count);
        match direct(
            &mut r,
            obj,
            0,
            MAX_ACCELERATED_WORDS as u64,
            &NullCalls,
            &mut NoTrace,
            0,
            &mut TraceRow::default(),
        ) {
            Outcome::Ok(id, b) => {
                assert_eq!(b, 0);
                assert_eq!(r.atom_value(id).unwrap().as_u64(), 7);
            }
            other => panic!("{other:?}"),
        }
    }
}
#[test]
fn admitted_small_inputs_have_the_same_values_on_all_backends() {
    let mut r = Reduction::<1024>::new();
    let p = formulas::build_poly_eval_formula(&mut r).unwrap();
    let n = formulas::build_ntt_formula(&mut r).unwrap();
    let a = atom(&mut r, 3);
    let b = atom(&mut r, 7);
    let tree = r.pair(a, b).unwrap();
    let one = atom(&mut r, 1);
    let point = r.pair(one, tree).unwrap();
    for (f, param) in [(p, point), (n, one)] {
        let obj = object(&mut r, 1, f, tree, param);
        let mut pure = copy(&r);
        let expected = reduce(&mut pure, obj, f, 1000, &NullCalls, &mut NoTrace);
        let expected = match expected {
            Outcome::Ok(id, _) => digest_key(pure.digest(id).unwrap()),
            other => panic!("{other:?}"),
        };
        for registry in all_backends() {
            let key = digest_key(r.digest(f).unwrap());
            assert!(registry.lookup_exact_for(&key, &r, obj, 1000).is_some());
            let mut ar = copy(&r);
            match reduce_with_registry(&mut ar, obj, f, 1000, &NullCalls, &mut NoTrace, &registry) {
                Outcome::Ok(id, _) => assert_eq!(digest_key(ar.digest(id).unwrap()), expected),
                other => panic!("{other:?}"),
            }
        }
    }
}

#[test]
fn direct_unbalanced_calls_fail_without_mutating_the_arena() {
    let mut r = Reduction::<1024>::new();
    let poly = formulas::build_poly_eval_formula(&mut r).unwrap();
    let ntt = formulas::build_ntt_formula(&mut r).unwrap();
    let a = atom(&mut r, 1);
    let b = atom(&mut r, 2);
    let c = atom(&mut r, 3);
    let d = atom(&mut r, 4);
    let cd = r.pair(c, d).unwrap();
    let bcd = r.pair(b, cd).unwrap();
    let tree = r.pair(a, bcd).unwrap();
    let tail = r.pair(a, b).unwrap();
    let point = r.pair(a, tail).unwrap();
    for (formula, parameter) in [(poly, point), (ntt, a)] {
        let obj = object(&mut r, 2, formula, tree, parameter);
        let key = digest_key(r.digest(formula).unwrap());
        let count = r.count();
        for registry in all_backends() {
            let direct = registry.lookup_exact(&key).unwrap();
            assert!(matches!(
                direct(
                    &mut r,
                    obj,
                    0,
                    1000,
                    &NullCalls,
                    &mut NoTrace,
                    0,
                    &mut TraceRow::default()
                ),
                Outcome::Error(ErrorKind::TypeError)
            ));
            assert_eq!(r.count(), count);
        }
    }
}
