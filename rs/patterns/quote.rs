//! pattern 1: quote — return body as literal

use crate::noun::NounId;
use crate::reduce::Outcome;
use crate::trace::TraceRow;

pub fn quote(body: NounId, budget: u64, row: &mut TraceRow) -> Outcome {
    row.r[4] = body as u64;
    row.r[7] = body as u64;
    Outcome::Ok(body, budget)
}

#[cfg(test)]
mod tests {
    use crate::reduce::{reduce, Outcome};
    use crate::call::NullCalls;
    use crate::trace::NoTrace;
    use crate::noun::{Order, Tag};
    use nebu::Goldilocks;

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    #[test]
    fn quote_returns_literal() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        let body = ar.atom(g(42), Tag::Field).unwrap();
        let tag = ar.atom(g(1), Tag::Field).unwrap();
        let formula = ar.cell(tag, body).unwrap();
        match reduce(&mut ar, obj, formula, 100, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(r, body),
            o => panic!("{:?}", o),
        }
    }
}
