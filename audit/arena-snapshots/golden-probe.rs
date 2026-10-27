use nox::{Reduction, Order};
use nox::sequential::{observe::*, CompactionLimits};
use nebu::Goldilocks;
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

struct Digest { hash: hemera::Hasher }
impl Observer for Digest {
    type Error = ();
    fn record(&mut self, e: Event) -> Result<(), ()> {
        self.hash.update(e.encode().as_bytes());
        Ok(())
    }
}
fn main() {
    let mut ar = Reduction::<256>::new();
    let (object, formula) = loop_core(&mut ar, 40);
    assert!(ar.limit_allocations(100));
    let mut sink = Digest { hash: hemera::Hasher::new() };
    let run = reduce_compacting_observed_controlled(&mut ar, object, formula, 1000,
        CompactionLimits { max_frames: 4096, max_total_allocations: 100_000, max_collection_work: 100_000_000 },
        CaptureLimits { max_events: 1_000_000, max_bytes: 500_000_000, max_work: 4_000_000 },
        &mut sink, &mut || false).unwrap();
    println!("{:?}", sink.hash.finalize().as_bytes());
    println!("{:?}", run.capture);
    println!("{:?}", run.execution.stats);
    println!("{:?}", run.execution.outcome);
}
