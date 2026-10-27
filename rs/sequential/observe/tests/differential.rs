use super::*;

#[test]
fn observed_all_pure_patterns_match_unobserved_values_failures_and_physical_reports() {
    for tag in 0..16 {
        for value in [0, 7, 1 << 32] {
            for budget in [0, 1, 3, 25, 33, 65, 1000] {
                fn build(ar: &mut Reduction<256>, tag: u64, value: u64) -> (Order, Order) {
                    let object = atom(ar, 0);
                    let q = quote(ar, value);
                    let other = quote(ar, 7);
                    let formula = match tag {
                        0 => axis(ar, value),
                        1 => op(ar, 1, other),
                        8 | 13 | 15 => op(ar, tag, q),
                        4 => {
                            let arms = ar.pair(q, other).unwrap();
                            binary(ar, 4, q, arms)
                        }
                        _ => binary(ar, tag, q, other),
                    };
                    (object, formula)
                }
                let mut old = Reduction::<256>::new();
                let mut new = Reduction::<256>::new();
                let (object, formula) = build(&mut old, tag, value);
                assert_eq!(build(&mut new, tag, value), (object, formula));
                let expected = sequential::reduce_compacting_cached(
                    &mut old,
                    object,
                    formula,
                    budget,
                    limits(),
                )
                .unwrap();
                let (observed, events) = run(&mut new, object, formula, budget);
                let actual = observed.execution;
                assert_eq!(
                    std::format!("{:?}", actual.outcome),
                    std::format!("{:?}", expected.outcome)
                );
                assert_eq!(actual.peak_frames, expected.peak_frames);
                assert_eq!(actual.stats, expected.stats);
                assert_eq!(new.count(), old.count());
                for id in 0..new.count() {
                    assert_eq!(new.digest(id), old.digest(id));
                    assert_eq!(crate::bound(&new, id), crate::bound(&old, id));
                }
                checked(&events);
                assert_eq!(
                    matches!(actual.outcome, Outcome::Ok(..)),
                    matches!(events.0.last(), Some(Event::Completed { .. }))
                );
            }
        }
    }
}
