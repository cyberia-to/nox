extern crate std;

use super::*;
use crate::{NoTrace, Outcome, artifact, sequential};

#[test]
fn invalid_capacity_and_null_allocation_return_errors() {
    assert!(matches!(
        Reduction::<0>::try_new_boxed(),
        Err(AllocationError::InvalidCapacity)
    ));
    assert!(matches!(
        Reduction::<3>::try_new_boxed(),
        Err(AllocationError::InvalidCapacity)
    ));
    // SAFETY: null is explicitly accepted by the constructor's allocation
    // boundary. This exercises host allocation failure without exhausting RAM.
    assert!(matches!(
        unsafe { Reduction::<16>::initialize_allocation(ptr::null_mut()) },
        Err(AllocationError::OutOfMemory)
    ));
    let mut single = Reduction::<1>::try_new_boxed().unwrap();
    assert_eq!(single.allocation_limit(), 0);
    assert_eq!(single.atom(Goldilocks::ZERO), None);
}

#[test]
fn boxed_arena_preserves_orders_particles_and_exact_lifetime_limits() {
    let mut local = Reduction::<64>::new();
    let mut heap = Reduction::<64>::try_new_boxed().unwrap();
    assert_eq!(heap.count(), 0);
    assert_eq!(heap.allocation_limit(), local.allocation_limit());
    assert!(local.limit_allocations(20));
    assert!(heap.limit_allocations(20));
    for value in 0..10 {
        let a = local.atom(Goldilocks::new(value)).unwrap();
        let b = heap.atom(Goldilocks::new(value)).unwrap();
        assert_eq!(a, b);
        assert_eq!(local.digest(a), heap.digest(b));
        let left = local.pair(a, a).unwrap();
        let right = heap.pair(b, b).unwrap();
        assert_eq!(left, right);
        assert_eq!(
            local.get(left).unwrap().bound,
            heap.get(right).unwrap().bound
        );
        assert_eq!(local.digest(left), heap.digest(right));
    }
    assert_eq!(heap.count(), 20);
    assert_eq!(heap.atom(Goldilocks::new(99)), None);
    assert_eq!(local.atom(Goldilocks::new(99)), None);
    assert_eq!(heap.atom(Goldilocks::ZERO), local.atom(Goldilocks::ZERO));
    assert_eq!(heap.pair(0, 0), local.pair(0, 0));
    assert!(!heap.limit_allocations(19));
    assert!(!heap.limit_allocations(21));
    assert_eq!(heap.allocation_limit(), 20);
}

#[test]
fn heap_and_value_arenas_execute_and_encode_the_same_formula() {
    let mut local = Reduction::<64>::new();
    let mut heap = Reduction::<64>::try_new_boxed().unwrap();
    let make = |ar: &mut Reduction<64>| {
        let zero = ar.atom(Goldilocks::ZERO).unwrap();
        let one = ar.atom(Goldilocks::ONE).unwrap();
        let add = ar.atom(Goldilocks::new(5)).unwrap();
        let seven = ar.atom(Goldilocks::new(7)).unwrap();
        let quote = ar.pair(one, seven).unwrap();
        let args = ar.pair(quote, quote).unwrap();
        (zero, ar.pair(add, args).unwrap())
    };
    let (input, formula) = make(&mut local);
    assert_eq!(make(&mut heap), (input, formula));
    let limits = sequential::Limits { max_frames: 32 };
    let a = sequential::reduce(&mut local, input, formula, 100, limits, &mut NoTrace).unwrap();
    let b = sequential::reduce(&mut heap, input, formula, 100, limits, &mut NoTrace).unwrap();
    match (a.outcome, b.outcome) {
        (Outcome::Ok(x, left), Outcome::Ok(y, right)) => {
            assert_eq!((x, left), (y, right));
            assert_eq!(local.atom_value(x).unwrap(), Goldilocks::new(14));
            let limits = artifact::Limits {
                max_bytes: 4096,
                max_nodes: 64,
                max_depth: 16,
            };
            assert_eq!(
                artifact::encode(&local, x, limits).unwrap(),
                artifact::encode(&heap, y, limits).unwrap()
            );
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(a.peak_frames, b.peak_frames);
    assert_eq!(local.count(), heap.count());
}

#[test]
fn large_heap_arena_constructs_and_drops_on_a_small_worker_stack() {
    std::thread::Builder::new()
        .stack_size(128 << 10)
        .spawn(|| {
            let mut ar = Reduction::<{ 1 << 20 }>::try_new_boxed().unwrap();
            assert_eq!(ar.allocation_limit(), 786432);
            assert!(ar.limit_allocations(2));
            let zero = ar.atom(Goldilocks::ZERO).unwrap();
            let pair = ar.pair(zero, zero).unwrap();
            assert_eq!(ar.head(pair), Some(zero));
            assert_eq!(ar.tail(pair), Some(zero));
            assert_eq!(ar.atom(Goldilocks::ONE), None);
        })
        .unwrap()
        .join()
        .unwrap();
}
