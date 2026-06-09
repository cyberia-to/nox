//! pattern 1: quote — return body as literal

use crate::data::OrderId;
use crate::reduce::Outcome;
use crate::trace::TraceRow;

pub fn quote(body: OrderId, budget: u64, row: &mut TraceRow) -> Outcome {
    row.r[4] = body as u64;
    row.r[7] = body as u64;
    Outcome::Ok(body, budget)
}

#[cfg(test)]
mod tests {
    use crate::reduce::{reduce, Outcome};
    use crate::call::NullCalls;
    use crate::trace::NoTrace;
    use crate::data::{Order};
    use nebu::Goldilocks;

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    #[test]
    fn quote_returns_literal() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0)).unwrap();
        let body = ar.atom(g(42)).unwrap();
        let tag = ar.atom(g(1)).unwrap();
        let formula = ar.pair(tag, body).unwrap();
        match reduce(&mut ar, obj, formula, 100, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, _) => assert_eq!(r, body),
            o => panic!("{:?}", o),
        }
    }
}
