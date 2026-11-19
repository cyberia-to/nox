use super::*;

fn g(v: u64) -> Goldilocks {
    Goldilocks::new(v)
}

#[test]
fn atom_hash_conses_identical_values() {
    let mut r = Reduction::<16>::new();
    let a = r.atom(g(7)).unwrap();
    let b = r.atom(g(7)).unwrap();
    assert_eq!(a, b);
    assert_eq!(r.count(), 1);
}

#[test]
fn atom_distinguishes_values() {
    let mut r = Reduction::<16>::new();
    let a = r.atom(g(7)).unwrap();
    let b = r.atom(g(8)).unwrap();
    assert_ne!(a, b);
    assert_eq!(r.count(), 2);
}

#[test]
fn pair_hash_conses_identical_children() {
    let mut r = Reduction::<16>::new();
    let a = r.atom(g(1)).unwrap();
    let b = r.atom(g(2)).unwrap();
    let p1 = r.pair(a, b).unwrap();
    let p2 = r.pair(a, b).unwrap();
    assert_eq!(p1, p2);
    assert_eq!(r.count(), 3); // two atoms + one pair, not two pairs
}

#[test]
fn get_returns_none_out_of_bounds() {
    let mut r = Reduction::<16>::new();
    let a = r.atom(g(1)).unwrap();
    assert!(r.get(a).is_some());
    assert!(r.get(a + 1).is_none());
    assert!(r.get(Order::MAX).is_none());
}

#[test]
fn alloc_raw_refuses_past_three_quarter_load_factor() {
    // N=4: cap is (4/4)*3 = 3 entries. Distinct atoms never hash-cons,
    // so the 4th allocation must be refused instead of overflowing entries[].
    let mut r = Reduction::<4>::new();
    assert!(r.atom(g(1)).is_some());
    assert!(r.atom(g(2)).is_some());
    assert!(r.atom(g(3)).is_some());
    assert_eq!(r.count(), 3);
    assert!(r.atom(g(4)).is_none());
    assert_eq!(r.count(), 3); // refused allocation must not bump count
}

#[test]
fn hash_data_round_trips_through_read_hash_data() {
    let mut r = Reduction::<16>::new();
    let d = hash_atom(g(0xC0FFEE));
    let node = r.hash_data(&d).unwrap();
    assert_eq!(r.read_hash_data(node).unwrap(), d);
}

#[test]
fn read_hash_data_rejects_a_plain_atom() {
    let mut r = Reduction::<16>::new();
    let a = r.atom(g(1)).unwrap();
    assert!(r.read_hash_data(a).is_none());
}

#[test]
fn is_atom_is_pair_head_tail_atom_value() {
    let mut r = Reduction::<16>::new();
    let a = r.atom(g(9)).unwrap();
    let b = r.atom(g(10)).unwrap();
    let p = r.pair(a, b).unwrap();

    assert!(r.is_atom(a) && !r.is_pair(a));
    assert!(r.is_pair(p) && !r.is_atom(p));
    assert_eq!(r.head(p), Some(a));
    assert_eq!(r.tail(p), Some(b));
    assert_eq!(r.atom_value(a), Some(g(9)));
    assert_eq!(r.atom_value(p), None);
    assert_eq!(r.head(a), None);
}

// compute_pair_bound dispatch — each arm below matches the same tag
// that data::cost::PATTERN_COSTS (and reduce::COSTS, locked together
// in row 211) assigns that pattern.
mod compute_pair_bound {
    use super::*;

    fn tagged_pair<const N: usize>(r: &mut Reduction<N>, tag: u64, body: Order) -> Order {
        let t = r.atom(g(tag)).unwrap();
        r.pair(t, body).unwrap()
    }

    #[test]
    fn no_sub_formula_tag_is_exact_base() {
        let mut r = Reduction::<64>::new();
        let body = r.atom(g(0)).unwrap();
        let p = tagged_pair(&mut r, 1, body); // 1 = quote
        assert_eq!(r.get(p).unwrap().bound, Cost::Exact(PATTERN_COSTS[1]));
    }

    #[test]
    fn compose_tag_is_dynamic() {
        let mut r = Reduction::<64>::new();
        let x = r.atom(g(0)).unwrap();
        let y = r.atom(g(0)).unwrap();
        let xy = r.pair(x, y).unwrap();
        let p = tagged_pair(&mut r, 2, xy); // 2 = compose
        assert!(r.get(p).unwrap().bound.is_dynamic());
    }

    #[test]
    fn binary_structural_tag_sums_children() {
        let mut r = Reduction::<64>::new();
        let a = r.atom(g(0)).unwrap();
        let b = r.atom(g(0)).unwrap();
        let ab = r.pair(a, b).unwrap();
        let p = tagged_pair(&mut r, 5, ab); // 5 = add
        assert_eq!(r.get(p).unwrap().bound, Cost::Exact(PATTERN_COSTS[5]));
    }

    #[test]
    fn branch_tag_takes_max_of_arms() {
        let mut r = Reduction::<64>::new();
        // Build a deep `yes` branch (tag 5, add) so its bound exceeds `no`'s.
        let x = r.atom(g(0)).unwrap();
        let y = r.atom(g(0)).unwrap();
        let xy = r.pair(x, y).unwrap();
        let yes = tagged_pair(&mut r, 5, xy);
        let no = r.atom(g(0)).unwrap();
        let test = r.atom(g(0)).unwrap();
        let rest = r.pair(yes, no).unwrap();
        let body = r.pair(test, rest).unwrap();
        let p = tagged_pair(&mut r, 4, body); // 4 = branch

        let expected = Cost::branch(
            PATTERN_COSTS[4],
            Cost::Exact(0), // test is a bare atom, bound 0
            r.get(yes).unwrap().bound,
            Cost::Exact(0), // no is a bare atom, bound 0
        );
        assert_eq!(r.get(p).unwrap().bound, expected);
    }

    #[test]
    fn unary_tag_uses_sum1() {
        let mut r = Reduction::<64>::new();
        let inner = r.atom(g(0)).unwrap();
        let p = tagged_pair(&mut r, 8, inner); // 8 = inv
        assert_eq!(r.get(p).unwrap().bound, Cost::Exact(PATTERN_COSTS[8]));
    }

    #[test]
    fn call_tag_is_dynamic() {
        let mut r = Reduction::<64>::new();
        let tag_f = r.atom(g(0)).unwrap();
        let check_f = r.atom(g(0)).unwrap();
        let body = r.pair(tag_f, check_f).unwrap();
        let p = tagged_pair(&mut r, 16, body); // 16 = call
        assert!(r.get(p).unwrap().bound.is_dynamic());
    }

    #[test]
    fn unknown_tag_is_exact_zero() {
        let mut r = Reduction::<64>::new();
        let body = r.atom(g(0)).unwrap();
        let p = tagged_pair(&mut r, 99, body); // no pattern uses tag 99
        assert_eq!(r.get(p).unwrap().bound, Cost::Exact(0));
    }

    #[test]
    fn left_child_not_an_atom_is_exact_zero() {
        // compute_pair_bound treats a pair as data, not a formula, when
        // its own left child isn't an atom (no tag to dispatch on).
        let mut r = Reduction::<64>::new();
        let a = r.atom(g(0)).unwrap();
        let b = r.atom(g(0)).unwrap();
        let left = r.pair(a, b).unwrap(); // pair, not an atom tag
        let right = r.atom(g(0)).unwrap();
        let p = r.pair(left, right).unwrap();
        assert_eq!(r.get(p).unwrap().bound, Cost::Exact(0));
    }
}
