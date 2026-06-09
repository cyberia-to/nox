// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! Dump all genesis jet formulas in bracket notation to stdout.
//!
//! Usage:
//!   cargo run --example dump_formulas
//!
//! Output one block per formula: === <name> === followed by the bracket text.
//! Redirect to generate the canonical .nox files in jets/.

use nox::data::{Reduction, Order, Data};
use nox::jets::formulas::{
    build_poly_eval_formula,
    build_merkle_verify_formula,
    build_fri_fold_formula,
    build_ntt_formula,
    build_cyberlink_formula,
    build_decider_formula,
};

const N: usize = 1 << 14; // 16 K nodes — enough for all six formulas

fn print_data(reduction: &Reduction<N>, id: Order) -> String {
    match reduction.get(id).map(|e| e.inner) {
        Some(Data::Atom { value, .. }) => value.as_u64().to_string(),
        Some(Data::Pair { left, right }) => {
            format!("[{} {}]", print_data(reduction, left), print_data(reduction, right))
        }
        None => "<invalid>".to_string(),
    }
}

fn dump(reduction: &Reduction<N>, name: &str, formula: Option<Order>) {
    match formula {
        Some(id) => {
            println!("=== {} ===", name);
            println!("{}", print_data(reduction, id));
            println!();
        }
        None => eprintln!("error: {} formula build failed (reduction full?)", name),
    }
}

fn main() {
    let mut reduction = Reduction::<N>::new();

    let poly_eval     = build_poly_eval_formula(&mut reduction);
    let merkle_verify = build_merkle_verify_formula(&mut reduction);
    let fri_fold      = build_fri_fold_formula(&mut reduction);
    let ntt           = build_ntt_formula(&mut reduction);
    let cyberlink     = build_cyberlink_formula(&mut reduction);
    let decider       = build_decider_formula(&mut reduction);

    dump(&reduction, "poly_eval",     poly_eval);
    dump(&reduction, "merkle_verify", merkle_verify);
    dump(&reduction, "fri_fold",      fri_fold);
    dump(&reduction, "ntt",           ntt);
    dump(&reduction, "cyberlink",     cyberlink);
    dump(&reduction, "decider",       decider);
}
