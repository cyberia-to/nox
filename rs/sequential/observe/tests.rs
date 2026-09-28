extern crate std;
use super::*;
use crate::sequential::{
    self,
    tests::{atom, axis, binary, loop_core, op, quote},
};
use crate::{
    Outcome,
    data::{Cost, hash_atom, hash_pair},
};
use std::{collections::BTreeMap, vec::Vec};

mod differential;
mod failures;
mod wire;

#[derive(Default)]
struct Collect(Vec<Event>);
impl Observer for Collect {
    type Error = &'static str;
    fn record(&mut self, event: Event) -> Result<(), Self::Error> {
        self.0.push(event);
        Ok(())
    }
}
fn limits() -> CompactionLimits {
    CompactionLimits {
        max_frames: 4096,
        max_total_allocations: 100_000,
        max_collection_work: 100_000_000,
    }
}
fn caps() -> CaptureLimits {
    CaptureLimits {
        max_events: 1_000_000,
        max_bytes: 500_000_000,
        max_work: 4_000_000,
    }
}
fn run<const N: usize>(
    ar: &mut Reduction<N>,
    object: Order,
    formula: Order,
    budget: u64,
) -> (ObservedExecution, Collect) {
    let mut sink = Collect::default();
    let run = reduce_compacting_observed_controlled(
        ar,
        object,
        formula,
        budget,
        limits(),
        caps(),
        &mut sink,
        &mut || false,
    )
    .unwrap();
    assert_eq!(run.capture.events, sink.0.len() as u64);
    assert_eq!(
        run.capture.bytes,
        sink.0
            .iter()
            .map(|e| e.encode().as_bytes().len() as u64)
            .sum()
    );
    (run, sink)
}
fn particle<const N: usize>(ar: &Reduction<N>, id: Order) -> Particle {
    ar.digest(id).unwrap().map(|v| v.as_u64())
}

/// Independent stream integrity check: definitions, cost metadata, stack and
/// action continuity. This test helper supplies no cryptographic proof claim.
fn checked(sink: &Collect) -> Vec<Transition> {
    let mut definitions = BTreeMap::new();
    let mut replay = Reduction::<32768>::try_new_boxed().unwrap();
    let mut stack = Vec::new();
    let mut transitions = Vec::new();
    let mut previous = None;
    let mut nodes = 0;
    let mut initial_nodes = 0;
    let mut done = false;
    for (i, event) in sink.0.iter().enumerate() {
        assert!(!done);
        match *event {
            Event::Begin {
                version,
                initial,
                initial_nodes: count,
                ..
            } => {
                assert_eq!(i, 0);
                assert_eq!(version, 1);
                previous = Some(initial);
                initial_nodes = count;
            }
            Event::Node(node) => {
                let r = match node.value {
                    NodeValue::Atom(v) => {
                        let field = nebu::Goldilocks::new(v);
                        assert_eq!(v, field.canonicalize().as_u64());
                        assert_eq!(hash_atom(field).map(|v| v.as_u64()), node.particle);
                        replay.atom(field).unwrap()
                    }
                    NodeValue::Pair { left, right } => {
                        let (a, b) = (definitions[&left], definitions[&right]);
                        assert_eq!(
                            hash_pair(replay.digest(a).unwrap(), replay.digest(b).unwrap())
                                .map(|v| v.as_u64()),
                            node.particle
                        );
                        replay.pair(a, b).unwrap()
                    }
                };
                assert_eq!(crate::bound(&replay, r), node.bound);
                if let Some(old) = definitions.insert(node.particle, r) {
                    assert_eq!(old, r);
                }
                nodes += 1;
            }
            Event::Transition(t) => {
                assert_eq!(t.sequence, transitions.len() as u64);
                assert_eq!(Some(t.before), previous);
                assert_eq!(
                    nodes,
                    t.fresh_nodes + if t.sequence == 0 { initial_nodes } else { 0 }
                );
                nodes = 0;
                for action in [t.before, t.after] {
                    match action {
                        LogicalAction::Enter {
                            object, formula, ..
                        } => {
                            assert!(definitions.contains_key(&object));
                            assert!(definitions.contains_key(&formula));
                        }
                        LogicalAction::Return { value, .. } => {
                            assert!(definitions.contains_key(&value))
                        }
                        _ => {}
                    }
                }
                assert_eq!(stack.len(), t.depth_before as usize);
                if let Some(frame) = t.popped {
                    assert_eq!(stack.pop(), Some(frame));
                }
                if let Some(frame) = t.pushed {
                    stack.push(frame);
                }
                assert_eq!(stack.len(), t.depth_after as usize);
                previous = Some(t.after);
                // Recreated nodes after GC change only physical allocation history.
                transitions.push(Transition {
                    fresh_nodes: 0,
                    ..t
                });
            }
            Event::Completed {
                steps,
                value,
                remaining,
            } => {
                assert_eq!(steps, transitions.len() as u64);
                assert_eq!(previous, Some(LogicalAction::Return { value, remaining }));
                let last = transitions.last().unwrap();
                assert_eq!(last.before, last.after);
                assert_eq!((last.depth_before, last.depth_after), (0, 0));
                assert!(last.popped.is_none() && last.pushed.is_none());
                assert!(stack.is_empty());
                assert_eq!(nodes, 0);
                done = true;
            }
        }
    }
    transitions
}

fn dynamic<const N: usize>(ar: &mut Reduction<N>, selector: u64) -> (Order, Order, Order) {
    let (a, b, c) = (atom(ar, 7), atom(ar, 8), atom(ar, 9));
    let ab = ar.pair(a, b).unwrap();
    let bc = ar.pair(b, c).unwrap();
    let yes = ar.pair(ab, c).unwrap();
    let no = ar.pair(a, bc).unwrap();
    let qyes = op(ar, 1, yes);
    let qno = op(ar, 1, no);
    let alternatives = ar.pair(qyes, qno).unwrap();
    let test = axis(ar, 2);
    let branch = binary(ar, 4, test, alternatives);
    let one = quote(ar, 1);
    let producer = binary(ar, 3, one, branch);
    let zero = quote(ar, 0);
    let formula = binary(ar, 2, zero, producer);
    let selector_atom = atom(ar, selector);
    let zero = atom(ar, 0);
    (
        ar.pair(selector_atom, zero).unwrap(),
        formula,
        if selector == 0 { yes } else { no },
    )
}

#[test]
fn computed_continuation_preserves_variable_topology_at_exact_budget() {
    let mut outputs = Vec::new();
    for selector in [0, 1] {
        for budget in [7, 8, 9] {
            let mut ar = Reduction::<256>::new();
            let mut old = Reduction::<256>::new();
            let (object, formula, expected) = dynamic(&mut ar, selector);
            assert_eq!(dynamic(&mut old, selector), (object, formula, expected));
            let ordinary =
                sequential::reduce_compacting_cached(&mut old, object, formula, budget, limits())
                    .unwrap();
            let (observed, events) = run(&mut ar, object, formula, budget);
            assert_eq!(observed.execution.stats, ordinary.stats);
            assert_eq!(observed.execution.peak_frames, ordinary.peak_frames);
            assert_eq!(
                std::format!("{:?}", observed.execution.outcome),
                std::format!("{:?}", ordinary.outcome)
            );
            let transitions = checked(&events);
            let composed = transitions
                .iter()
                .find(|t| t.pushed == Some(LiveFrame::Compose))
                .unwrap();
            let LogicalAction::Return { value, .. } = composed.before else {
                panic!()
            };
            let LogicalAction::Enter {
                formula: continuation,
                ..
            } = composed.after
            else {
                panic!()
            };
            assert_eq!(continuation, value);
            if budget == 7 {
                assert!(matches!(observed.execution.outcome, Outcome::Halt(0)));
                assert!(
                    !events
                        .0
                        .iter()
                        .any(|e| matches!(e, Event::Completed { .. }))
                );
            } else {
                let Outcome::Ok(actual, remaining) = observed.execution.outcome else {
                    panic!()
                };
                assert_eq!(particle(&ar, actual), particle(&ar, expected));
                assert_eq!(remaining, budget - 8);
                assert!(matches!(events.0.last(), Some(Event::Completed { .. })));
                if budget == 8 {
                    outputs.push(particle(&ar, actual));
                }
            }
        }
    }
    assert_ne!(outputs[0], outputs[1]);
}

#[test]
fn relocation_changes_physical_orders_without_changing_logical_transitions() {
    let mut small = Reduction::<256>::new();
    let mut ordinary = Reduction::<256>::new();
    let mut large = Reduction::<32768>::try_new_boxed().unwrap();
    let (object, formula) = loop_core(&mut small, 300);
    assert_eq!(loop_core(&mut ordinary, 300), (object, formula));
    assert_eq!(loop_core(&mut large, 300), (object, formula));
    assert!(small.limit_allocations(100));
    assert!(ordinary.limit_allocations(100));
    let plain =
        sequential::reduce_compacting_cached(&mut ordinary, object, formula, 5000, limits())
            .unwrap();
    let (observed, events) = run(&mut small, object, formula, 5000);
    let (large_run, large_events) = run(&mut large, object, formula, 5000);
    assert_eq!(observed.execution.stats, plain.stats);
    assert_eq!(observed.execution.peak_frames, plain.peak_frames);
    assert!(observed.execution.stats.collections > 10);
    assert_eq!(large_run.execution.stats.collections, 0);
    assert_eq!(checked(&events), checked(&large_events));
    let (Outcome::Ok(a, left), Outcome::Ok(b, right)) =
        (observed.execution.outcome, large_run.execution.outcome)
    else {
        panic!()
    };
    // The final counter reuses a pinned atom. Other surviving allocations have
    // moved, so matching final result Orders would not establish GC identity.
    assert!(
        (observed.execution.stats.pinned_nodes..small.count()).any(|id| {
            let same = (0..large.count()).find(|other| small.digest(id) == large.digest(*other));
            same.is_some_and(|other| other != id)
        })
    );
    assert_eq!(particle(&small, a), particle(&large, b));
    assert_eq!((left, right), (495, 495)); // 15*300+5 reductions
}

#[test]
fn cache_hits_still_emit_each_hash_finalizer_and_keep_107_reductions() {
    for budget in [106, 107, 108] {
        fn build(ar: &mut Reduction<256>) -> (Order, Order) {
            let object = atom(ar, 0);
            let q = quote(ar, 42);
            let h = op(ar, 15, q);
            let two = binary(ar, 3, h, h);
            (object, binary(ar, 3, two, two))
        }
        let mut ar = Reduction::<256>::new();
        let mut old = Reduction::<256>::new();
        let (object, formula) = build(&mut ar);
        assert_eq!(build(&mut old), (object, formula));
        let plain =
            sequential::reduce_compacting_cached(&mut old, object, formula, budget, limits())
                .unwrap();
        let (observed, events) = run(&mut ar, object, formula, budget);
        assert_eq!(
            std::format!("{:?}", observed.execution.outcome),
            std::format!("{:?}", plain.outcome)
        );
        assert_eq!(observed.execution.stats, plain.stats);
        checked(&events);
        if budget >= 107 {
            let finals: Vec<_> = events
                .0
                .iter()
                .filter_map(|e| match e {
                    Event::Transition(t)
                        if matches!(t.popped, Some(LiveFrame::Unary { opcode: 15, .. })) =>
                    {
                        Some(t)
                    }
                    _ => None,
                })
                .collect();
            assert_eq!(finals.len(), 4);
            assert!(finals[0].fresh_nodes > 0);
            assert!(finals[1..].iter().all(|t| t.fresh_nodes == 0));
            let Outcome::Ok(_, remaining) = observed.execution.outcome else {
                panic!()
            };
            assert_eq!(remaining, budget - 107);
        } else {
            assert!(matches!(observed.execution.outcome, Outcome::Halt(_)));
            assert!(
                !events
                    .0
                    .iter()
                    .any(|e| matches!(e, Event::Completed { .. }))
            );
        }
    }
}
