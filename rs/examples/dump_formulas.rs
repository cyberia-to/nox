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

use nox::noun::{Order, NounId, Noun};
use nox::jets::formulas::{
    build_poly_eval_formula,
    build_merkle_verify_formula,
    build_fri_fold_formula,
    build_ntt_formula,
    build_cyberlink_formula,
    build_decider_formula,
};

const N: usize = 1 << 14; // 16 K nodes — enough for all six formulas

fn print_noun(order: &Order<N>, id: NounId) -> String {
    match order.get(id).map(|e| e.inner) {
        Some(Noun::Atom { value, .. }) => value.as_u64().to_string(),
        Some(Noun::Cell { left, right }) => {
            format!("[{} {}]", print_noun(order, left), print_noun(order, right))
        }
        None => "<invalid>".to_string(),
    }
}

fn dump(order: &Order<N>, name: &str, formula: Option<NounId>) {
    match formula {
        Some(id) => {
            println!("=== {} ===", name);
            println!("{}", print_noun(order, id));
            println!();
        }
        None => eprintln!("error: {} formula build failed (order full?)", name),
    }
}

fn main() {
    let mut order = Order::<N>::new();

    let poly_eval     = build_poly_eval_formula(&mut order);
    let merkle_verify = build_merkle_verify_formula(&mut order);
    let fri_fold      = build_fri_fold_formula(&mut order);
    let ntt           = build_ntt_formula(&mut order);
    let cyberlink     = build_cyberlink_formula(&mut order);
    let decider       = build_decider_formula(&mut order);

    dump(&order, "poly_eval",     poly_eval);
    dump(&order, "merkle_verify", merkle_verify);
    dump(&order, "fri_fold",      fri_fold);
    dump(&order, "ntt",           ntt);
    dump(&order, "cyberlink",     cyberlink);
    dump(&order, "decider",       decider);
}
