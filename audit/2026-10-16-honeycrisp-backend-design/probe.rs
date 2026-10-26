//! Diagnostic only: exact upstream APIs, independent arithmetic, existing Nox callers.
use nebu::Goldilocks;
use nox::data::Data;
use nox::jets::registry::{JetFn, compute_genesis_digests};
use nox::{NullCalls, Order, Outcome, Reduction, TraceRow, VecTrace};

const P: u64 = 0xffff_ffff_0000_0001;
const ARENA: usize = 512;

fn add(a: u64, b: u64) -> u64 {
    ((u128::from(a) + u128::from(b)) % u128::from(P)) as u64
}
fn mul(a: u64, b: u64) -> u64 {
    (u128::from(a) * u128::from(b) % u128::from(P)) as u64
}
fn sub(a: u64, b: u64) -> u64 {
    ((u128::from(a) + u128::from(P) - u128::from(b)) % u128::from(P)) as u64
}
fn pow(mut a: u64, mut e: u64) -> u64 {
    let mut r = 1;
    while e != 0 {
        if e & 1 != 0 {
            r = mul(r, a);
        }
        a = mul(a, a);
        e >>= 1;
    }
    r
}
fn root(n: usize) -> u64 {
    let omega = pow(7, (P - 1) / n as u64);
    assert_eq!(pow(omega, n as u64), 1);
    if n > 1 {
        assert_ne!(pow(omega, (n / 2) as u64), 1);
    }
    omega
}
fn dft(values: &[u64], omega: u64) -> Vec<u64> {
    (0..values.len())
        .map(|j| {
            values.iter().enumerate().fold(0, |sum, (i, &v)| {
                add(sum, mul(v, pow(omega, (i * j) as u64)))
            })
        })
        .collect()
}
fn hypercube(evals: &[u64], point: &[u64]) -> u64 {
    evals.iter().enumerate().fold(0, |sum, (index, &v)| {
        let weight = point.iter().enumerate().fold(1, |weight, (j, &x)| {
            let bit = (index >> (point.len() - j - 1)) & 1;
            mul(weight, if bit == 0 { sub(1, x) } else { x })
        });
        add(sum, mul(v, weight))
    })
}
fn boundary_values(n: usize) -> Vec<u64> {
    let values = [0, 1, P - 1, P - 2, 0xffff_ffff, 0x1_0000_0000, 7, P / 2];
    (0..n).map(|i| values[i % values.len()]).collect()
}

#[test]
fn upstream_ntt_matches_independent_dft_in_both_directions() {
    let mut cases = 0;
    for log_n in 0..=5 {
        let n = 1 << log_n;
        let mut first = vec![0; n];
        first[0] = 1;
        let mut last = vec![0; n];
        last[n - 1] = 1;
        for values in [vec![0; n], first, last, boundary_values(n)] {
            for omega in [root(n), pow(root(n), P - 2)] {
                let mut actual = values.clone();
                acpu::field::ntt_forward(&mut actual, omega);
                let actual: Vec<_> = actual.into_iter().map(|v| v % P).collect();
                assert_eq!(actual, dft(&values, omega));
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 48);
}

#[test]
fn upstream_multilinear_matches_independent_hypercube_sum() {
    let mut cases = 0;
    for k in 0..=5 {
        let n = 1 << k;
        for evals in [(0..n as u64).collect::<Vec<_>>(), boundary_values(n)] {
            let small = (0..k).map(|j| j as u64 + 2).collect();
            let large = (0..k).map(|j| P - 1 - j as u64).collect();
            for point in [vec![0; k], vec![1; k], small, large] {
                assert_eq!(
                    acpu::field::multilinear_eval(&evals, &point) % P,
                    hypercube(&evals, &point)
                );
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 48);
}

#[test]
fn raw_field_outputs_can_be_noncanonical() {
    let mut values = [P - 1, 1];
    acpu::field::ntt_forward(&mut values, P - 1);
    assert_eq!(values[0], P);
    assert_eq!(Goldilocks::new(values[0]).as_u64(), 0);
    let value = acpu::field::multilinear_eval(&[P - 1, 1], &[1]);
    assert_eq!(value, P + 1);
    assert_eq!(Goldilocks::new(value).as_u64(), 1);
}

fn tree(ar: &mut Reduction<ARENA>, values: &[u64]) -> Order {
    if values.len() == 1 {
        return ar.atom(Goldilocks::new(values[0])).unwrap();
    }
    let mid = values.len() / 2;
    let left = tree(ar, &values[..mid]);
    let right = tree(ar, &values[mid..]);
    ar.pair(left, right).unwrap()
}
fn flatten(ar: &Reduction<ARENA>, id: Order, output: &mut Vec<u64>) {
    match ar.get(id).unwrap().inner {
        Data::Atom { .. } => output.push(ar.atom_value(id).unwrap().as_u64()),
        Data::Pair { left, right } => {
            flatten(ar, left, output);
            flatten(ar, right, output);
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
struct Observation {
    outcome: String,
    data: Vec<u64>,
    row: [u64; 16],
    trace: Vec<[u64; 16]>,
}
fn observe(jet: JetFn<ARENA>, values: &[u64], point: Option<&[u64]>, budget: u64) -> Observation {
    let mut ar = Reduction::new();
    let log = ar
        .atom(Goldilocks::new(values.len().trailing_zeros() as u64))
        .unwrap();
    let zero = ar.atom(Goldilocks::ZERO).unwrap();
    let lhs = ar.pair(log, zero).unwrap();
    let vals = tree(&mut ar, values);
    let parameter = match point {
        None => ar.atom(Goldilocks::new(root(values.len()))).unwrap(),
        Some(point) => point.iter().rev().fold(zero, |tail, &x| {
            let head = ar.atom(Goldilocks::new(x)).unwrap();
            ar.pair(head, tail).unwrap()
        }),
    };
    let rhs = ar.pair(vals, parameter).unwrap();
    let object = ar.pair(lhs, rhs).unwrap();
    let mut row = TraceRow::default();
    let mut tracer = VecTrace::default();
    let outcome = jet(
        &mut ar,
        object,
        zero,
        budget,
        &NullCalls,
        &mut tracer,
        0,
        &mut row,
    );
    let mut data = Vec::new();
    if let Outcome::Ok(id, _) = outcome {
        flatten(&ar, id, &mut data);
        let expected = match point {
            None => dft(values, root(values.len())),
            Some(point) => vec![hypercube(values, point)],
        };
        assert_eq!(data, expected);
    }
    Observation {
        outcome: format!("{outcome:?}"),
        data,
        row: *row.r(),
        trace: tracer.0.iter().map(|row| *row.r()).collect(),
    }
}

#[test]
fn nox_source_backend_matches_cpu_result_budget_and_trace() {
    let cpu = nox::jets::backends::cpu::genesis_cpu::<ARENA>();
    let hc = nox::jets::backends::honeycrisp::genesis_honeycrisp::<ARENA>();
    let ids = compute_genesis_digests();
    let mut cases = 0;
    for k in 0..=4 {
        let values = boundary_values(1 << k);
        let point: Vec<_> = (0..k).map(|j| P - 1 - j as u64).collect();
        for (id, parameter, cost) in [
            (ids.ntt, None, k as u64 * values.len() as u64),
            (ids.poly_eval, Some(point.as_slice()), values.len() as u64),
        ] {
            for budget in [0, cost.saturating_sub(1), cost, cost + 13] {
                assert_eq!(
                    observe(cpu.lookup_exact(&id).unwrap(), &values, parameter, budget),
                    observe(hc.lookup_exact(&id).unwrap(), &values, parameter, budget)
                );
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 40);
}
