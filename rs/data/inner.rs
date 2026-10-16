//! atom or pair — the two kinds of data. nothing else.

use super::Order;
use nebu::Goldilocks;

/// the two kinds of data
#[derive(Debug, Clone, Copy)]
pub enum Data {
    Atom { value: Goldilocks },
    Pair { left: Order, right: Order },
}
