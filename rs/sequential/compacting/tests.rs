extern crate std;
use super::*;
use crate::ErrorKind;
use crate::data::Data;
use crate::sequential::tests::{atom, axis, binary, loop_core, op, quote};

mod collection;
mod compiler;
mod frontier;

fn limits() -> CompactionLimits {
    CompactionLimits {
        max_frames: 16384,
        max_total_allocations: 100_000,
        max_collection_work: 100_000_000,
    }
}

fn snapshot<const N: usize>(ar: &Reduction<N>) -> Vec<std::string::String> {
    (0..ar.count())
        .map(|id| std::format!("{:?}", ar.get(id).unwrap()))
        .collect()
}

fn valid_index<const N: usize>(ar: &mut Reduction<N>) {
    let count = ar.count();
    for id in 0..count {
        let entry = *ar.get(id).unwrap();
        let interned = match entry.inner {
            Data::Atom { value } => ar.atom(value),
            Data::Pair { left, right } => {
                assert!(left < id && right < id);
                ar.pair(left, right)
            }
        };
        assert_eq!(interned, Some(id), "interned entry {id}");
    }
    assert_eq!(ar.count(), count);
}

#[test]
fn repeated_collections_keep_pins_sharing_bounds_gas_and_logical_depth() {
    let mut old = Reduction::<32768>::try_new_boxed().unwrap();
    let mut new = Reduction::<256>::new();
    let (object, formula) = loop_core(&mut new, 3000);
    assert_eq!(loop_core(&mut old, 3000), (object, formula));
    // Unrelated caller-held data is pinned too.
    let extra = quote(&mut new, 777);
    assert_eq!(quote(&mut old, 777), extra);
    let pins = snapshot(&new);
    assert!(new.limit_allocations(160));
    let expected = reduce_cached(
        &mut old,
        object,
        formula,
        100_000,
        Limits { max_frames: 16384 },
    )
    .unwrap();
    let actual = reduce_compacting_cached(&mut new, object, formula, 100_000, limits()).unwrap();
    let (Outcome::Ok(a, left_a), Outcome::Ok(b, left_b)) = (expected.outcome, actual.outcome)
    else {
        panic!("both profiles must complete the loop");
    };
    assert_eq!(old.digest(a), new.digest(b));
    assert_eq!(left_a, left_b);
    assert_eq!(left_b, 100_000 - 45_005);
    assert_eq!(expected.peak_frames, actual.peak_frames);
    assert!(actual.stats.collections > 10);
    assert!(actual.stats.reclaimed_nodes > 5000);
    assert_eq!(
        actual.stats.total_allocations,
        u64::from(new.count()) + actual.stats.reclaimed_nodes
    );
    assert_eq!(actual.stats.pinned_nodes as usize, pins.len());
    assert!(actual.stats.peak_resident_nodes <= 160);
    assert_eq!(actual.stats.scratch_bytes, (256 + 160) * 4);
    assert_eq!(&snapshot(&new)[..pins.len()], pins);
    for id in 0..new.count() {
        let matching = (0..old.count())
            .find(|old_id| old.digest(*old_id) == new.digest(id))
            .unwrap();
        assert_eq!(crate::bound(&new, id), crate::bound(&old, matching));
    }
    valid_index(&mut new);
    // A later call pins the previous result and all other resident entries.
    let q = op(&mut new, 1, b);
    let count = new.count();
    let second = reduce_compacting_cached(&mut new, object, q, 1, limits()).unwrap();
    assert!(matches!(second.outcome, Outcome::Ok(result, 0) if result == b));
    assert_eq!(second.stats.pinned_nodes, count);
}

#[test]
fn no_collection_preserves_all_pure_patterns_failures_budgets_and_checkpoints() {
    for tag in 0..16 {
        for value in [0, 7, 1 << 32] {
            for budget in [0, 1, 3, 25, 33, 65, 1000] {
                fn build<const N: usize>(
                    ar: &mut Reduction<N>,
                    tag: u64,
                    value: u64,
                ) -> (Order, Order) {
                    let object = atom(ar, 0);
                    let q = quote(ar, value);
                    let other = quote(ar, 7);
                    let f = match tag {
                        0 => axis(ar, value),
                        1 => op(ar, 1, other),
                        8 | 13 | 15 => op(ar, tag, q),
                        4 => {
                            let arms = ar.pair(q, other).unwrap();
                            binary(ar, 4, q, arms)
                        }
                        _ => binary(ar, tag, q, other),
                    };
                    (object, f)
                }
                let mut old = Reduction::<256>::new();
                let mut new = Reduction::<256>::new();
                let (object, formula) = build(&mut old, tag, value);
                assert_eq!(build(&mut new, tag, value), (object, formula));
                let mut old_checks = 0;
                let mut new_checks = 0;
                let a = reduce_cached_controlled(
                    &mut old,
                    object,
                    formula,
                    budget,
                    Limits { max_frames: 64 },
                    &mut || {
                        old_checks += 1;
                        false
                    },
                )
                .unwrap();
                let b = reduce_compacting_cached_controlled(
                    &mut new,
                    object,
                    formula,
                    budget,
                    limits(),
                    &mut || {
                        new_checks += 1;
                        false
                    },
                )
                .unwrap();
                assert_eq!(
                    std::format!("{:?}", a.outcome),
                    std::format!("{:?}", b.outcome)
                );
                assert_eq!(a.peak_frames, b.peak_frames);
                assert_eq!(old_checks, new_checks);
                assert_eq!(b.stats.evaluator_checkpoints, new_checks);
                assert_eq!(b.stats.collections, 0);
                assert_eq!(snapshot(&old), snapshot(&new));
            }
        }
    }
}

#[test]
fn cumulative_admission_counts_partial_hash_output_and_restores_resident_allowance() {
    for extra in 0..7 {
        let mut ar = Reduction::<128>::new();
        let object = atom(&mut ar, 0);
        let q = quote(&mut ar, 42);
        let f = op(&mut ar, 15, q);
        let loaded = ar.count();
        let resident = ar.allocation_limit();
        let mut cap = limits();
        cap.max_total_allocations = u64::from(loaded + extra);
        let failed = reduce_compacting_cached(&mut ar, object, f, 1000, cap).unwrap_err();
        assert_eq!(failed.kind, CompactionFailureKind::TotalAllocations);
        assert_eq!(failed.stats.total_allocations, cap.max_total_allocations);
        assert_eq!(ar.count(), loaded + extra);
        assert_eq!(ar.allocation_limit(), resident);
        valid_index(&mut ar);
        assert!(ar.atom(nebu::Goldilocks::new(987654)).is_some());
    }
}

#[test]
fn resident_exhaustion_keeps_partial_output_and_no_collection_work_fails_before_mutation() {
    for extra in 0..7 {
        let mut ar = Reduction::<128>::new();
        let object = atom(&mut ar, 0);
        let q = quote(&mut ar, 42);
        let f = op(&mut ar, 15, q);
        let loaded = ar.count();
        assert!(ar.limit_allocations(loaded + extra));
        let before = snapshot(&ar);
        let mut cap = limits();
        cap.max_collection_work = 0;
        let failed = reduce_compacting_cached(&mut ar, object, f, 1000, cap).unwrap_err();
        assert_eq!(failed.kind, CompactionFailureKind::CollectionWork);
        assert_eq!(snapshot(&ar), before);
        let result = reduce_compacting_cached(&mut ar, object, f, 1000, limits()).unwrap();
        assert!(matches!(
            result.outcome,
            Outcome::Error(ErrorKind::Unavailable)
        ));
        assert_eq!(ar.count(), loaded + extra);
        valid_index(&mut ar);
    }
}

#[test]
fn frame_cancellation_and_service_errors_leave_a_reusable_arena() {
    for tag in [16, 17] {
        let mut ar = Reduction::<128>::new();
        let object = atom(&mut ar, 0);
        let f = op(&mut ar, tag, object);
        let cap = ar.allocation_limit();
        let error = reduce_compacting_cached(&mut ar, object, f, 100, limits()).unwrap_err();
        assert_eq!(
            error.kind,
            CompactionFailureKind::Execution(Error::UnsupportedService(tag))
        );
        assert_eq!(ar.allocation_limit(), cap);
        valid_index(&mut ar);
    }
    for max_frames in [0, 1, 3] {
        let mut ar = Reduction::<256>::new();
        let (object, formula) = loop_core(&mut ar, 20);
        let mut cap = limits();
        cap.max_frames = max_frames;
        let failed = reduce_compacting_cached(&mut ar, object, formula, 1000, cap).unwrap_err();
        assert_eq!(failed.kind, CompactionFailureKind::Execution(Error::Frames));
        let cancelled = reduce_compacting_cached_controlled(
            &mut ar,
            object,
            formula,
            1000,
            limits(),
            &mut || true,
        )
        .unwrap_err();
        assert_eq!(
            cancelled.kind,
            CompactionFailureKind::Execution(Error::Cancelled)
        );
        assert_eq!(cancelled.stats.evaluator_checkpoints, 1);
        valid_index(&mut ar);
    }
}

#[test]
fn axis_zero_entry_collects_discarded_hash_output_before_allocating() {
    fn build<const N: usize>(ar: &mut Reduction<N>) -> (Order, Order) {
        let object = atom(ar, 42);
        let q7 = quote(ar, 7);
        let hash = op(ar, 15, q7);
        let q42 = quote(ar, 42);
        let quoted = op(ar, 1, q42);
        let discard = binary(ar, 2, hash, quoted);
        let a0 = axis(ar, 0);
        let qa0 = op(ar, 1, a0);
        (object, binary(ar, 2, discard, qa0))
    }
    let mut old = Reduction::<128>::new();
    let mut new = Reduction::<128>::new();
    let (object, formula) = build(&mut old);
    assert_eq!(build(&mut new), (object, formula));
    assert!(new.limit_allocations(new.count() + 7));
    let expected =
        reduce_cached(&mut old, object, formula, 1000, Limits { max_frames: 64 }).unwrap();
    let actual = reduce_compacting_cached(&mut new, object, formula, 1000, limits()).unwrap();
    let (Outcome::Ok(a, left_a), Outcome::Ok(b, left_b)) = (expected.outcome, actual.outcome)
    else {
        panic!("axis zero must complete");
    };
    assert_eq!(old.digest(a), new.digest(b));
    assert_eq!(left_a, left_b);
    assert_eq!(expected.peak_frames, actual.peak_frames);
    assert!(actual.stats.reclaimed_nodes >= 7);
    valid_index(&mut new);
}

#[test]
fn full_pinned_arena_accepts_intern_hits_with_zero_fresh_allocation_allowance() {
    for tag in [0, 3, 5, 15] {
        let mut ar = Reduction::<128>::new();
        let object = atom(&mut ar, 42);
        let q = quote(&mut ar, 7);
        let f = match tag {
            0 => axis(&mut ar, 0),
            15 => op(&mut ar, 15, q),
            _ => binary(&mut ar, tag, q, q),
        };
        let expected = reduce_cached(&mut ar, object, f, 1000, Limits { max_frames: 64 }).unwrap();
        let loaded = ar.count();
        assert!(ar.limit_allocations(loaded));
        let mut cap = limits();
        cap.max_total_allocations = u64::from(loaded);
        let actual = reduce_compacting_cached(&mut ar, object, f, 1000, cap).unwrap();
        assert_eq!(
            std::format!("{:?}", expected.outcome),
            std::format!("{:?}", actual.outcome)
        );
        assert_eq!(actual.stats.total_allocations, u64::from(loaded));
        assert_eq!(actual.stats.reclaimed_nodes, 0);
        assert!(actual.stats.collections > 0);
        valid_index(&mut ar);
    }
}

#[test]
fn collection_keeps_budget_halts_and_frame_failures_at_the_same_guest_boundary() {
    for budget in [0, 1, 3, 30, 199, 1504, 1505, 2000] {
        let mut old = Reduction::<1024>::new();
        let mut new = Reduction::<256>::new();
        let (object, formula) = loop_core(&mut old, 100);
        assert_eq!(loop_core(&mut new, 100), (object, formula));
        assert!(new.limit_allocations(100));
        let a = reduce_cached(
            &mut old,
            object,
            formula,
            budget,
            Limits { max_frames: 1000 },
        )
        .unwrap();
        let b = reduce_compacting_cached(&mut new, object, formula, budget, limits()).unwrap();
        match (a.outcome, b.outcome) {
            (Outcome::Ok(a, x), Outcome::Ok(b, y)) => {
                assert_eq!(old.digest(a), new.digest(b));
                assert_eq!(x, y);
            }
            (Outcome::Halt(x), Outcome::Halt(y)) => assert_eq!(x, y),
            (x, y) => panic!("unexpected outcomes: {x:?}, {y:?}"),
        }
        assert_eq!(a.peak_frames, b.peak_frames);
        valid_index(&mut new);
    }
    for frames in [90, 120, 150] {
        let mut old = Reduction::<1024>::new();
        let mut new = Reduction::<256>::new();
        let (object, formula) = loop_core(&mut old, 100);
        assert_eq!(loop_core(&mut new, 100), (object, formula));
        assert!(new.limit_allocations(100));
        let mut cap = limits();
        cap.max_frames = frames;
        assert_eq!(
            reduce_cached(
                &mut old,
                object,
                formula,
                2000,
                Limits { max_frames: frames }
            )
            .unwrap_err(),
            Error::Frames
        );
        let error = reduce_compacting_cached(&mut new, object, formula, 2000, cap).unwrap_err();
        assert_eq!(error.kind, CompactionFailureKind::Execution(Error::Frames));
        assert!(error.stats.collections > 0, "frame limit {frames}");
        valid_index(&mut new);
    }
}

#[test]
fn zero_budget_axis_and_rejected_frames_do_not_start_collection() {
    let mut ar = Reduction::<128>::new();
    let object = atom(&mut ar, 42);
    let formula = axis(&mut ar, 0);
    assert!(ar.limit_allocations(ar.count()));
    let mut cap = limits();
    cap.max_collection_work = 0;
    let halted = reduce_compacting_cached(&mut ar, object, formula, 0, cap).unwrap();
    assert!(matches!(halted.outcome, Outcome::Halt(0)));
    assert_eq!(halted.stats.collections, 0);
    cap.max_frames = 0;
    assert_eq!(
        reduce_compacting_cached(&mut ar, object, formula, 100, cap)
            .unwrap_err()
            .kind,
        CompactionFailureKind::Execution(Error::Frames)
    );
}

#[test]
fn invalid_external_operands_preserve_unavailable_without_claiming_allocation_rejection() {
    for tag in [0, 3, 9, 15] {
        for full in [false, true] {
            let mut ar = Reduction::<128>::new();
            let identity = axis(&mut ar, 1);
            let q = quote(&mut ar, 7);
            let formula = match tag {
                0 => axis(&mut ar, 0),
                15 => op(&mut ar, 15, identity),
                _ => binary(&mut ar, tag, identity, q),
            };
            let loaded = ar.count();
            if full {
                assert!(ar.limit_allocations(loaded));
            }
            let mut cap = limits();
            cap.max_total_allocations = u64::from(loaded);
            cap.max_collection_work = 0;
            let expected = reduce_cached(
                &mut ar,
                crate::NIL,
                formula,
                1000,
                Limits { max_frames: 64 },
            )
            .unwrap();
            assert!(matches!(
                expected.outcome,
                Outcome::Error(ErrorKind::Unavailable)
            ));
            let actual = reduce_compacting_cached(&mut ar, crate::NIL, formula, 1000, cap).unwrap();
            assert!(matches!(
                actual.outcome,
                Outcome::Error(ErrorKind::Unavailable)
            ));
            assert_eq!(actual.stats.total_allocations, u64::from(loaded));
            assert_eq!(actual.stats.collections, 0);
            assert_eq!(ar.count(), loaded);
        }
    }
}
