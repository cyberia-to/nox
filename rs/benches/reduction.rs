// ---
// tags: nox, bench
// crystal-type: source
// crystal-domain: comp
// ---
//! Criterion benchmarks for nox reduction patterns and jets.

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use nebu::Goldilocks;
use nox::data::{Order};
use nox::call::NullCalls;
use nox::trace::NoTrace;
use nox::reduce::reduce;
use nox::trace::TraceRow;
use nox::encode::{encode_tree, encode_atom, particle_of};
use nox::jets::merkle_verify::merkle_verify_jet;

fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

// ── arithmetic patterns ──────────────────────────────────────────────────────

fn bench_add(c: &mut Criterion) {
    c.bench_function("pattern/add", |b| {
        b.iter(|| {
            let mut ar = Order::<4096>::new();
            let obj  = ar.atom(g(0)).unwrap();
            let t5   = ar.atom(g(5)).unwrap();
            let t1   = ar.atom(g(1)).unwrap();
            let va   = ar.atom(g(12345678)).unwrap();
            let vb   = ar.atom(g(87654321)).unwrap();
            let qa   = ar.pair(t1, va).unwrap();
            let qb   = ar.pair(t1, vb).unwrap();
            let body = ar.pair(qa, qb).unwrap();
            let fml  = ar.pair(t5, body).unwrap();
            black_box(reduce(&mut ar, obj, fml, 1_000_000, &NullCalls, &mut NoTrace))
        })
    });
}

fn bench_mul(c: &mut Criterion) {
    c.bench_function("pattern/mul", |b| {
        b.iter(|| {
            let mut ar = Order::<4096>::new();
            let obj  = ar.atom(g(0)).unwrap();
            let t7   = ar.atom(g(7)).unwrap();
            let t1   = ar.atom(g(1)).unwrap();
            let va   = ar.atom(g(999_999_999)).unwrap();
            let vb   = ar.atom(g(111_111_111)).unwrap();
            let qa   = ar.pair(t1, va).unwrap();
            let qb   = ar.pair(t1, vb).unwrap();
            let body = ar.pair(qa, qb).unwrap();
            let fml  = ar.pair(t7, body).unwrap();
            black_box(reduce(&mut ar, obj, fml, 1_000_000, &NullCalls, &mut NoTrace))
        })
    });
}

fn bench_inv(c: &mut Criterion) {
    c.bench_function("pattern/inv", |b| {
        b.iter(|| {
            let mut ar = Order::<4096>::new();
            let obj  = ar.atom(g(0)).unwrap();
            let t8   = ar.atom(g(8)).unwrap();
            let t1   = ar.atom(g(1)).unwrap();
            let va   = ar.atom(g(999_999_937)).unwrap();
            let qa   = ar.pair(t1, va).unwrap();
            let fml  = ar.pair(t8, qa).unwrap();
            black_box(reduce(&mut ar, obj, fml, 1_000_000, &NullCalls, &mut NoTrace))
        })
    });
}

// ── hash pattern (hashrate) ──────────────────────────────────────────────────

fn bench_hash(c: &mut Criterion) {
    // Each iteration is one Poseidon2 hash (25 rows: 24 rounds + 1 squeeze).
    // Throughput = hashes/sec.
    let mut group = c.benchmark_group("hash");
    group.throughput(Throughput::Elements(1));

    group.bench_function("pattern/hash", |b| {
        b.iter(|| {
            let mut ar = Order::<4096>::new();
            let obj   = ar.atom(g(0)).unwrap();
            let t15   = ar.atom(g(15)).unwrap();
            let t1    = ar.atom(g(1)).unwrap();
            let input = ar.atom(g(42)).unwrap();
            let qi    = ar.pair(t1, input).unwrap();
            let fml   = ar.pair(t15, qi).unwrap();
            black_box(reduce(&mut ar, obj, fml, 1_000_000, &NullCalls, &mut NoTrace))
        })
    });

    // Batch: chain N hashes in one reduce call to amortize Order allocation.
    // Measures sustained hashrate under real workload pressure.
    const CHAIN: u64 = 64;
    group.throughput(Throughput::Elements(CHAIN));
    group.bench_function("pattern/hash_chain_64", |b| {
        b.iter(|| {
            let mut ar = Order::<4096>::new();
            let obj   = ar.atom(g(0)).unwrap();
            let t15   = ar.atom(g(15)).unwrap();
            let t1    = ar.atom(g(1)).unwrap();
            // Build a chain: hash(hash(hash(...hash(42)...))) 64 deep via compose.
            // compose tag = 3; hash tag = 15.
            let t3    = ar.atom(g(3)).unwrap();
            let input = ar.atom(g(42)).unwrap();
            let qi    = ar.pair(t1, input).unwrap();
            let mut fml = ar.pair(t15, qi).unwrap();
            for _ in 1..CHAIN {
                let placeholder = ar.atom(g(0)).unwrap();
                let inner = ar.pair(t1, placeholder).unwrap();
                let hash_fml = ar.pair(t15, inner).unwrap();
                let pair = ar.pair(hash_fml, fml).unwrap();
                fml = ar.pair(t3, pair).unwrap();
            }
            black_box(reduce(&mut ar, obj, fml, 1_000_000_000, &NullCalls, &mut NoTrace))
        })
    });

    group.finish();
}

// ── look pattern (null provider) ─────────────────────────────────────────────

fn bench_look(c: &mut Criterion) {
    c.bench_function("pattern/look_null", |b| {
        b.iter(|| {
            let mut ar = Order::<4096>::new();
            let l0   = ar.atom(g(0)).unwrap();
            let l1   = ar.atom(g(0)).unwrap();
            let l2   = ar.atom(g(0)).unwrap();
            let l3   = ar.atom(g(0)).unwrap();
            let inner = ar.pair(l2, l3).unwrap();
            let mid   = ar.pair(l1, inner).unwrap();
            let rc    = ar.pair(l0, mid).unwrap();
            let rest  = ar.atom(g(0)).unwrap();
            let obj   = ar.pair(rc, rest).unwrap();
            let t17   = ar.atom(g(17)).unwrap();
            let t1    = ar.atom(g(1)).unwrap();
            let vns   = ar.atom(g(0)).unwrap();
            let vkey  = ar.atom(g(42)).unwrap();
            let ns_f  = ar.pair(t1, vns).unwrap();
            let key_f = ar.pair(t1, vkey).unwrap();
            let body  = ar.pair(ns_f, key_f).unwrap();
            let fml   = ar.pair(t17, body).unwrap();
            black_box(reduce(&mut ar, obj, fml, 1_000_000, &NullCalls, &mut NoTrace))
        })
    });
}

// ── jet: merkle_verify ────────────────────────────────────────────────────────

fn bench_merkle_verify(c: &mut Criterion) {
    use hemera::tree::{hash_leaf, hash_node};
    use hemera::Hash;

    let leaf0 = hash_leaf(b"leaf0", 0, false);
    let leaf1 = hash_leaf(b"leaf1", 1, false);
    let root  = hash_node(&leaf0, &leaf1, true);

    let build_hash_data = |ar: &mut Order<4096>, hash: &Hash| {
        let bytes = hash.as_bytes();
        let mut limb = |off: usize| -> u32 {
            let mut buf = [0u8; 8];
            buf.copy_from_slice(&bytes[off..off + 8]);
            ar.atom(g(u64::from_le_bytes(buf))).unwrap()
        };
        let h0 = limb(0); let h1 = limb(8);
        let h2 = limb(16); let h3 = limb(24);
        let left  = ar.pair(h0, h1).unwrap();
        let right = ar.pair(h2, h3).unwrap();
        ar.pair(left, right).unwrap()
    };

    c.bench_function("jet/merkle_verify", |b| {
        b.iter(|| {
            let mut ar = Order::<4096>::new();
            // object = [[depth | [root | formula]] | [current | path]]
            let root_data = build_hash_data(&mut ar, &root);
            let leaf_data = build_hash_data(&mut ar, &leaf0);
            let sib_data  = build_hash_data(&mut ar, &leaf1);
            let depth_id  = ar.atom(g(1)).unwrap();
            let formula   = ar.atom(g(0)).unwrap();
            let root_fml  = ar.pair(root_data, formula).unwrap();
            let meta      = ar.pair(depth_id, root_fml).unwrap();
            let dir       = ar.atom(g(0)).unwrap(); // left sibling
            let step      = ar.pair(sib_data, dir).unwrap();
            let term      = ar.atom(g(0)).unwrap();
            let path      = ar.pair(step, term).unwrap();
            let data      = ar.pair(leaf_data, path).unwrap();
            let obj       = ar.pair(meta, data).unwrap();
            let body      = ar.atom(g(0)).unwrap();
            let mut row   = TraceRow::default();
            black_box(merkle_verify_jet(&mut ar, obj, body, 1_000_000, &NullCalls, &mut NoTrace, 0, &mut row))
        })
    });
}

// ── encode_tree ───────────────────────────────────────────────────────────────

fn bench_encode_tree(c: &mut Criterion) {
    let mut ar = Order::<4096>::new();
    let mut cur = ar.atom(g(0)).unwrap();
    for i in 1u64..=64 {
        let a = ar.atom(g(i)).unwrap();
        cur = ar.pair(a, cur).unwrap();
    }
    let root = cur;

    c.bench_function("encode/tree_64", |b| {
        b.iter(|| {
            black_box(encode_tree(&ar, root))
        })
    });
}

fn bench_particle(c: &mut Criterion) {
    let encoded = encode_atom(g(12345));
    c.bench_function("encode/particle_atom", |b| {
        b.iter(|| {
            black_box(particle_of(&encoded))
        })
    });
}

criterion_group!(
    benches,
    bench_add,
    bench_mul,
    bench_inv,
    bench_hash,
    bench_look,
    bench_merkle_verify,
    bench_encode_tree,
    bench_particle,
);
criterion_main!(benches);
