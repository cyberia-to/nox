//! pattern 4: branch — evaluate test, take yes (0) or no (nonzero)

use crate::noun::{Order, NounId};
use crate::reduce::{Outcome, ErrorKind, cell_pair, evaluate_unary};
use crate::call::CallProvider;
use crate::trace::{Tracer, TraceRow};
use crate::jets::registry::JetRegistry;

pub fn branch<const N: usize, T: Tracer>(
    order: &mut Order<N>, object: NounId, body: NounId, budget: u64,
    hints: &dyn CallProvider<N>, tracer: &mut T, depth: u64,
    row: &mut TraceRow, registry: &JetRegistry<N>,
) -> Outcome {
    let (test_formula, rest) = match cell_pair(order, body) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    let (yes_formula, no_formula) = match cell_pair(order, rest) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    // Step 1: evaluate test with its bound (partitioned if possible).
    let (test_result, budget) = match evaluate_unary(order, object, test_formula, budget, hints, tracer, depth, registry) {
        Ok(v) => v, Err(o) => return o,
    };
    let test_value = match order.atom_value(test_result) {
        Some((v, _)) => v.as_u64(),
        None => return Outcome::Error(ErrorKind::TypeError),
    };
    let selector: u64 = if test_value == 0 { 0 } else { 1 };
    let chosen = if selector == 0 { yes_formula } else { no_formula };
    // r4 = test value (canonical field repr), r5 = inverse hint for (test != 0),
    // r6 = result NounId (output of chosen arm), r10 = selector (0=yes, 1=no).
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
    match evaluate_unary(order, object, chosen, budget, hints, tracer, depth, registry) {
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
    use crate::noun::{Order, Tag};
    use nebu::Goldilocks;

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    /// formula = [4 [1 0] [1 yes_val] [1 no_val]]
    /// test = quote(0) = 0 → take yes branch → yes_val
    #[test]
    fn branch_zero_takes_yes() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        let t1 = ar.atom(g(1), Tag::Field).unwrap();

        let yes_val = ar.atom(g(10), Tag::Field).unwrap();
        let no_val = ar.atom(g(20), Tag::Field).unwrap();

        let zero = ar.atom(g(0), Tag::Field).unwrap();
        let test_f = ar.cell(t1, zero).unwrap();
        let yes_f = ar.cell(t1, yes_val).unwrap();
        let no_f = ar.cell(t1, no_val).unwrap();

        let branches = ar.cell(yes_f, no_f).unwrap();
        let body = ar.cell(test_f, branches).unwrap();
        let t4 = ar.atom(g(4), Tag::Field).unwrap();
        let formula = ar.cell(t4, body).unwrap();

        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(10)),
            o => panic!("{:?}", o),
        }
    }

    /// test = quote(1) = 1 → take no branch → no_val
    #[test]
    fn branch_nonzero_takes_no() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        let t1 = ar.atom(g(1), Tag::Field).unwrap();

        let yes_val = ar.atom(g(10), Tag::Field).unwrap();
        let no_val = ar.atom(g(20), Tag::Field).unwrap();

        let one = ar.atom(g(1), Tag::Field).unwrap();
        let test_f = ar.cell(t1, one).unwrap();
        let yes_f = ar.cell(t1, yes_val).unwrap();
        let no_f = ar.cell(t1, no_val).unwrap();

        let branches = ar.cell(yes_f, no_f).unwrap();
        let body = ar.cell(test_f, branches).unwrap();
        let t4 = ar.atom(g(4), Tag::Field).unwrap();
        let formula = ar.cell(t4, body).unwrap();

        match reduce(&mut ar, obj, formula, 1000, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(20)),
            o => panic!("{:?}", o),
        }
    }
}
