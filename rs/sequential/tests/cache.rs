use super::*;

fn same_arena<const N: usize>(a: &Reduction<N>, b: &Reduction<N>) {
    assert_eq!(a.count(), b.count());
    for i in 0..a.count() {
        assert_eq!(a.atom_value(i), b.atom_value(i));
        assert_eq!(a.head(i), b.head(i));
        assert_eq!(a.tail(i), b.tail(i));
        assert_eq!(a.digest(i), b.digest(i));
        assert_eq!(crate::bound(a, i), crate::bound(b, i));
    }
}

fn repeated<const N: usize>(ar: &mut Reduction<N>, tag: u64) -> (Order, Order) {
    let object = atom(ar, 0);
    let a = quote(ar, 42);
    let b = quote(ar, 19);
    let f = if matches!(tag, 8 | 13 | 15) {
        op(ar, tag, a)
    } else {
        binary(ar, tag, a, b)
    };
    let f = binary(ar, 3, f, f);
    (object, binary(ar, 3, f, f))
}

#[test]
fn repeated_finalizers_preserve_budget_frames_and_allocation_failures() {
    for tag in [3, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15] {
        for extra in [0, 1, 2, 3, 6, 7, 8, 10, 20] {
            for budget in [0, 1, 3, 4, 8, 9, 10, 15, 32, 65, 130, 257, 512] {
                let mut old = Reduction::<128>::new();
                let mut new = Reduction::<128>::new();
                let (object, formula) = repeated(&mut old, tag);
                assert_eq!(repeated(&mut new, tag), (object, formula));
                assert!(old.limit_allocations(old.count() + extra));
                assert!(new.limit_allocations(new.count() + extra));
                let limits = Limits { max_frames: 16 };
                let a = reduce(&mut old, object, formula, budget, limits, &mut NoTrace);
                let b = reduce_cached(&mut new, object, formula, budget, limits);
                assert_eq!(std::format!("{a:?}"), std::format!("{b:?}"));
                same_arena(&old, &new);
            }
        }
    }
}

#[test]
fn cancellation_and_frame_failures_have_identical_checkpoints_and_partial_arenas() {
    for frames in 0..9 {
        for cancel_at in 1..100 {
            let mut old = Reduction::<128>::new();
            let mut new = Reduction::<128>::new();
            let (object, formula) = repeated(&mut old, 15);
            assert_eq!(repeated(&mut new, 15), (object, formula));
            let mut old_checks = 0;
            let mut new_checks = 0;
            let limits = Limits { max_frames: frames };
            let a = reduce_controlled(
                &mut old,
                object,
                formula,
                1000,
                limits,
                &mut NoTrace,
                &mut || {
                    old_checks += 1;
                    old_checks == cancel_at
                },
            );
            let b = reduce_cached_controlled(&mut new, object, formula, 1000, limits, &mut || {
                new_checks += 1;
                new_checks == cancel_at
            });
            assert_eq!(old_checks, new_checks);
            assert_eq!(std::format!("{a:?}"), std::format!("{b:?}"));
            same_arena(&old, &new);
        }
    }
}

#[test]
fn shared_arena_reuse_and_new_arenas_never_reuse_stale_orders() {
    let mut old = Reduction::<256>::new();
    let mut new = Reduction::<256>::new();
    for tag in [15, 5, 3, 8, 15, 5] {
        let (object, formula) = repeated(&mut old, tag);
        assert_eq!(repeated(&mut new, tag), (object, formula));
        for budget in [1000, 0, 64, 1000] {
            let limits = Limits { max_frames: 16 };
            let a = reduce(&mut old, object, formula, budget, limits, &mut NoTrace);
            let b = reduce_cached(&mut new, object, formula, budget, limits);
            assert_eq!(std::format!("{a:?}"), std::format!("{b:?}"));
            same_arena(&old, &new);
        }
    }
    for value in [7, 8, 9] {
        let mut ar = Reduction::<64>::new();
        let object = atom(&mut ar, 0);
        let a = quote(&mut ar, value);
        let sum = binary(&mut ar, 5, a, a);
        let run = reduce_cached(&mut ar, object, sum, 3, Limits { max_frames: 2 }).unwrap();
        let Outcome::Ok(result, 0) = run.outcome else {
            panic!("{:?}", run.outcome)
        };
        assert_eq!(ar.atom_value(result).unwrap().as_u64(), value * 2);
    }
}

#[test]
fn cached_work_never_hides_reached_services_or_child_failure_precedence() {
    for tag in [16, 17] {
        let mut ar = Reduction::<128>::new();
        let (object, pure) = repeated(&mut ar, 5);
        let service = op(&mut ar, tag, object);
        let quoted = op(&mut ar, 1, service);
        let applied = binary(&mut ar, 2, pure, quoted);
        let formula = binary(&mut ar, 3, pure, applied);
        let limits = Limits { max_frames: 16 };
        assert_eq!(
            reduce_cached(&mut ar, object, formula, 1000, limits).unwrap_err(),
            Error::UnsupportedService(tag)
        );
        let malformed = binary(&mut ar, 3, object, service);
        assert!(matches!(
            reduce_cached(&mut ar, object, malformed, 1000, limits)
                .unwrap()
                .outcome,
            Outcome::Error(ErrorKind::Malformed)
        ));
    }
}

#[test]
fn cached_dynamic_loop_keeps_legacy_depth_independence_and_exact_cost() {
    let mut ar = Reduction::<32768>::try_new_boxed().unwrap();
    let (object, formula) = loop_core(&mut ar, 5000);
    let run = reduce_cached(
        &mut ar,
        object,
        formula,
        100_000,
        Limits { max_frames: 16_384 },
    )
    .unwrap();
    let Outcome::Ok(result, remaining) = run.outcome else {
        panic!("{:?}", run.outcome)
    };
    assert_eq!(ar.atom_value(result).unwrap().as_u64(), 5000);
    assert_eq!(100_000 - remaining, 75_005);
    assert!(run.peak_frames > 5000);
}

#[test]
fn finalizer_cache_has_fixed_memory_and_never_caches_failure() {
    assert_eq!(finalizer_cache_storage_bytes(), 1_048_576);
    let mut cache = finalizer_cache::Cache::<true>::new().unwrap();
    let mut calls = 0;
    for _ in 0..2 {
        let outcome = cache.finish(3, 12, 13, 10, || {
            calls += 1;
            Outcome::Error(ErrorKind::Unavailable)
        });
        assert!(matches!(outcome, Outcome::Error(ErrorKind::Unavailable)));
    }
    assert_eq!(calls, 2);
    for remaining in [10, 0, u64::MAX] {
        let outcome = cache.finish(3, 12, 13, remaining, || {
            calls += 1;
            Outcome::Ok(14, remaining)
        });
        assert!(matches!(outcome, Outcome::Ok(14, b) if b == remaining));
    }
    assert_eq!(calls, 3);
}

#[test]
fn collisions_and_distinct_tags_always_check_the_complete_key() {
    let mut cache = finalizer_cache::Cache::<true>::new().unwrap();
    // More unique keys than fixed slots forces replacement independently of
    // the indexing function. Revisiting every key must still return its value.
    for _ in 0..2 {
        for operand in 0..70_000 {
            for tag in [3, 5, 15] {
                let result = operand + tag as u32;
                let outcome = cache.finish(tag, operand, 12, 99, || Outcome::Ok(result, 99));
                assert!(matches!(outcome, Outcome::Ok(r, 99) if r == result));
            }
        }
    }
}
