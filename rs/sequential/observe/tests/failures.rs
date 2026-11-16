use super::*;
use std::cell::Cell;

fn completed(events: &[Event]) -> bool {
    events.iter().any(|e| matches!(e, Event::Completed { .. }))
}

#[test]
fn exact_capture_caps_pass_and_one_less_rejects_without_completion() {
    let mut ar = Reduction::<256>::new();
    let (object, formula, _) = dynamic(&mut ar, 0);
    let (baseline, _) = run(&mut ar, object, formula, 8);
    let exact = CaptureLimits {
        max_events: baseline.capture.events,
        max_bytes: baseline.capture.bytes,
        max_work: baseline.capture.work,
    };
    for which in 0..4 {
        let mut ar = Reduction::<256>::new();
        let (object, formula, _) = dynamic(&mut ar, 0);
        let mut cap = exact;
        match which {
            0 => {}
            1 => cap.max_events -= 1,
            2 => cap.max_bytes -= 1,
            _ => cap.max_work -= 1,
        }
        let mut sink = Collect::default();
        let result = reduce_compacting_observed_controlled(
            &mut ar,
            object,
            formula,
            8,
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
            assert!(failure.capture.events <= cap.max_events);
            assert!(failure.capture.bytes <= cap.max_bytes);
            assert!(failure.capture.work <= cap.max_work);
        }
        assert_eq!(ar.allocation_limit(), 192);
    }
}

#[test]
fn sink_failure_at_every_delivery_leaves_only_accepted_prefix() {
    struct Reject {
        after: usize,
        events: Vec<Event>,
    }
    impl Observer for Reject {
        type Error = &'static str;
        fn record(&mut self, event: Event) -> Result<(), Self::Error> {
            if self.events.len() == self.after {
                return Err("sink full");
            }
            self.events.push(event);
            Ok(())
        }
    }
    let mut ar = Reduction::<256>::new();
    let (object, formula, _) = dynamic(&mut ar, 0);
    let (_, baseline) = run(&mut ar, object, formula, 8);
    for after in 0..baseline.0.len() {
        let mut ar = Reduction::<256>::new();
        let (object, formula, _) = dynamic(&mut ar, 0);
        let mut sink = Reject {
            after,
            events: Vec::new(),
        };
        let failure = reduce_compacting_observed_controlled(
            &mut ar,
            object,
            formula,
            8,
            limits(),
            caps(),
            &mut sink,
            &mut || false,
        )
        .unwrap_err();
        assert!(matches!(
            failure.kind,
            ObservedFailureKind::Capture(CaptureFailure::Sink("sink full"))
        ));
        assert_eq!(sink.events, baseline.0[..after]);
        assert_eq!(failure.capture.events, after as u64 + 1);
        assert!(!completed(&sink.events));
        assert_eq!(ar.allocation_limit(), 192);
    }
}

#[test]
fn cancellation_after_terminal_transition_prevents_completed_delivery() {
    struct Cancel<'a> {
        flag: &'a Cell<bool>,
        events: Vec<Event>,
    }
    impl Observer for Cancel<'_> {
        type Error = ();
        fn record(&mut self, event: Event) -> Result<(), ()> {
            if let Event::Transition(t) = event {
                if t.depth_before == 0 && matches!(t.before, LogicalAction::Return { .. }) {
                    self.flag.set(true);
                }
            }
            self.events.push(event);
            Ok(())
        }
    }
    let mut ar = Reduction::<256>::new();
    let (object, formula, _) = dynamic(&mut ar, 0);
    let flag = Cell::new(false);
    let mut sink = Cancel {
        flag: &flag,
        events: Vec::new(),
    };
    let failure = reduce_compacting_observed_controlled(
        &mut ar,
        object,
        formula,
        8,
        limits(),
        caps(),
        &mut sink,
        &mut || flag.get(),
    )
    .unwrap_err();
    assert!(matches!(
        failure.kind,
        ObservedFailureKind::Capture(CaptureFailure::Cancelled)
    ));
    assert!(flag.get());
    assert!(!completed(&sink.events));
    assert!(matches!(sink.events.last(), Some(Event::Transition(_))));
}

#[test]
fn zero_caps_stop_before_export_and_initial_export_is_cancellable() {
    for cap in [
        CaptureLimits {
            max_events: 0,
            ..caps()
        },
        CaptureLimits {
            max_bytes: 0,
            ..caps()
        },
        CaptureLimits {
            max_work: 0,
            ..caps()
        },
    ] {
        let mut ar = Reduction::<256>::new();
        let (object, formula, _) = dynamic(&mut ar, 0);
        let loaded = ar.count();
        let mut sink = Collect::default();
        assert!(
            reduce_compacting_observed_controlled(
                &mut ar,
                object,
                formula,
                8,
                limits(),
                cap,
                &mut sink,
                &mut || false
            )
            .is_err()
        );
        assert!(sink.0.is_empty());
        assert_eq!(ar.count(), loaded);
    }
    let mut ar = Reduction::<256>::new();
    let (object, formula, _) = dynamic(&mut ar, 0);
    let loaded = ar.count();
    let mut sink = Collect::default();
    let mut polls = 0;
    let failure = reduce_compacting_observed_controlled(
        &mut ar,
        object,
        formula,
        8,
        limits(),
        caps(),
        &mut sink,
        &mut || {
            polls += 1;
            polls == 8
        },
    )
    .unwrap_err();
    assert!(matches!(
        failure.kind,
        ObservedFailureKind::Capture(CaptureFailure::Cancelled)
    ));
    assert_eq!(ar.count(), loaded);
    assert!(!completed(&sink.0));
    assert!(
        sink.0
            .iter()
            .all(|e| matches!(e, Event::Begin { .. } | Event::Node(_)))
    );
}

#[test]
fn execution_failures_keep_their_kinds_and_never_complete_capture() {
    for which in 0..4 {
        let mut ar = Reduction::<256>::new();
        let (object, mut formula) = loop_core(&mut ar, 100);
        assert!(ar.limit_allocations(100));
        let mut cap = limits();
        let expected = match which {
            0 => {
                cap.max_frames = 3;
                CompactionFailureKind::Execution(sequential::Error::Frames)
            }
            1 => {
                cap.max_total_allocations = u64::from(ar.count());
                CompactionFailureKind::TotalAllocations
            }
            2 => {
                cap.max_collection_work = 0;
                CompactionFailureKind::CollectionWork
            }
            _ => {
                formula = op(&mut ar, 16, object);
                CompactionFailureKind::Execution(sequential::Error::UnsupportedService(16))
            }
        };
        let mut sink = Collect::default();
        let failure = reduce_compacting_observed_controlled(
            &mut ar,
            object,
            formula,
            5000,
            cap,
            caps(),
            &mut sink,
            &mut || false,
        )
        .unwrap_err();
        assert!(matches!(failure.kind, ObservedFailureKind::Execution(kind) if kind == expected));
        assert!(!completed(&sink.0));
        assert_eq!(ar.allocation_limit(), 100);
    }
}

#[test]
fn invalid_export_references_fail_closed_without_changing_legacy_behavior() {
    let mut ar = Reduction::<128>::new();
    let object = atom(&mut ar, 0);
    let formula = quote(&mut ar, 7);
    let invalid = ar.count();
    ar.alloc_raw(crate::data::DataEntry {
        inner: crate::data::Data::Pair {
            left: invalid,
            right: object,
        },
        hash: *ar.digest(object).unwrap(),
        bound: Cost::Exact(0),
    })
    .unwrap();
    let mut sink = Collect::default();
    let failure = reduce_compacting_observed_controlled(
        &mut ar,
        object,
        formula,
        1,
        limits(),
        caps(),
        &mut sink,
        &mut || false,
    )
    .unwrap_err();
    assert!(matches!(
        failure.kind,
        ObservedFailureKind::Capture(CaptureFailure::InvalidArena)
    ));
    assert!(!completed(&sink.0));
    // The opt-in exporter detects even unrelated malformed pinned data.
    assert!(matches!(
        sequential::reduce_compacting_cached(&mut ar, object, formula, 1, limits())
            .unwrap()
            .outcome,
        Outcome::Ok(_, 0)
    ));
}
