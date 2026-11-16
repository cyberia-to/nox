use super::*;
use crate::{ErrorKind, NoTrace, NullCalls, Outcome, reduce};

#[test]
fn exhausted_allowance_preserves_data_and_hash_consing() {
    let mut ar = Reduction::<16>::new();
    assert_eq!(ar.allocation_limit(), 12);
    assert!(ar.limit_allocations(3));
    let a = ar.atom(Goldilocks::new(7)).unwrap();
    let b = ar.atom(Goldilocks::new(8)).unwrap();
    let pair = ar.pair(a, b).unwrap();
    let identity = *ar.digest(pair).unwrap();
    assert_eq!(ar.atom(Goldilocks::new(9)), None);
    assert_eq!(ar.pair(b, a), None);
    assert_eq!(ar.atom(Goldilocks::new(7)), Some(a));
    assert_eq!(ar.pair(a, b), Some(pair));
    assert_eq!(ar.digest(pair), Some(&identity));
    assert_eq!(ar.count(), 3);
}

#[test]
fn allowance_only_tightens_without_invalidating_existing_nodes() {
    let mut ar = Reduction::<16>::new();
    let a = ar.atom(Goldilocks::ZERO).unwrap();
    assert!(!ar.limit_allocations(0));
    assert!(!ar.limit_allocations(13));
    assert_eq!(ar.allocation_limit(), 12);
    assert!(ar.limit_allocations(1));
    assert!(ar.limit_allocations(1));
    assert!(!ar.limit_allocations(2));
    assert_eq!(ar.atom(Goldilocks::ZERO), Some(a));
    let entry = *ar.get(a).unwrap();
    assert_eq!(ar.alloc_raw(entry), None);
    let mut empty = Reduction::<16>::new();
    assert!(empty.limit_allocations(0));
    assert_eq!(empty.atom(Goldilocks::ZERO), None);
}

#[test]
fn evaluation_obeys_the_same_allowance_as_loading() {
    let mut ar = Reduction::<64>::new();
    let zero = ar.atom(Goldilocks::ZERO).unwrap();
    let quote = ar.atom(Goldilocks::ONE).unwrap();
    let add = ar.atom(Goldilocks::new(5)).unwrap();
    let seven = ar.atom(Goldilocks::new(7)).unwrap();
    let q = ar.pair(quote, seven).unwrap();
    let operands = ar.pair(q, q).unwrap();
    let formula = ar.pair(add, operands).unwrap();
    let before = ar.count();
    assert!(ar.limit_allocations(before));
    assert!(matches!(
        reduce(&mut ar, zero, formula, 3, &NullCalls, &mut NoTrace),
        Outcome::Error(ErrorKind::Unavailable)
    ));
    assert_eq!(ar.count(), before);
    // Reusing an existing result remains executable at the same limit.
    assert!(matches!(
        reduce(&mut ar, zero, q, 1, &NullCalls, &mut NoTrace),
        Outcome::Ok(r, 0) if r == seven
    ));
}

#[test]
fn artifact_decode_cannot_bypass_the_arena_allowance() {
    let limits = crate::artifact::Limits {
        max_bytes: 4096,
        max_nodes: 20,
        max_depth: 4,
    };
    let mut source = Reduction::<64>::new();
    let a = source.atom(Goldilocks::new(7)).unwrap();
    let b = source.atom(Goldilocks::new(8)).unwrap();
    let pair = source.pair(a, b).unwrap();
    let bytes = crate::artifact::encode(&source, pair, limits).unwrap();
    let mut dest = Reduction::<64>::new();
    assert!(dest.limit_allocations(2));
    assert!(crate::artifact::decode(&mut dest, &bytes, limits).is_err());
    assert_eq!(dest.count(), 2);
    assert_eq!(dest.atom(Goldilocks::new(7)), Some(a));
    assert_eq!(dest.pair(a, b), None);
}

#[cfg(feature = "std")]
#[test]
fn forks_inherit_allowance_and_reinter_obeys_parent_limit() {
    let mut ar = Reduction::<64>::new();
    let a = ar.atom(Goldilocks::new(7)).unwrap();
    assert!(ar.limit_allocations(2));
    let mut fork = ar.fork();
    assert_eq!(fork.allocation_limit(), 2);
    let b = fork.atom(Goldilocks::new(8)).unwrap();
    assert_eq!(fork.pair(a, b), None);
    ar.atom(Goldilocks::new(9)).unwrap();
    assert_eq!(ar.reinter(&fork, a), Some(a));
    assert_eq!(ar.reinter(&fork, b), None);
    assert_eq!(ar.count(), 2);
}
