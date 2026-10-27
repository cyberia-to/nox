use super::*;
use crate::sequential::compacting::collection::collect;

fn setup() -> (
    Reduction<64>,
    Action,
    Vec<Frame>,
    finalizer_cache::Cache<true>,
    CompactionStats,
) {
    let mut ar = Reduction::<64>::new();
    let a = atom(&mut ar, 7);
    let b = atom(&mut ar, 8);
    atom(&mut ar, 999); // discarded before the surviving pair: forces relocation
    let pair = ar.pair(a, b).unwrap();
    let result = ar.pair(pair, pair).unwrap();
    let dead = atom(&mut ar, 15);
    let mut cache = finalizer_cache::Cache::<true>::new().unwrap();
    cache.finish(3, a, b, 10, || Outcome::Ok(pair, 10));
    cache.finish(5, a, b, 10, || Outcome::Ok(dead, 10));
    let mut row = TraceRow::default();
    row.r[0] = 3;
    row.r[1] = u64::from(dead); // unused at BinaryRight; must not pin garbage
    row.r[2] = u64::from(dead);
    let stack = std::vec![Frame {
        row,
        budget: 10,
        phase: Phase::BinaryRight {
            a: dead,
            b: dead,
            left: a,
            used: 0,
            second: 5,
        }
    }];
    let stats = CompactionStats {
        pinned_nodes: 2,
        resident_nodes: ar.count(),
        peak_resident_nodes: ar.count(),
        total_allocations: u64::from(ar.count()),
        ..CompactionStats::default()
    };
    (
        ar,
        Action::Return(Outcome::Ok(result, 3)),
        stack,
        cache,
        stats,
    )
}

#[test]
fn relocation_preserves_shared_structure_and_flushes_live_and_dead_cache_results() {
    let (mut ar, mut action, mut stack, mut cache, mut stats) = setup();
    let digest = *ar.digest(4).unwrap();
    let bound = crate::bound(&ar, 4);
    let mut scratch = Scratch::new::<64>(ar.allocation_limit()).unwrap();
    collect(
        &mut ar,
        &mut action,
        &mut stack,
        &mut cache,
        &mut scratch,
        1_000_000,
        &mut stats,
        &mut || false,
    )
    .unwrap();
    assert_eq!(ar.count(), 4);
    assert_eq!(stats.reclaimed_nodes, 2);
    assert_eq!(stats.collections, 1);
    assert!(matches!(action, Action::Return(Outcome::Ok(3, 3))));
    assert_eq!(ar.head(3), Some(2));
    assert_eq!(ar.tail(3), Some(2));
    assert_eq!(ar.digest(3), Some(&digest));
    assert_eq!(crate::bound(&ar, 3), bound);
    let mut calls = 0;
    let pair = cache.finish(3, 0, 1, 10, || {
        calls += 1;
        Outcome::Ok(ar.pair(0, 1).unwrap(), 10)
    });
    assert!(matches!(pair, Outcome::Ok(2, 10)));
    let sum = cache.finish(5, 0, 1, 10, || {
        calls += 1;
        Outcome::Ok(atom(&mut ar, 15), 10)
    });
    assert!(matches!(sum, Outcome::Ok(4, 10)));
    assert_eq!(calls, 2);
    valid_index(&mut ar);
}

#[test]
fn cancellation_before_and_after_commit_always_returns_a_valid_arena() {
    for cancel_at in 1..=3 {
        let (mut ar, mut action, mut stack, mut cache, mut stats) = setup();
        let before = snapshot(&ar);
        let mut scratch = Scratch::new::<64>(ar.allocation_limit()).unwrap();
        let mut calls = 0;
        let error = collect(
            &mut ar,
            &mut action,
            &mut stack,
            &mut cache,
            &mut scratch,
            1_000_000,
            &mut stats,
            &mut || {
                calls += 1;
                calls == cancel_at
            },
        )
        .unwrap_err();
        assert_eq!(error, Fault::Cancelled);
        assert_eq!(stats.collection_checkpoints, cancel_at);
        if cancel_at < 3 {
            assert_eq!(snapshot(&ar), before);
            assert_eq!(stats.collections, 0);
        } else {
            assert_eq!(ar.count(), 4);
            assert_eq!(stats.collections, 1);
        }
        assert_eq!(ar.atom_value(0).unwrap().as_u64(), 7);
        assert_eq!(ar.atom_value(1).unwrap().as_u64(), 8);
        valid_index(&mut ar);
    }
}

#[test]
fn commit_work_is_fully_admitted_before_mutation() {
    let (mut ar, mut action, mut stack, mut cache, mut stats) = setup();
    let mut scratch = Scratch::new::<64>(ar.allocation_limit()).unwrap();
    collect(
        &mut ar,
        &mut action,
        &mut stack,
        &mut cache,
        &mut scratch,
        1_000_000,
        &mut stats,
        &mut || false,
    )
    .unwrap();
    let needed = stats.collection_work;
    for cap in [0, 100, needed - 1] {
        let (mut ar, mut action, mut stack, mut cache, mut stats) = setup();
        let before = snapshot(&ar);
        let mut scratch = Scratch::new::<64>(ar.allocation_limit()).unwrap();
        assert_eq!(
            collect(
                &mut ar,
                &mut action,
                &mut stack,
                &mut cache,
                &mut scratch,
                cap,
                &mut stats,
                &mut || false
            )
            .unwrap_err(),
            Fault::Work
        );
        assert_eq!(snapshot(&ar), before);
        assert_eq!(stats.collections, 0);
        assert!(stats.collection_work <= cap);
        valid_index(&mut ar);
    }
}

#[test]
fn planning_polls_during_bounded_metadata_scans() {
    let mut ar = Reduction::<8192>::try_new_boxed().unwrap();
    let a = atom(&mut ar, 7);
    let before = snapshot(&ar);
    let mut action = Action::Return(Outcome::Ok(a, 1));
    let mut scratch = Scratch::new::<8192>(ar.allocation_limit()).unwrap();
    let mut cache = finalizer_cache::Cache::<true>::new().unwrap();
    let mut stats = CompactionStats {
        pinned_nodes: 1,
        ..CompactionStats::default()
    };
    let mut calls = 0;
    assert_eq!(
        collect(
            &mut ar,
            &mut action,
            &mut [],
            &mut cache,
            &mut scratch,
            1_000_000,
            &mut stats,
            &mut || {
                calls += 1;
                calls == 2
            }
        )
        .unwrap_err(),
        Fault::Cancelled
    );
    assert_eq!(stats.collection_work, 4096);
    assert_eq!(stats.collections, 0);
    assert_eq!(snapshot(&ar), before);
    valid_index(&mut ar);
}

#[test]
fn invalid_roots_and_unindexed_raw_entries_reject_before_mutation() {
    for raw in [false, true] {
        let (mut ar, mut action, mut stack, mut cache, mut stats) = setup();
        if raw {
            let mut entry = *ar.get(0).unwrap();
            entry.inner = Data::Pair {
                left: ar.count(),
                right: 0,
            };
            ar.alloc_raw(entry).unwrap();
        } else {
            action = Action::Return(Outcome::Ok(crate::NIL, 0));
        }
        let before = snapshot(&ar);
        let mut scratch = Scratch::new::<64>(ar.allocation_limit()).unwrap();
        assert_eq!(
            collect(
                &mut ar,
                &mut action,
                &mut stack,
                &mut cache,
                &mut scratch,
                1_000_000,
                &mut stats,
                &mut || false
            )
            .unwrap_err(),
            Fault::InvalidArena
        );
        assert_eq!(snapshot(&ar), before);
    }
}

#[test]
fn phase_root_matrix_remaps_only_future_noun_reads() {
    let reservation = Reservation {
        parent: 100,
        child: 10,
    };
    for (phase, expected) in [
        (Phase::Unary(reservation), std::vec![]),
        (Phase::BranchChosen(reservation), std::vec![]),
        (Phase::Compose, std::vec![]),
        (
            Phase::BinaryLeft {
                a: 21,
                b: 22,
                first: 10,
                second: Some(10),
            },
            std::vec![20, 22],
        ),
        (
            Phase::BinaryRight {
                a: 21,
                b: 22,
                left: 23,
                used: 4,
                second: 10,
            },
            std::vec![23],
        ),
        (
            Phase::BranchTest {
                yes: 24,
                no: 25,
                reservation,
            },
            std::vec![20, 24, 25],
        ),
    ] {
        let mut row = TraceRow::default();
        row.r[1] = 20;
        row.r[2] = 999_999;
        let mut frame = Frame {
            row,
            budget: 777,
            phase,
        };
        let mut seen = Vec::new();
        roots::frame(&mut frame, &mut |id| {
            seen.push(id);
            id + 100
        });
        assert_eq!(seen, expected);
        assert_eq!(frame.budget, 777);
        assert_eq!(frame.row.r[2], 999_999);
        let mut after = Vec::new();
        roots::frame(&mut frame, &mut |id| {
            after.push(id);
            id
        });
        assert_eq!(
            after,
            expected.into_iter().map(|id| id + 100).collect::<Vec<_>>()
        );
    }
    for (mut action, expected) in [
        (
            Action::Enter {
                object: 20,
                formula: 21,
                budget: 100,
            },
            std::vec![20, 21],
        ),
        (Action::Return(Outcome::Ok(22, 30)), std::vec![22]),
        (Action::Return(Outcome::Halt(55)), std::vec![]),
        (
            Action::Return(Outcome::Error(ErrorKind::Unavailable)),
            std::vec![],
        ),
    ] {
        let mut seen = Vec::new();
        roots::action(&mut action, &mut |id| {
            seen.push(id);
            id + 100
        });
        assert_eq!(seen, expected);
    }
}

#[test]
fn returning_finalizer_can_collect_at_the_frame_capacity_boundary() {
    let (mut ar, mut action, mut stack, mut cache, mut stats) = setup();
    assert!(ar.limit_allocations(ar.count()));
    let max_frames = stack.len() as u32;
    // Return consumes its frame; it does not require a new frame admission.
    assert_eq!(
        roots::allocation_headroom(&ar, &action, &stack, max_frames),
        1
    );
    let mut scratch = Scratch::new::<64>(ar.allocation_limit()).unwrap();
    collect(
        &mut ar,
        &mut action,
        &mut stack,
        &mut cache,
        &mut scratch,
        1_000_000,
        &mut stats,
        &mut || false,
    )
    .unwrap();
    let next = step(&mut ar, action, &mut stack, &mut cache, max_frames, &mut 1).unwrap();
    assert!(matches!(
        next,
        Step::Next(Action::Return(Outcome::Ok(_, 8)))
    ));
    assert!(stack.is_empty());
    assert_eq!(stats.collections, 1);
    valid_index(&mut ar);
}
