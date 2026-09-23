use super::*;

fn compare(
    build: impl Fn(&mut Reduction<512>) -> (Order, Order),
    budgets: &[u64],
    extra_nodes: Option<u32>,
) {
    for &budget in budgets {
        let mut old = Reduction::<512>::new();
        let mut new = Reduction::<512>::new();
        let (obj, f) = build(&mut old);
        assert_eq!(build(&mut new), (obj, f));
        if let Some(extra) = extra_nodes {
            assert!(old.limit_allocations(old.count() + extra));
            assert!(new.limit_allocations(new.count() + extra));
        }
        let mut old_trace = VecTrace::default();
        let mut new_trace = VecTrace::default();
        let expected = crate::reduce(&mut old, obj, f, budget, &crate::NullCalls, &mut old_trace);
        let actual = reduce(
            &mut new,
            obj,
            f,
            budget,
            Limits { max_frames: 256 },
            &mut new_trace,
        )
        .unwrap();
        assert_eq!(
            std::format!("{:?}", expected),
            std::format!("{:?}", actual.outcome),
            "budget {budget}, formula {f}"
        );
        assert_eq!(old.count(), new.count());
        for i in 0..old.count() {
            assert_eq!(old.atom_value(i), new.atom_value(i));
            assert_eq!(old.head(i), new.head(i));
            assert_eq!(old.tail(i), new.tail(i));
            assert_eq!(old.digest(i), new.digest(i));
        }
        assert_eq!(
            old_trace.0.len(),
            new_trace.0.len(),
            "budget {budget}, formula {f}"
        );
        for (a, b) in old_trace.0.iter().zip(&new_trace.0) {
            assert_eq!(a.r, b.r);
        }
    }
}

#[test]
fn all_pure_patterns_preserve_rows_types_and_budget_boundaries() {
    for tag in 0..16 {
        for value in [0, 1, 7, u32::MAX as u64, 1 << 32, 0xffff_ffff_0000_0000] {
            compare(
                |ar| {
                    let zero = atom(ar, 0);
                    let v = atom(ar, value);
                    let object = ar.pair(zero, v).unwrap();
                    let qv = op(ar, 1, v);
                    let q7 = quote(ar, 7);
                    let body = match tag {
                        0 | 1 => v,
                        8 | 13 | 15 => qv,
                        4 => {
                            let arms = ar.pair(q7, qv).unwrap();
                            ar.pair(qv, arms).unwrap()
                        }
                        _ => ar.pair(qv, q7).unwrap(),
                    };
                    (object, op(ar, tag, body))
                },
                &[0, 1, 2, 3, 25, 26, 32, 33, 34, 64, 65, 66, 128, 1000],
                None,
            );
        }
    }
}

#[test]
fn allocation_failure_preserves_partial_multirow_blocks() {
    for tag in [0, 3, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15] {
        for extra in 0..9 {
            compare(
                |ar| {
                    let a = atom(ar, 42);
                    let b = atom(ar, 19);
                    let obj = ar.pair(a, b).unwrap();
                    let qa = op(ar, 1, a);
                    let qb = op(ar, 1, b);
                    let body = match tag {
                        0 => atom(ar, 0),
                        8 | 13 | 15 => qa,
                        _ => ar.pair(qa, qb).unwrap(),
                    };
                    (obj, op(ar, tag, body))
                },
                &[0, 1, 33, 65, 1000],
                Some(extra),
            );
        }
    }
}

#[test]
fn dynamic_compose_and_selected_branch_preserve_reservations() {
    for condition in [0, 1] {
        compare(
            |ar| {
                let obj = atom(ar, 0);
                let test = quote(ar, condition);
                let cheap = quote(ar, 7);
                let expensive = op(ar, 8, cheap);
                let arms = ar.pair(cheap, expensive).unwrap();
                let branch = binary(ar, 4, test, arms);
                let identity = axis(ar, 1);
                let q_identity = op(ar, 1, identity);
                let dynamic = binary(ar, 2, branch, q_identity);
                (obj, binary(ar, 5, branch, dynamic))
            },
            &[0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 67, 68, 70, 1000],
            None,
        );
    }
}

#[test]
fn errors_wait_for_required_children_and_keep_failure_precedence() {
    for tag in [2, 3, 5, 8, 10, 11, 13, 15, 99] {
        for malformed in [false, true] {
            compare(
                |ar| {
                    let zero = atom(ar, 0);
                    let pair = ar.pair(zero, zero).unwrap();
                    let left = op(ar, 1, pair);
                    let q0 = op(ar, 1, zero);
                    let right = op(ar, 8, q0);
                    let body = if malformed {
                        zero
                    } else if [8, 13, 15].contains(&tag) {
                        left
                    } else {
                        ar.pair(left, right).unwrap()
                    };
                    (zero, op(ar, tag, body))
                },
                &[0, 1, 2, 3, 32, 33, 64, 65, 66, 1000],
                None,
            );
        }
    }
}

fn generated(ar: &mut Reduction<512>, seed: &mut u64, depth: u32) -> Order {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
    let tag = (*seed >> 32) % 16;
    if depth == 0 {
        return quote(ar, (*seed >> 8) % 20);
    }
    let a = generated(ar, seed, depth - 1);
    let b = generated(ar, seed, depth - 1);
    let body = match tag {
        0 => atom(ar, (*seed >> 16) % 8),
        1 | 8 | 13 | 15 => a,
        4 => {
            let c = generated(ar, seed, depth - 1);
            let rest = ar.pair(b, c).unwrap();
            ar.pair(a, rest).unwrap()
        }
        _ => ar.pair(a, b).unwrap(),
    };
    op(ar, tag, body)
}

#[test]
fn generated_nested_formulas_preserve_all_observable_rows_and_nodes() {
    for seed in 0..256 {
        compare(
            |ar| {
                let zero = atom(ar, 0);
                let one = atom(ar, 1);
                let obj = ar.pair(zero, one).unwrap();
                (obj, generated(ar, &mut (seed + 1), 4))
            },
            &[0, 1, 3, 25, 32, 64, 128, 1000],
            None,
        );
    }
}
