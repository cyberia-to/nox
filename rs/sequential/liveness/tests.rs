use super::*;
use crate::data::{Cost, reduction::DataEntry};
use nebu::Goldilocks;
use std::{collections::BTreeSet, io::Read};

fn frame(phase: Phase, object: Order) -> Frame {
    let mut row = TraceRow::default();
    row.r[1] = u64::from(object);
    row.r[2] = 999_999; // dead trace-only reference must never become a root
    Frame {
        row,
        budget: 100,
        phase,
    }
}

#[test]
fn phases_retain_only_values_read_after_the_child_returns() {
    let reservation = Reservation {
        parent: 100,
        child: 10,
    };
    let phases = [
        (Phase::Unary(reservation), std::vec![]),
        (Phase::Compose, std::vec![]),
        (Phase::BranchChosen(reservation), std::vec![]),
        (
            Phase::BinaryLeft {
                a: 21,
                b: 22,
                first: 10,
                second: Some(10),
            },
            std::vec![20, 22],
        ),
        (
            Phase::BinaryRight {
                a: 21,
                b: 22,
                left: 23,
                used: 4,
                second: 10,
            },
            std::vec![23],
        ),
        (
            Phase::BranchTest {
                yes: 24,
                no: 25,
                reservation,
            },
            std::vec![20, 24, 25],
        ),
    ];
    for (phase, expected) in phases {
        let f = frame(phase, 20);
        let mut roots = Vec::new();
        frame_roots(&f).each(|id| roots.push(id));
        assert_eq!(roots, expected);
        let action = Action::Return(Outcome::Ok(26, 3));
        let retained = step_roots(&action, &[f]);
        let mut actual = Vec::new();
        retained.each(|id| actual.push(id));
        assert_eq!(actual[0], 26);
        assert_eq!(&actual[1..], expected);
    }
}

fn reference<const N: usize>(ar: &Reduction<N>, mut pending: Vec<Order>) -> usize {
    let mut seen = BTreeSet::new();
    while let Some(id) = pending.pop() {
        if !seen.insert(id) {
            continue;
        }
        if let Data::Pair { left, right } = ar.get(id).unwrap().inner {
            pending.push(left);
            pending.push(right);
        }
    }
    seen.len()
}

#[test]
fn descending_marks_match_independent_walk_with_pins_and_failed_operands() {
    for seed in 1..64 {
        let mut ar = Reduction::<256>::new();
        for i in 0..16 {
            ar.atom(Goldilocks::new(i)).unwrap();
        }
        let pinned = ar.count();
        let mut state: u64 = seed;
        for _ in 0..64 {
            state = state.wrapping_mul(6364136223846793005u64).wrapping_add(1);
            let a = (state >> 32) as u32 % ar.count();
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let b = (state >> 32) as u32 % ar.count();
            ar.pair(a, b).unwrap();
        }
        let left = ar.count() - 7;
        let result = ar.count() - 1;
        let f = frame(
            Phase::BinaryRight {
                a: 999,
                b: 999,
                left,
                used: 0,
                second: 50,
            },
            999,
        );
        let active = step_roots(&Action::Return(Outcome::Ok(result, 40)), &[f]);
        let ancestors = [
            frame(Phase::Compose, 999),
            frame(
                Phase::Unary(Reservation {
                    parent: 10,
                    child: 9,
                }),
                999,
            ),
        ];
        let actual = census(
            &ar,
            pinned,
            &ancestors,
            active,
            Instant::now() + Duration::from_secs(5),
        )
        .unwrap();
        let mut roots: Vec<_> = (0..pinned).collect();
        active.each(|id| roots.push(id));
        assert_eq!(actual.live as usize, reference(&ar, roots));
        assert!(actual.live < ar.count());
        assert_eq!(actual.bitset_bytes, (ar.count() as usize).div_ceil(64) * 8);
    }
}

#[test]
fn invalid_roots_non_topological_pairs_and_elapsed_deadline_reject_measurement() {
    let mut ar = Reduction::<16>::new();
    let a = ar.atom(Goldilocks::ZERO).unwrap();
    let mut roots = Roots::empty();
    roots.add(1);
    let future = Instant::now() + Duration::from_secs(5);
    assert!(census(&ar, 0, &[], roots, future).is_err());
    let bad = ar
        .alloc_raw(DataEntry {
            inner: Data::Pair { left: 1, right: a },
            hash: [Goldilocks::ZERO; 4],
            bound: Cost::Exact(0),
        })
        .unwrap();
    assert_eq!(bad, 1);
    assert!(census(&ar, 0, &[], roots, future).is_err());
    assert!(census(&ar, 0, &[], Roots::empty(), future).is_err());
    let mut roots = Roots::empty();
    roots.add(a);
    let mut valid = Reduction::<16>::new();
    valid.atom(Goldilocks::ZERO).unwrap();
    assert!(census(&valid, 0, &[], roots, Instant::now()).is_err());
}

fn read(path: &PathBuf, max_bytes: usize) -> Vec<u8> {
    let file = std::fs::File::open(path).unwrap();
    let metadata = file.metadata().unwrap();
    assert!(metadata.is_file() && metadata.len() <= max_bytes as u64);
    let mut bytes = Vec::new();
    file.take(max_bytes as u64 + 1)
        .read_to_end(&mut bytes)
        .unwrap();
    assert!(bytes.len() <= max_bytes);
    bytes
}

#[test]
#[ignore = "explicit multi-gigabyte diagnostic; requires NOX_LIVENESS_* paths"]
fn compiler_stage_four_liveness() {
    std::thread::Builder::new().stack_size(256 << 20).spawn(|| {
        let output = PathBuf::from(std::env::var_os("NOX_LIVENESS_OUTPUT").unwrap());
        assert!(!output.exists());
        let report = output.with_extension("execution.json");
        assert!(!report.exists());
        let program = PathBuf::from(std::env::var_os("NOX_LIVENESS_PROGRAM").unwrap());
        let job = PathBuf::from(std::env::var_os("NOX_LIVENESS_JOB").unwrap());
        let codec = crate::artifact::Limits { max_bytes: 16 << 20, max_nodes: 196_608, max_depth: 4096 };
        let mut ar = Reduction::<{ 1 << 25 }>::try_new_boxed().unwrap();
        assert!(ar.limit_allocations(25_165_824));
        let program = crate::artifact::decode(&mut ar, &read(&program, codec.max_bytes), codec).unwrap();
        let job = crate::artifact::decode(&mut ar, &read(&job, codec.max_bytes), codec).unwrap();
        let stage = ar.atom(Goldilocks::new(4)).unwrap();
        let input = ar.pair(job, stage).unwrap();
        let mut body = ar.tail(program).unwrap();
        for _ in 0..3 { body = ar.tail(body).unwrap(); }
        let formula = ar.head(body).unwrap();
        let pinned = ar.count();
        assert_eq!(pinned, 159_037);
        let start = Instant::now();
        let mut checkpoints = 0u64;
        let mut next_report = Duration::from_secs(60);
        let run = reduce_cached_controlled(&mut ar, input, formula, 10_000_000_000, Limits { max_frames: 65_536 }, &mut || {
            checkpoints += 1;
            let elapsed = start.elapsed();
            if elapsed >= next_report {
                std::eprintln!("liveness probe: {}s, {checkpoints} evaluator checkpoints", elapsed.as_secs());
                next_report += Duration::from_secs(60);
            }
            elapsed >= Duration::from_secs(1200)
        }).unwrap();
        let elapsed_micros = start.elapsed().as_micros();
        assert!(matches!(run.outcome, Outcome::Error(ErrorKind::Unavailable)));
        assert_eq!(ar.count(), 25_165_824);
        assert_eq!(run.peak_frames, 3591);
        assert_eq!(checkpoints, 1_091_810_645);
        assert!(output.exists());
        std::fs::write(report, std::format!(
            "{{\n  \"outcome\": \"Error(Unavailable)\",\n  \"total_nodes\": {},\n  \"preloaded_nodes\": {},\n  \"evaluator_checkpoints\": {},\n  \"peak_frames\": {},\n  \"frame_buffer_bytes\": {},\n  \"arena_reserved_bytes\": {},\n  \"cache_bytes\": {},\n  \"elapsed_micros\": {},\n  \"charged_reductions\": null\n}}\n",
            ar.count(), pinned, checkpoints, run.peak_frames, frame_storage_bytes(Limits { max_frames: 65_536 }).unwrap(),
            core::mem::size_of::<Reduction<{ 1 << 25 }>>(), finalizer_cache_storage_bytes(), elapsed_micros,
        )).unwrap();
    }).unwrap().join().unwrap();
}
