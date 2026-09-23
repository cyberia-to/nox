extern crate std;
use super::*;
use crate::{ErrorKind, NoTrace, VecTrace};
use nebu::Goldilocks;

#[cfg(not(feature = "parallel"))]
mod differential;

fn atom<const N: usize>(ar: &mut Reduction<N>, v: u64) -> Order {
    ar.atom(Goldilocks::new(v)).unwrap()
}
fn op<const N: usize>(ar: &mut Reduction<N>, tag: u64, body: Order) -> Order {
    let t = atom(ar, tag);
    ar.pair(t, body).unwrap()
}
fn binary<const N: usize>(ar: &mut Reduction<N>, tag: u64, a: Order, b: Order) -> Order {
    let body = ar.pair(a, b).unwrap();
    op(ar, tag, body)
}
fn quote<const N: usize>(ar: &mut Reduction<N>, v: u64) -> Order {
    let v = atom(ar, v);
    op(ar, 1, v)
}
fn axis<const N: usize>(ar: &mut Reduction<N>, v: u64) -> Order {
    let v = atom(ar, v);
    op(ar, 0, v)
}
fn loop_core<const N: usize>(ar: &mut Reduction<N>, n: u64) -> (Order, Order) {
    let q0 = quote(ar, 0);
    let q1 = quote(ar, 1);
    let code = axis(ar, 2);
    let remaining = axis(ar, 6);
    let acc = axis(ar, 7);
    let done = binary(ar, 9, remaining, q0);
    let next = binary(ar, 6, remaining, q1);
    let acc_next = binary(ar, 5, acc, q1);
    let state = binary(ar, 3, next, acc_next);
    let subject = binary(ar, 3, code, state);
    let again = binary(ar, 2, subject, code);
    let arms = ar.pair(acc, again).unwrap();
    let formula = binary(ar, 4, done, arms);
    let zero = atom(ar, 0);
    let count = atom(ar, n);
    let state = ar.pair(count, zero).unwrap();
    (ar.pair(formula, state).unwrap(), formula)
}

#[test]
fn compact_loop_runs_beyond_legacy_depth_and_unroll_limits() {
    std::thread::Builder::new()
        .stack_size(32 << 20)
        .spawn(|| {
            let mut identity = None;
            for n in [0, 1, 16, 4097, 5000] {
                let mut ar = Reduction::<32768>::new();
                let (object, formula) = loop_core(&mut ar, n);
                let hash = *ar.digest(formula).unwrap();
                assert_eq!(identity.get_or_insert(hash), &hash);
                let mut trace = VecTrace::default();
                let result = reduce(
                    &mut ar,
                    object,
                    formula,
                    100_000,
                    Limits { max_frames: 16_384 },
                    &mut trace,
                )
                .unwrap();
                match result.outcome {
                    Outcome::Ok(r, remaining) => {
                        assert_eq!(ar.atom_value(r).unwrap().as_u64(), n);
                        assert_eq!(100_000 - remaining, 15 * n + 5);
                        assert_eq!(trace.0.len() as u64, 15 * n + 5);
                    }
                    outcome => panic!("{outcome:?}"),
                }
                assert!(result.peak_frames > n as u32);
                // Same nodes and trace-free outcome on a fresh arena.
                let mut untraced = Reduction::<32768>::new();
                let (obj, f) = loop_core(&mut untraced, n);
                let other = reduce(
                    &mut untraced,
                    obj,
                    f,
                    100_000,
                    Limits {
                        max_frames: result.peak_frames,
                    },
                    &mut NoTrace,
                )
                .unwrap();
                assert_eq!(
                    std::format!("{:?}", result.outcome),
                    std::format!("{:?}", other.outcome)
                );
                assert_eq!(ar.count(), untraced.count());
                for i in 0..ar.count() {
                    assert_eq!(ar.digest(i), untraced.digest(i));
                }
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn exact_frame_boundary_includes_root_and_leaf() {
    let mut ar = Reduction::<128>::new();
    let zero = atom(&mut ar, 0);
    let q = quote(&mut ar, 7);
    let f = binary(&mut ar, 5, q, q);
    assert_eq!(
        reduce(&mut ar, zero, q, 1, Limits { max_frames: 0 }, &mut NoTrace).unwrap_err(),
        Error::Frames
    );
    assert!(matches!(
        reduce(&mut ar, zero, q, 1, Limits { max_frames: 1 }, &mut NoTrace)
            .unwrap()
            .outcome,
        Outcome::Ok(_, 0)
    ));
    assert_eq!(
        reduce(&mut ar, zero, f, 3, Limits { max_frames: 1 }, &mut NoTrace).unwrap_err(),
        Error::Frames
    );
    let run = reduce(&mut ar, zero, f, 3, Limits { max_frames: 2 }, &mut NoTrace).unwrap();
    assert_eq!(run.peak_frames, 2);
    assert!(matches!(run.outcome, Outcome::Ok(_, 0)));
}

#[test]
fn reached_services_fail_but_quoted_services_are_data() {
    for tag in [16, 17] {
        let mut ar = Reduction::<128>::new();
        let zero = atom(&mut ar, 0);
        let service = op(&mut ar, tag, zero);
        let quoted = op(&mut ar, 1, service);
        let subject = quote(&mut ar, 0);
        let apply = binary(&mut ar, 2, subject, quoted);
        let limits = Limits { max_frames: 8 };
        assert!(
            matches!(reduce(&mut ar, zero, quoted, 1, limits, &mut NoTrace).unwrap().outcome, Outcome::Ok(r, 0) if r == service)
        );
        for f in [service, apply] {
            assert_eq!(
                reduce(&mut ar, zero, f, 100, limits, &mut NoTrace).unwrap_err(),
                Error::UnsupportedService(tag)
            );
        }
    }
}

#[test]
fn sequential_feature_independent_trace_and_budget_golden() {
    let mut ar = Reduction::<128>::new();
    let zero = atom(&mut ar, 0);
    let q3 = quote(&mut ar, 3);
    let q5 = quote(&mut ar, 5);
    let f = binary(&mut ar, 5, q3, q5);
    let mut trace = VecTrace::default();
    let run = reduce(&mut ar, zero, f, 99, Limits { max_frames: 2 }, &mut trace).unwrap();
    let Outcome::Ok(result, 96) = run.outcome else {
        panic!("{:?}", run.outcome)
    };
    assert_eq!(ar.atom_value(result).unwrap().as_u64(), 8);
    assert_eq!(trace.0.len(), 3);
    for (index, (tag, input, output)) in [(1, 1, 0), (1, 1, 0), (5, 99, 96)].into_iter().enumerate()
    {
        assert_eq!(
            (
                trace.0[index].r[0],
                trace.0[index].r[8],
                trace.0[index].r[9]
            ),
            (tag, input, output)
        );
    }
    // Left fails before right even in a parallel-feature build.
    let bad = axis(&mut ar, 2);
    let right = op(&mut ar, 8, q3);
    let fail = binary(&mut ar, 3, bad, right);
    let mut trace = VecTrace::default();
    assert!(matches!(
        reduce(
            &mut ar,
            zero,
            fail,
            100,
            Limits { max_frames: 4 },
            &mut trace
        )
        .unwrap()
        .outcome,
        Outcome::Error(ErrorKind::AxisError)
    ));
    assert_eq!(trace.0.len(), 2);
    assert_eq!(trace.0[0].r[0], 0);
    assert_eq!(trace.0[1].r[0], 3);
}

#[test]
fn cancellation_aborts_before_work_and_between_continuations() {
    let mut ar = Reduction::<512>::new();
    let (object, formula) = loop_core(&mut ar, 20);
    let before = ar.count();
    let limits = Limits { max_frames: 128 };
    let mut trace = VecTrace::default();
    assert_eq!(
        reduce_controlled(
            &mut ar,
            object,
            formula,
            1000,
            limits,
            &mut trace,
            &mut || true
        )
        .unwrap_err(),
        Error::Cancelled
    );
    assert!(trace.0.is_empty());
    assert_eq!(ar.count(), before);
    let mut checks = 0;
    let mut cancel = || {
        checks += 1;
        checks == 100
    };
    assert_eq!(
        reduce_controlled(
            &mut ar,
            object,
            formula,
            1000,
            limits,
            &mut trace,
            &mut cancel
        )
        .unwrap_err(),
        Error::Cancelled
    );
    assert_eq!(checks, 100);
    assert!(!trace.0.is_empty());
    assert!(trace.0.len() < 305);
}
