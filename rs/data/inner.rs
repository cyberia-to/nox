//! atom or pair — the two kinds of data. nothing else.

use nebu::Goldilocks;
use super::Order;

/// the two kinds of data
#[derive(Debug, Clone, Copy)]
pub enum Data {
    Atom { value: Goldilocks },
    Pair { left: Order, right: Order },
}
