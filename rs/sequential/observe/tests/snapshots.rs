use super::*;

mod failures;

#[derive(Default)]
struct CollectV2(Vec<EventV2>);
impl ObserverV2 for CollectV2 {
    type Error = &'static str;
    fn record(&mut self, event: EventV2) -> Result<(), Self::Error> {
        self.0.push(event);
        Ok(())
    }
}

fn fixture() -> (Reduction<256>, Order, Order) {
    let mut ar = Reduction::new();
    let (object, formula) = loop_core(&mut ar, 40);
    assert!(ar.limit_allocations(100));
    (ar, object, formula)
}

fn run_v2(cap: CaptureLimits) -> (ObservedExecution, CollectV2) {
    let (mut ar, object, formula) = fixture();
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
    )
    .unwrap();
    (result, sink)
}

/// Validates epoch ordering and metadata, then removes snapshots for comparison
/// with the existing version-1 integrity oracle. This remains a test helper.
fn normalized(events: &[EventV2]) -> (Collect, u64, u64) {
    let mut legacy = Collect::default();
    let mut definitions = BTreeMap::new();
    let mut previous = BTreeMap::new();
    let mut replay = Reduction::<256>::new();
    let mut pending = 0;
    let mut sequence = 0;
    let mut resets = 0;
    let mut snapshots = 0;
    for event in events {
        match *event {
            EventV2::ArenaReset {
                next_sequence,
                live_nodes,
            } => {
                assert_eq!(pending, 0);
                assert_eq!(next_sequence, sequence);
                assert!(live_nodes > 0 && live_nodes <= 100);
                previous = core::mem::take(&mut definitions);
                replay = Reduction::new();
                pending = live_nodes;
                resets += 1;
            }
            EventV2::Event(mut e) => {
                if pending > 0 {
                    assert!(matches!(e, Event::Node(_)));
                }
                if let Event::Node(node) = e {
                    let id = match node.value {
                        NodeValue::Atom(v) => {
                            let field = nebu::Goldilocks::new(v);
                            assert_eq!(v, field.canonicalize().as_u64());
                            assert_eq!(hash_atom(field).map(|v| v.as_u64()), node.particle);
                            replay.atom(field).unwrap()
                        }
                        NodeValue::Pair { left, right } => {
                            let (a, _) = definitions[&left];
                            let (b, _) = definitions[&right];
                            replay.pair(a, b).unwrap()
                        }
                    };
                    assert_eq!(particle(&replay, id), node.particle);
                    assert_eq!(crate::bound(&replay, id), node.bound);
                    definitions.insert(node.particle, (id, node));
                    if pending > 0 {
                        assert_eq!(previous[&node.particle].1, node);
                        pending -= 1;
                        snapshots += 1;
                        continue;
                    }
                }
                if let Event::Begin {
                    ref mut version, ..
                } = e
                {
                    assert_eq!(*version, 2);
                    *version = 1;
                }
                if let Event::Transition(t) = e {
                    assert_eq!(t.sequence, sequence);
                    sequence += 1;
                    for action in [t.before, t.after] {
                        match action {
                            LogicalAction::Enter {
                                object, formula, ..
                            } => {
                                assert!(definitions.contains_key(&object));
                                assert!(definitions.contains_key(&formula));
                            }
                            LogicalAction::Return { value, .. } => {
                                assert!(definitions.contains_key(&value));
                            }
                            _ => {}
                        }
                    }
                }
                legacy.0.push(e);
            }
        }
    }
    assert_eq!(pending, 0);
    (legacy, resets, snapshots)
}

#[test]
fn snapshots_preserve_runtime_stats_logical_steps_and_version_one_bytes() {
    let (observed, events) = run_v2(caps());
    let (mut ar, object, formula) = fixture();
    let mut v1 = Collect::default();
    let baseline = reduce_compacting_observed_controlled(
        &mut ar,
        object,
        formula,
        1000,
        limits(),
        caps(),
        &mut v1,
        &mut || false,
    )
    .unwrap();
    let (mut plain_ar, object, formula) = fixture();
    let mut plain_polls = 0;
    let plain = sequential::reduce_compacting_cached_controlled(
        &mut plain_ar,
        object,
        formula,
        1000,
        limits(),
        &mut || {
            plain_polls += 1;
            false
        },
    )
    .unwrap();
    assert_eq!(observed.execution.stats, baseline.execution.stats);
    assert_eq!(observed.execution.stats, plain.stats);
    assert_eq!(
        plain_polls,
        plain.stats.evaluator_checkpoints + plain.stats.collection_checkpoints
    );
    assert_eq!(observed.execution.peak_frames, plain.peak_frames);
    assert_eq!(
        std::format!("{:?}", observed.execution.outcome),
        std::format!("{:?}", plain.outcome)
    );
    let (normalized, resets, snapshots) = normalized(&events.0);
    assert!(resets > 1);
    assert_eq!(resets, plain.stats.collections);
    assert_eq!(normalized.0, v1.0);
    assert_eq!(checked(&normalized), checked(&v1));
    assert_eq!(
        observed.capture.events,
        baseline.capture.events + resets + snapshots
    );
    assert_eq!(
        observed.capture.work,
        baseline.capture.work + resets + snapshots * 2
    );
    assert_eq!(
        observed.capture.bytes,
        events
            .0
            .iter()
            .map(|e| e.encode().as_bytes().len() as u64)
            .sum::<u64>()
    );
    assert_eq!(ar.count(), plain_ar.count());
    for id in 0..ar.count() {
        assert_eq!(ar.digest(id), plain_ar.digest(id));
    }
}

#[test]
fn version_two_without_collection_only_changes_begin_version() {
    let mut old = Reduction::<256>::new();
    let (object, formula, _) = dynamic(&mut old, 0);
    let (baseline, original) = run(&mut old, object, formula, 8);
    let mut ar = Reduction::<256>::new();
    assert_eq!(dynamic(&mut ar, 0).0, object);
    let mut sink = CollectV2::default();
    let observed = reduce_compacting_observed_v2_controlled(
        &mut ar,
        object,
        formula,
        8,
        limits(),
        caps(),
        &mut sink,
        &mut || false,
    )
    .unwrap();
    assert_eq!(observed.capture, baseline.capture);
    assert_eq!(observed.execution.stats, baseline.execution.stats);
    assert_eq!(normalized(&sink.0).0.0, original.0);
}

#[test]
fn version_two_reset_has_a_distinct_tag_without_wrapping_ordinary_bytes() {
    let reset = EventV2::ArenaReset {
        next_sequence: u64::MAX,
        live_nodes: u32::MAX,
    };
    let expected: Vec<u8> = [4, u64::MAX, u64::from(u32::MAX)]
        .into_iter()
        .flat_map(u64::to_le_bytes)
        .collect();
    assert_eq!(reset.encode().as_bytes(), expected);
    let (_, sink) = run_v2(caps());
    for event in sink.0 {
        if let EventV2::Event(inner) = event {
            assert_eq!(event.encode().as_bytes(), inner.encode().as_bytes());
        }
    }
}

#[test]
fn version_one_gc_stream_matches_pre_extension_golden() {
    // Captured independently from nox172811b with Rust1.89. See audit/arena-snapshots.
    let (mut ar, object, formula) = fixture();
    let mut sink = Collect::default();
    let result = reduce_compacting_observed_controlled(
        &mut ar,
        object,
        formula,
        1000,
        limits(),
        caps(),
        &mut sink,
        &mut || false,
    )
    .unwrap();
    let mut hash = hemera::Hasher::new();
    for event in sink.0 {
        hash.update(event.encode().as_bytes());
    }
    assert_eq!(
        *hash.finalize().as_bytes(),
        [
            247, 224, 8, 81, 140, 212, 21, 213, 119, 4, 247, 94, 124, 162, 147, 214, 209, 213, 65,
            23, 165, 192, 251, 229, 103, 82, 241, 203, 84, 99, 27, 97,
        ]
    );
    assert_eq!(
        result.capture,
        CaptureStats {
            events: 1383,
            bytes: 341960,
            work: 5183
        }
    );
    assert_eq!(
        result.execution.stats,
        CompactionStats {
            pinned_nodes: 32,
            resident_nodes: 43,
            peak_resident_nodes: 100,
            total_allocations: 171,
            reclaimed_nodes: 128,
            collections: 2,
            collection_work: 133415,
            scratch_bytes: 1424,
            evaluator_checkpoints: 1211,
            collection_checkpoints: 6,
        }
    );
}

#[test]
fn pending_parent_inputs_can_disappear_from_later_arena_epochs() {
    let (_, events) = run_v2(caps());
    let mut active = Vec::new();
    let mut epoch = std::collections::BTreeSet::new();
    let mut pending = 0;
    let mut missing_parent = false;
    for event in events.0 {
        match event {
            EventV2::ArenaReset { live_nodes, .. } => {
                epoch.clear();
                pending = live_nodes;
            }
            EventV2::Event(Event::Node(node)) => {
                epoch.insert(node.particle);
                if pending > 0 {
                    pending -= 1;
                    if pending == 0 {
                        missing_parent |= active.iter().any(|(object, formula)| {
                            !epoch.contains(object) || !epoch.contains(formula)
                        });
                    }
                }
            }
            EventV2::Event(Event::Transition(t)) => match t.before {
                LogicalAction::Enter {
                    object, formula, ..
                } => active.push((object, formula)),
                LogicalAction::Return { .. } => {
                    active.pop().unwrap();
                }
                _ => panic!(),
            },
            _ => {}
        }
    }
    assert!(active.is_empty());
    assert!(missing_parent);
}
