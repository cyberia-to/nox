//! pattern 4: branch — evaluate test, take yes (0) or no (nonzero)

use crate::data::{Reduction, Order};
use crate::reduce::{Outcome, ErrorKind, pair_children, evaluate_unary};
use crate::call::CallProvider;
use crate::trace::{Tracer, TraceRow};
use crate::jets::registry::JetRegistry;

pub fn branch<const N: usize, T: Tracer>(
    reduction: &mut Reduction<N>, object: Order, body: Order, budget: u64,
    hints: &dyn CallProvider<N>, tracer: &mut T, depth: u64,
    row: &mut TraceRow, registry: &JetRegistry<N>,
) -> Outcome {
    let (test_formula, rest) = match pair_children(reduction, body) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    let (yes_formula, no_formula) = match pair_children(reduction, rest) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    // Step 1: evaluate test with its bound (partitioned if possible).
    let (test_result, budget) = match evaluate_unary(reduction, object, test_formula, budget, hints, tracer, depth, registry) {
        Ok(v) => v, Err(o) => return o,
    };
    let test_value = match reduction.atom_value(test_result) {
        Some(v) => v.as_u64(),
        None => return Outcome::Error(ErrorKind::TypeError),
    };
    let selector: u64 = if test_value == 0 { 0 } else { 1 };
    let chosen = if selector == 0 { yes_formula } else { no_formula };
    // r4 = test value (canonical field repr), r5 = inverse hint for (test != 0),
    // r6 = result Order (output of chosen arm), r10 = selector (0=yes, 1=no).
    // gadget: selector * (1 - r4 * r5) = 0 binds selector to "test != 0".
    row.r[4] = test_value;
    row.r[5] = if test_value == 0 {
        0
    } else {
        nebu::Goldilocks::new(test_value).inv().as_u64()
    };
    row.r[10] = selector;
    // Step 2: evaluate ONLY the chosen arm under partitioned semantics.
    // Slack from max-of-arms accrues to the caller automatically — we only
    // charge for the chosen arm's actual cost.
    match evaluate_unary(reduction, object, chosen, budget, hints, tracer, depth, registry) {
        Ok((val, remaining)) => {
            row.r[6] = val as u64;
            Outcome::Ok(val, remaining)
        }
        Err(o) => o,
    }
}

#[cfg(test)]
mod tests {
    use crate::reduce::{reduce, Outcome};
    use crate::call::NullCalls;
    use crate::trace::NoTrace;
    use crate::data::{Reduction};
    use nebu::Goldilocks;

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    /// formula = [4 [1 0] [1 yes_val] [1 no_val]]
    /// test = quote(0) = 0 → take yes branch → yes_val
    #[test]
    fn branch_zero_takes_yes() {
        let mut ar = Reduction::<1024>::new();
        let obj = ar.atom(g(0)).unwrap();
        let t1 = ar.atom(g(1)).unwrap();

        let yes_val = ar.atom(g(10)).unwrap();
        let no_val = ar.atom(g(20)).unwrap();

        let zero = ar.atom(g(0)).unwrap();
        let test_f = ar.pair(t1, zero).unwrap();
        let yes_f = ar.pair(t1, yes_val).unwrap();
        let no_f = ar.pair(t1, no_val).unwrap();

        let branches = ar.pair(yes_f, no_f).unwrap();
        let body = ar.pair(test_f, branches).unwrap();
        let t4 = ar.atom(g(4)).unwrap();
        let formula = ar.pair(t4, body).unwrap();

        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap(), g(10)),
            o => panic!("{:?}", o),
        }
    }

    /// test = quote(1) = 1 → take no branch → no_val
    #[test]
    fn branch_nonzero_takes_no() {
        let mut ar = Reduction::<1024>::new();
        let obj = ar.atom(g(0)).unwrap();
        let t1 = ar.atom(g(1)).unwrap();

        let yes_val = ar.atom(g(10)).unwrap();
        let no_val = ar.atom(g(20)).unwrap();

        let one = ar.atom(g(1)).unwrap();
        let test_f = ar.pair(t1, one).unwrap();
        let yes_f = ar.pair(t1, yes_val).unwrap();
        let no_f = ar.pair(t1, no_val).unwrap();

        let branches = ar.pair(yes_f, no_f).unwrap();
        let body = ar.pair(test_f, branches).unwrap();
        let t4 = ar.atom(g(4)).unwrap();
        let formula = ar.pair(t4, body).unwrap();

        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap(), g(20)),
            o => panic!("{:?}", o),
        }
    }
}
