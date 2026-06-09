//! atom or pair — the two kinds of data. nothing else.

use nebu::Goldilocks;
use super::OrderId;

/// the two kinds of data
#[derive(Debug, Clone, Copy)]
pub enum Data {
    Atom { value: Goldilocks },
    Pair { left: OrderId, right: OrderId },
}
