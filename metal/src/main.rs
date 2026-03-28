// ---
// tags: nox, metal, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! nox-metal — bare metal nox evaluator
//!
//! boots via UEFI, runs nox formulas with direct hardware access.
//! no OS. no kernel. no drivers except nox formulas.
//!
//! metal boundary: physical_read / physical_write
//! everything else is nox.

#![no_std]
#![no_main]

extern crate alloc;

use uefi::prelude::*;
use uefi::println;
use nebu::Goldilocks;
use nox::{Order, Noun, Tag, reduce, Outcome, NullCalls};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// metal boundary — the irreducible interface
// between mathematics and physics.
// two functions. everything above is nox.
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// read a value from a physical memory address.
#[inline]
unsafe fn physical_read(addr: usize) -> u64 {
    unsafe { core::ptr::read_volatile(addr as *const u64) }
}

/// write a value to a physical memory address.
#[inline]
unsafe fn physical_write(addr: usize, val: u64) {
    unsafe { core::ptr::write_volatile(addr as *mut u64, val) }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// everything below is nox computation.
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

const ORDER_SIZE: usize = 1024;

#[entry]
fn main() -> Status {
    println!("nox-metal — proof-native VM on bare metal");
    println!("metal boundary: physical_read + physical_write");
    println!("everything else: nox");
    println!();

    // ── demo: run [5 [[1 3] [1 5]]] on [0 0] ──
    // add(quote(3), quote(5)) = 8

    let mut order = Order::<ORDER_SIZE>::new();

    // object: [0, 0]
    let zero = order.atom(Goldilocks::ZERO, Tag::Field).unwrap();
    let object = order.cell(zero, zero).unwrap();

    // formula: [5 [[1 3] [1 5]]]
    let tag_q = order.atom(Goldilocks::new(1), Tag::Field).unwrap();
    let three = order.atom(Goldilocks::new(3), Tag::Field).unwrap();
    let five  = order.atom(Goldilocks::new(5), Tag::Field).unwrap();
    let q3    = order.cell(tag_q, three).unwrap();
    let q5    = order.cell(tag_q, five).unwrap();
    let pair  = order.cell(q3, q5).unwrap();
    let tag_a = order.atom(Goldilocks::new(5), Tag::Field).unwrap();
    let formula = order.cell(tag_a, pair).unwrap();

    // reduce
    let hints = NullCalls;
    let result = reduce(&mut order, object, formula, 100, &hints);

    match result {
        Outcome::Ok(id, remaining) => {
            if let Some((val, _)) = order.atom_value(id) {
                println!("  3 + 5 = {}", val.as_u64());
                println!("  budget remaining: {}", remaining);
            } else {
                println!("  result: <cell>");
            }
        }
        Outcome::Halt(r) => println!("  halted at budget {}", r),
        Outcome::Error(e) => println!("  error: {:?}", e),
    }

    println!();
    println!("nox runs on bare metal. no OS between math and physics.");

    Status::SUCCESS
}
