// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! nox — run noun formulas from stdin or file
//!
//! Usage:
//!   nox <file.nox>              # evaluate formula from file
//!   nox -e '[5 [[1 3] [1 5]]]' # evaluate inline formula
//!   echo '[1 42]' | nox        # evaluate from stdin
//!
//! Object defaults to atom 0. Override with --object.
//! Budget defaults to 1_000_000. Override with --budget.

use std::io::Read;

use nebu::Goldilocks;
use nox::noun::{Order, NounId, Noun, Tag};
use nox::reduce::{reduce, Outcome};
use nox::call::NullCalls;
use nox::trace::NoTrace;

const ORDER_SIZE: usize = 1 << 16; // 64K nouns

// ─── noun parser ─────────────────────────────────────────────────

/// Parse a textual noun `[a b]` or `42` into the order.
fn parse_noun(order: &mut Order<ORDER_SIZE>, input: &str) -> Result<NounId, String> {
    let tokens = tokenize(input)?;
    let mut pos = 0;
    let result = parse_expr(order, &tokens, &mut pos, 0)?;
    Ok(result)
}

/// Cap parse_expr recursion. Adversarial input like `[[[[…` would otherwise
/// blow the 32 MB worker stack at ~250k levels in debug.
const MAX_PARSE_DEPTH: usize = 4096;

#[derive(Debug)]
enum Token {
    Open,
    Close,
    Num(u64),
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(&ch) = chars.peek() {
        match ch {
            '[' => { tokens.push(Token::Open); chars.next(); }
            ']' => { tokens.push(Token::Close); chars.next(); }
            ' ' | '\t' | '\n' | '\r' => { chars.next(); }
            '0'..='9' => {
                let mut num = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() || c == '_' {
                        if c != '_' { num.push(c); }
                        chars.next();
                    } else {
                        break;
                    }
                }
                let v: u64 = num.parse().map_err(|e| format!("bad number '{}': {}", num, e))?;
                tokens.push(Token::Num(v));
            }
            _ => { chars.next(); } // skip unknown chars
        }
    }
    Ok(tokens)
}

fn parse_expr(
    order: &mut Order<ORDER_SIZE>,
    tokens: &[Token],
    pos: &mut usize,
    depth: usize,
) -> Result<NounId, String> {
    if depth > MAX_PARSE_DEPTH {
        return Err(format!("nesting depth exceeds {}", MAX_PARSE_DEPTH));
    }
    if *pos >= tokens.len() {
        return Err("unexpected end of input".to_string());
    }
    match &tokens[*pos] {
        Token::Num(v) => {
            *pos += 1;
            order.atom(Goldilocks::new(*v), Tag::Field)
                .ok_or_else(|| "order full".to_string())
        }
        Token::Open => {
            *pos += 1; // skip [
            // Parse all elements inside brackets, right-nest them.
            // [a b c d] → Cell(a, Cell(b, Cell(c, d)))
            let mut elems = Vec::new();
            while *pos < tokens.len() && !matches!(tokens[*pos], Token::Close) {
                elems.push(parse_expr(order, tokens, pos, depth + 1)?);
            }
            if *pos < tokens.len() && matches!(tokens[*pos], Token::Close) {
                *pos += 1;
            } else {
                return Err("expected ']'".to_string());
            }
            if elems.is_empty() {
                return Err("empty brackets".to_string());
            }
            if elems.len() == 1 {
                return Ok(elems[0]);
            }
            // Right-nest: [a b c] → Cell(a, Cell(b, c))
            let mut result = elems.pop().unwrap();
            while let Some(head) = elems.pop() {
                result = order.cell(head, result)
                    .ok_or_else(|| "order full".to_string())?;
            }
            Ok(result)
        }
        Token::Close => {
            Err("unexpected ']'".to_string())
        }
    }
}

// ─── noun printer ────────────────────────────────────────────────

fn print_noun(order: &Order<ORDER_SIZE>, r: NounId) -> String {
    match order.get(r).map(|e| e.inner) {
        Some(Noun::Atom { value, .. }) => format!("{}", value.as_u64()),
        Some(Noun::Cell { left, right }) => {
            format!("[{} {}]", print_noun(order, left), print_noun(order, right))
        }
        None => "<invalid>".to_string(),
    }
}

// ─── main ────────────────────────────────────────────────────────

fn main() {
    // Order<65536> is ~6 MB — too large for the default 8 MB main thread stack.
    // Spawn a thread with an explicit 32 MB stack so the heap-allocated order is safe.
    let exit_code = std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(run)
        .expect("failed to spawn nox thread")
        .join()
        .expect("nox thread panicked");
    std::process::exit(exit_code);
}

fn run() -> i32 {
    let args: Vec<String> = std::env::args().collect();

    let mut formula_text = String::new();
    let mut object_text = String::from("0");
    let mut budget: u64 = 1_000_000;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-e" => {
                i += 1;
                if i < args.len() {
                    formula_text = args[i].clone();
                } else {
                    eprintln!("error: -e requires an argument");
                    return 1;
                }
            }
            "--object" | "-s" => {
                i += 1;
                if i < args.len() {
                    object_text = args[i].clone();
                } else {
                    eprintln!("error: --object requires an argument");
                    return 1;
                }
            }
            "--budget" | "-b" => {
                i += 1;
                if i < args.len() {
                    match args[i].parse::<u64>() {
                        Ok(b) => budget = b,
                        Err(_) => { eprintln!("error: invalid budget"); return 1; }
                    }
                } else {
                    eprintln!("error: --budget requires an argument");
                    return 1;
                }
            }
            "--help" | "-h" => {
                print_usage();
                return 0;
            }
            other => {
                match std::fs::read_to_string(other) {
                    Ok(s) => formula_text = s,
                    Err(e) => { eprintln!("error reading '{}': {}", other, e); return 1; }
                }
            }
        }
        i += 1;
    }

    if formula_text.is_empty() {
        use std::io::IsTerminal;
        if std::io::stdin().is_terminal() {
            print_usage();
            return 1;
        }
        if let Err(e) = std::io::stdin().read_to_string(&mut formula_text) {
            eprintln!("error reading stdin: {}", e);
            return 1;
        }
    }

    let formula_text = formula_text.trim().to_string();
    if formula_text.is_empty() {
        eprintln!("error: no formula provided");
        return 1;
    }

    let mut order = Order::<ORDER_SIZE>::new();
    let hints = NullCalls;

    let object = match parse_noun(&mut order, &object_text) {
        Ok(n) => n,
        Err(e) => { eprintln!("error parsing object: {}", e); return 1; }
    };

    let formula = match parse_noun(&mut order, &formula_text) {
        Ok(n) => n,
        Err(e) => { eprintln!("error parsing formula: {}", e); return 1; }
    };

    match reduce(&mut order, object, formula, budget, &hints, &mut NoTrace) {
        Outcome::Ok(result, remaining) => {
            println!("{}", print_noun(&order, result));
            eprintln!("cost: {} (budget remaining: {})", budget - remaining, remaining);
            0
        }
        Outcome::Halt(remaining) => {
            eprintln!("halted: budget exhausted (remaining: {})", remaining);
            1
        }
        Outcome::Error(kind) => {
            eprintln!("error: {:?}", kind);
            1
        }
    }
}

fn print_usage() {
    eprintln!(
        "\
\x1b[31m
    ███╗   ██╗ ██████╗ ██╗  ██╗
\x1b[33m    ████╗  ██║██╔═══██╗╚██╗██╔╝
\x1b[32m    ██╔██╗ ██║██║   ██║ ╚███╔╝
\x1b[36m    ██║╚██╗██║██║   ██║ ██╔██╗
\x1b[34m    ██║ ╚████║╚██████╔╝██╔╝ ██╗
\x1b[35m    ╚═╝  ╚═══╝ ╚═════╝ ╚═╝  ╚═╝
\x1b[0m\x1b[37m    the VM for superintelligence\x1b[0m
\x1b[90m
    16 patterns · 36 genesis jets · 5 algebras
    Goldilocks field · p = 2^64 - 2^32 + 1
    computation IS linking · proof-native
\x1b[0m
  nox <file.nox>                evaluate formula from file
  nox -e '[5 [[1 3] [1 5]]]'    evaluate inline formula
  echo '[1 42]' | nox            evaluate from stdin

  -s, --object <noun>    object (default: 0)
  -b, --budget <n>        budget (default: 1000000)
  -e <formula>            inline formula
"
    );
}
