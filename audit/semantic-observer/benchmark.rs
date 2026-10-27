//! Paired ordinary compacting-path benchmark; this does not capture a witness.
use nox::{Order, Outcome, Reduction, sequential::*};
use nebu::Goldilocks;
use std::{hint::black_box, time::Instant};

fn atom(ar: &mut Reduction<256>, value: u64) -> Order {
    ar.atom(Goldilocks::new(value)).unwrap()
}
fn op(ar: &mut Reduction<256>, tag: u64, body: Order) -> Order {
    let tag = atom(ar, tag); ar.pair(tag, body).unwrap()
}
fn binary(ar: &mut Reduction<256>, tag: u64, a: Order, b: Order) -> Order {
    let body = ar.pair(a, b).unwrap(); op(ar, tag, body)
}
fn quote(ar: &mut Reduction<256>, value: u64) -> Order {
    let value = atom(ar, value); op(ar, 1, value)
}
fn axis(ar: &mut Reduction<256>, value: u64) -> Order {
    let value = atom(ar, value); op(ar, 0, value)
}
fn build(ar: &mut Reduction<256>, n: u64) -> (Order, Order) {
    let q0 = quote(ar, 0); let q1 = quote(ar, 1);
    let code = axis(ar, 2); let remaining = axis(ar, 6); let acc = axis(ar, 7);
    let done = binary(ar, 9, remaining, q0);
    let next = binary(ar, 6, remaining, q1); let acc_next = binary(ar, 5, acc, q1);
    let state = binary(ar, 3, next, acc_next); let subject = binary(ar, 3, code, state);
    let again = binary(ar, 2, subject, code); let arms = ar.pair(acc, again).unwrap();
    let formula = binary(ar, 4, done, arms);
    let zero = atom(ar, 0); let count = atom(ar, n); let state = ar.pair(count, zero).unwrap();
    (ar.pair(formula, state).unwrap(), formula)
}
fn main() {
    let repetitions: u32 = std::env::args().nth(1).unwrap().parse().unwrap();
    let start = Instant::now();
    for _ in 0..repetitions {
        let mut ar = Reduction::<256>::new();
        let (object, formula) = build(&mut ar, 3000);
        assert!(ar.limit_allocations(160));
        let run = reduce_compacting_cached(&mut ar, object, formula, 100_000,
            CompactionLimits { max_frames: 16384, max_total_allocations: 100_000,
                max_collection_work: 100_000_000 }).unwrap();
        let Outcome::Ok(value, remaining) = run.outcome else { panic!("loop failed") };
        assert_eq!(ar.atom_value(value).unwrap().as_u64(), 3000);
        assert_eq!(remaining, 54_995);
        black_box((ar.digest(value), run.stats));
    }
    println!("{}", start.elapsed().as_nanos());
}
