//! Explicit component-only frontier measurements; no production compiler stages.
use super::*;
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    string::String,
    time::{Duration, Instant},
};

const BUDGET: u64 = 10_000_000_000;
const TOTAL_ALLOCATIONS: u64 = 1_000_000_000;
const COLLECTION_WORK: u64 = 10_000_000_000;
const RESIDENT: u32 = 3_145_728;
const SECONDS: u64 = 3600;

fn measurement_metadata() -> &'static str {
    "  \"measurement_reference_revision\": \"1eaa8a494f994e4da6b20509469c1e62624f3f2d\",\n  \"runtime_identity_source\": \"external source and binary manifest required\","
}

fn write_new(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    file.write_all(bytes)
}

fn quoted(value: &str) -> String {
    let mut json = String::from("\"");
    for c in value.chars() {
        match c {
            '"' => json.push_str("\\\""),
            '\\' => json.push_str("\\\\"),
            '\n' => json.push_str("\\n"),
            '\r' => json.push_str("\\r"),
            '\t' => json.push_str("\\t"),
            c if c < ' ' => json.push_str(&std::format!("\\u{:04x}", c as u32)),
            c => json.push(c),
        }
    }
    json.push('"');
    json
}

fn scalar(value: Option<u64>) -> String {
    value
        .map(|v| std::format!("{v}"))
        .unwrap_or_else(|| String::from("null"))
}

fn read(path: &PathBuf, max: usize) -> Vec<u8> {
    let file = std::fs::File::open(path).expect("open bounded artifact");
    let metadata = file.metadata().expect("artifact metadata");
    assert!(metadata.is_file() && metadata.len() <= max as u64);
    let mut bytes = Vec::new();
    file.take(max as u64 + 1)
        .read_to_end(&mut bytes)
        .expect("bounded artifact read");
    assert!(bytes.len() <= max);
    bytes
}

fn particle<const N: usize>(ar: &Reduction<N>, id: Order) -> String {
    crate::data::digest_bytes(ar.digest(id).expect("input particle"))
        .iter()
        .map(|b| std::format!("{b:02x}"))
        .collect()
}

fn five_words<const N: usize>(ar: &Reduction<N>, mut rest: Order) -> Option<[u64; 5]> {
    let mut words = [0; 5];
    for word in &mut words[..4] {
        *word = ar.atom_value(ar.head(rest)?)?.as_u64();
        rest = ar.tail(rest)?;
    }
    words[4] = ar.atom_value(rest)?.as_u64();
    Some(words)
}

#[test]
fn receipt_strings_escape_control_characters_and_failure_cost_is_null() {
    assert_eq!(quoted("a\n\"\\\u{7}"), "\"a\\n\\\"\\\\\\u0007\"");
    assert_eq!(scalar(None), "null");
    assert_eq!(scalar(Some(55)), "55");
}

#[test]
fn receipt_revision_is_a_reference_and_requires_external_runtime_identity() {
    let metadata = measurement_metadata();
    assert!(metadata.contains("\"measurement_reference_revision\":"));
    assert!(!metadata.contains("\"production_revision\":"));
    assert!(
        metadata.contains(
            "\"runtime_identity_source\": \"external source and binary manifest required\""
        )
    );
}

#[test]
fn receipt_writer_preserves_a_file_created_after_the_initial_check() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory = std::env::temp_dir().join(std::format!(
        "nox-frontier-writer-{}-{unique}",
        std::process::id()
    ));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("receipt.json");
    assert!(!path.exists());
    write_new(&path, b"first writer").unwrap();
    let error = write_new(&path, b"later writer").unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
    assert_eq!(std::fs::read(&path).unwrap(), b"first writer");
    std::fs::remove_file(&path).unwrap();
    std::fs::remove_dir(&directory).unwrap();
}

#[test]
#[ignore = "explicit full-source prefix frontier; requires NOX_FRONTIER_* inputs"]
fn measure_prefix_frontier() {
    std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(run)
        .unwrap()
        .join()
        .unwrap();
}

fn run() {
    let program_path = PathBuf::from(std::env::var_os("NOX_FRONTIER_PROGRAM").unwrap());
    let job_path = PathBuf::from(std::env::var_os("NOX_FRONTIER_JOB").unwrap());
    let output = PathBuf::from(std::env::var_os("NOX_FRONTIER_OUTPUT").unwrap());
    let stage: u32 = std::env::var("NOX_FRONTIER_STAGE")
        .unwrap()
        .parse()
        .unwrap();
    assert!(matches!(stage, 1..=5 | 100..=227));
    assert!(!output.exists() && !output.with_extension("dag").exists());
    let codec = crate::artifact::Limits {
        max_bytes: 16 << 20,
        max_nodes: 196_608,
        max_depth: 4096,
    };
    let setup = Instant::now();
    let mut ar = Reduction::<{ 1 << 22 }>::try_new_boxed().unwrap();
    assert!(ar.limit_allocations(RESIDENT));
    let program =
        crate::artifact::decode(&mut ar, &read(&program_path, codec.max_bytes), codec).unwrap();
    let job = crate::artifact::decode(&mut ar, &read(&job_path, codec.max_bytes), codec).unwrap();
    let stage_word = atom(&mut ar, u64::from(stage));
    let input = ar.pair(job, stage_word).unwrap();
    let program_particle = particle(&ar, program);
    let job_particle = particle(&ar, job);
    let subject_particle = particle(&ar, input);
    let loaded = ar.count();
    let mut fields = ar.tail(program).unwrap();
    for _ in 0..3 {
        fields = ar.tail(fields).unwrap();
    }
    let formula = ar.head(fields).unwrap();
    let setup_micros = setup.elapsed().as_micros();
    let started = Instant::now();
    let mut callbacks = 0u64;
    let mut next_report = Duration::from_secs(60);
    let execution = reduce_compacting_cached_controlled(
        &mut ar,
        input,
        formula,
        BUDGET,
        CompactionLimits {
            max_frames: 65_536,
            max_total_allocations: TOTAL_ALLOCATIONS,
            max_collection_work: COLLECTION_WORK,
        },
        &mut || {
            callbacks += 1;
            let elapsed = started.elapsed();
            if elapsed >= next_report {
                std::eprintln!(
                    "prefix stage {stage}: {}s, {callbacks} total checkpoints",
                    elapsed.as_secs()
                );
                next_report += Duration::from_secs(60);
            }
            elapsed >= Duration::from_secs(SECONDS)
        },
    );
    let elapsed_micros = started.elapsed().as_micros();
    let (stats, peak_frames, status, raw_outcome, value, remaining, halt_budget) = match execution {
        Ok(run) => {
            let raw = std::format!("{:?}", run.outcome);
            let (status, value, remaining, halt) = match run.outcome {
                Outcome::Ok(value, remaining) => ("returned", Some(value), Some(remaining), None),
                Outcome::Halt(budget) => ("budget_exhausted", None, None, Some(budget)),
                Outcome::Error(_) => ("execution_error", None, None, None),
            };
            (
                run.stats,
                run.peak_frames,
                status,
                raw,
                value,
                remaining,
                halt,
            )
        }
        Err(failure) => (
            failure.stats,
            failure.peak_frames,
            "host_limit",
            std::format!("{:?}", failure.kind),
            None,
            None,
            None,
        ),
    };
    let words = value.and_then(|r| five_words(&ar, r));
    let mut encode_error = None;
    if let Some(value) = value {
        match crate::artifact::encode(&ar, value, codec) {
            Ok(bytes) => write_new(&output.with_extension("dag"), &bytes).unwrap(),
            Err(error) => encode_error = Some(std::format!("{error:?}")),
        }
    }
    let args: Vec<_> = std::env::args_os()
        .map(|v| quoted(&v.to_string_lossy()))
        .collect();
    let receipt = std::format!(
        "{{\n  \"schema\": \"nox/prefix-frontier/v1\",\n  \"scope\": \"pure VM prefix component diagnostic; no Joy admission or C2 claim\",\n{}\n  \"artifact\": {},\n  \"input\": {},\n  \"program_particle\": {},\n  \"job_particle\": {},\n  \"executed_subject_particle\": {},\n  \"command\": [{}],\n  \"stage\": {stage},\n  \"budget\": {BUDGET},\n  \"allocation_limit\": {TOTAL_ALLOCATIONS},\n  \"resident_limit\": {RESIDENT},\n  \"physical_nodes\": 4194304,\n  \"collection_work_limit\": {COLLECTION_WORK},\n  \"max_frames\": 65536,\n  \"max_seconds\": {SECONDS},\n  \"status\": {},\n  \"outcome\": {},\n  \"remaining_budget\": {},\n  \"charged_reductions\": {},\n  \"propagated_halt_budget\": {},\n  \"peak_frames\": {peak_frames},\n  \"prefix_words\": {},\n  \"encode_error\": {},\n  \"setup_micros\": {setup_micros},\n  \"elapsed_micros\": {elapsed_micros},\n  \"pinned_nodes\": {},\n  \"resident_nodes\": {},\n  \"peak_resident_nodes\": {},\n  \"total_allocations\": {},\n  \"reclaimed_nodes\": {},\n  \"collections\": {},\n  \"collection_work\": {},\n  \"scratch_bytes\": {},\n  \"evaluator_checkpoints\": {},\n  \"collection_checkpoints\": {},\n  \"total_checkpoints\": {callbacks}\n}}\n",
        measurement_metadata(),
        quoted(&program_path.to_string_lossy()),
        quoted(&job_path.to_string_lossy()),
        quoted(&program_particle),
        quoted(&job_particle),
        quoted(&subject_particle),
        args.join(", "),
        quoted(status),
        quoted(&raw_outcome),
        scalar(remaining),
        scalar(remaining.map(|r| BUDGET - r)),
        scalar(halt_budget),
        words
            .map(|v| std::format!("{v:?}"))
            .unwrap_or_else(|| String::from("null")),
        encode_error
            .map(|v| quoted(&v))
            .unwrap_or_else(|| String::from("null")),
        stats.pinned_nodes,
        stats.resident_nodes,
        stats.peak_resident_nodes,
        stats.total_allocations,
        stats.reclaimed_nodes,
        stats.collections,
        stats.collection_work,
        stats.scratch_bytes,
        stats.evaluator_checkpoints,
        stats.collection_checkpoints,
    );
    write_new(&output, receipt.as_bytes()).unwrap();
    assert_eq!(stats.pinned_nodes, loaded);
    assert_eq!(
        callbacks,
        stats.evaluator_checkpoints + stats.collection_checkpoints
    );
    assert!(stats.peak_resident_nodes <= RESIDENT && stats.total_allocations <= TOTAL_ALLOCATIONS);
    assert!(stats.collection_work <= COLLECTION_WORK);
    std::eprintln!(
        "prefix stage {stage}: {status}, receipt {}",
        output.display()
    );
}
