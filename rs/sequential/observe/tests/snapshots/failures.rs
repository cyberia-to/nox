use super::*;
use std::cell::Cell;

fn completed(events: &[EventV2]) -> bool {
    events
        .iter()
        .any(|e| matches!(e, EventV2::Event(Event::Completed { .. })))
}

fn intact_index(ar: &mut Reduction<256>) {
    let count = ar.count();
    for id in 0..count {
        let same = match ar.get(id).unwrap().inner {
            crate::data::Data::Atom { value } => ar.atom(value),
            crate::data::Data::Pair { left, right } => ar.pair(left, right),
        };
        assert_eq!(same, Some(id));
    }
    assert_eq!(ar.count(), count);
}

#[test]
fn snapshot_caps_admit_exact_totals_and_reject_one_less_without_completion() {
    let (baseline, _) = run_v2(caps());
    for which in 0..4 {
        let (mut ar, object, formula) = fixture();
        let mut cap = CaptureLimits {
            max_events: baseline.capture.events,
            max_bytes: baseline.capture.bytes,
            max_work: baseline.capture.work,
        };
        match which {
            0 => {}
            1 => cap.max_events -= 1,
            2 => cap.max_bytes -= 1,
            _ => cap.max_work -= 1,
        }
        let mut sink = CollectV2::default();
        let result = reduce_compacting_observed_v2_controlled(
            &mut ar,
            object,
            formula,
            1000,
            limits(),
            cap,
            &mut sink,
            &mut || false,
        );
        if which == 0 {
            assert_eq!(result.unwrap().capture, baseline.capture);
            assert!(completed(&sink.0));
        } else {
            let failure = result.unwrap_err();
            assert!(matches!(
                (which, failure.kind),
                (1, ObservedFailureKind::Capture(CaptureFailure::Events))
                    | (2, ObservedFailureKind::Capture(CaptureFailure::Bytes))
                    | (3, ObservedFailureKind::Capture(CaptureFailure::Work))
            ));
            assert!(!completed(&sink.0));
        }
        assert_eq!(ar.allocation_limit(), 100);
    }
}

#[test]
fn caps_and_cancellation_interrupt_snapshot_without_a_following_transition() {
    let (_, baseline) = run_v2(caps());
    let reset = baseline
        .0
        .iter()
        .position(|e| matches!(e, EventV2::ArenaReset { .. }))
        .unwrap();
    let mut work_at_prefix = 0;
    for which in 0..4 {
        let (mut ar, object, formula) = fixture();
        let mut cap = caps();
        if which == 0 {
            cap.max_events = reset as u64 + 2;
        }
        if which == 1 {
            cap.max_bytes = baseline.0[..reset + 2]
                .iter()
                .map(|e| e.encode().as_bytes().len() as u64)
                .sum();
        }
        if which == 3 {
            cap.max_work = work_at_prefix;
        }
        struct CancelAfter<'a> {
            events: Vec<EventV2>,
            flag: &'a Cell<bool>,
            after: usize,
        }
        impl ObserverV2 for CancelAfter<'_> {
            type Error = &'static str;
            fn record(&mut self, event: EventV2) -> Result<(), Self::Error> {
                self.events.push(event);
                if self.events.len() == self.after {
                    self.flag.set(true);
                }
                Ok(())
            }
        }
        let flag = Cell::new(false);
        let mut sink = CancelAfter {
            events: Vec::new(),
            flag: &flag,
            after: if which == 2 { reset + 2 } else { usize::MAX },
        };
        let failure = reduce_compacting_observed_v2_controlled(
            &mut ar,
            object,
            formula,
            1000,
            limits(),
            cap,
            &mut sink,
            &mut || flag.get(),
        )
        .unwrap_err();
        assert!(matches!(
            (which, failure.kind),
            (0, ObservedFailureKind::Capture(CaptureFailure::Events))
                | (1, ObservedFailureKind::Capture(CaptureFailure::Bytes))
                | (2, ObservedFailureKind::Capture(CaptureFailure::Cancelled))
                | (3, ObservedFailureKind::Capture(CaptureFailure::Work))
        ));
        if which == 0 {
            // Failed next-node export admitted its node and delivery work.
            work_at_prefix = failure.capture.work - 2;
        }
        assert_eq!(sink.events, baseline.0[..reset + 2]);
        assert!(!completed(&sink.events));
        assert_eq!(failure.stats.collections, 1);
        // Collection already committed; the arena is usable after capture fails.
        intact_index(&mut ar);
        assert_eq!(ar.allocation_limit(), 100);
    }
}

#[test]
fn cancellation_after_collection_commit_exports_no_reset_and_leaves_valid_arena() {
    struct Polls<'a> {
        polls: &'a Cell<u64>,
        first_reset: Option<u64>,
    }
    impl ObserverV2 for Polls<'_> {
        type Error = ();
        fn record(&mut self, event: EventV2) -> Result<(), ()> {
            if matches!(event, EventV2::ArenaReset { .. }) && self.first_reset.is_none() {
                self.first_reset = Some(self.polls.get());
            }
            Ok(())
        }
    }
    let polls = Cell::new(0);
    let mut sink = Polls {
        polls: &polls,
        first_reset: None,
    };
    let (mut ar, object, formula) = fixture();
    reduce_compacting_observed_v2_controlled(
        &mut ar,
        object,
        formula,
        1000,
        limits(),
        caps(),
        &mut sink,
        &mut || {
            polls.set(polls.get() + 1);
            false
        },
    )
    .unwrap();
    // The poll immediately before reset delivery is collection's post-commit
    // checkpoint. No capture work occurs between those two polls.
    let cancel_at = sink.first_reset.unwrap() - 1;
    let (mut ar, object, formula) = fixture();
    let mut sink = CollectV2::default();
    let mut polls = 0;
    let failure = reduce_compacting_observed_v2_controlled(
        &mut ar,
        object,
        formula,
        1000,
        limits(),
        caps(),
        &mut sink,
        &mut || {
            polls += 1;
            polls == cancel_at
        },
    )
    .unwrap_err();
    assert!(matches!(
        failure.kind,
        ObservedFailureKind::Execution(CompactionFailureKind::Execution(
            sequential::Error::Cancelled
        ))
    ));
    assert_eq!(failure.stats.collections, 1);
    assert!(
        !sink
            .0
            .iter()
            .any(|e| matches!(e, EventV2::ArenaReset { .. }))
    );
    assert!(!completed(&sink.0));
    intact_index(&mut ar);
    assert_eq!(ar.allocation_limit(), 100);
}

#[test]
fn sink_failure_at_reset_or_each_snapshot_node_keeps_only_accepted_prefix() {
    let (_, baseline) = run_v2(caps());
    let reset = baseline
        .0
        .iter()
        .position(|e| matches!(e, EventV2::ArenaReset { .. }))
        .unwrap();
    let EventV2::ArenaReset { live_nodes, .. } = baseline.0[reset] else {
        panic!()
    };
    for after in reset..=reset + live_nodes as usize {
        struct Reject {
            events: Vec<EventV2>,
            after: usize,
        }
        impl ObserverV2 for Reject {
            type Error = &'static str;
            fn record(&mut self, event: EventV2) -> Result<(), Self::Error> {
                if self.events.len() == self.after {
                    return Err("full");
                }
                self.events.push(event);
                Ok(())
            }
        }
        let (mut ar, object, formula) = fixture();
        let mut sink = Reject {
            events: Vec::new(),
            after,
        };
        let failure = reduce_compacting_observed_v2_controlled(
            &mut ar,
            object,
            formula,
            1000,
            limits(),
            caps(),
            &mut sink,
            &mut || false,
        )
        .unwrap_err();
        assert!(matches!(
            failure.kind,
            ObservedFailureKind::Capture(CaptureFailure::Sink("full"))
        ));
        assert_eq!(sink.events, baseline.0[..after]);
        assert_eq!(failure.capture.events, after as u64 + 1);
        assert!(!completed(&sink.events));
        assert_eq!(failure.stats.collections, 1);
    }
}

#[test]
fn failed_collection_exports_no_reset() {
    let (mut ar, object, formula) = fixture();
    let mut sink = CollectV2::default();
    let failure = reduce_compacting_observed_v2_controlled(
        &mut ar,
        object,
        formula,
        1000,
        CompactionLimits {
            max_collection_work: 0,
            ..limits()
        },
        caps(),
        &mut sink,
        &mut || false,
    )
    .unwrap_err();
    assert!(matches!(
        failure.kind,
        ObservedFailureKind::Execution(CompactionFailureKind::CollectionWork)
    ));
    assert!(
        !sink
            .0
            .iter()
            .any(|e| matches!(e, EventV2::ArenaReset { .. }))
    );
    assert!(!completed(&sink.0));
}
