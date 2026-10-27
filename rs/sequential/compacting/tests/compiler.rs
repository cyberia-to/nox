use super::*;
use std::{
    io::Read,
    path::PathBuf,
    time::{Duration, Instant},
};

fn read(path: &PathBuf, max: usize) -> Vec<u8> {
    let file = std::fs::File::open(path).unwrap();
    let metadata = file.metadata().unwrap();
    assert!(metadata.is_file() && metadata.len() <= max as u64);
    let mut bytes = Vec::new();
    file.take(max as u64 + 1).read_to_end(&mut bytes).unwrap();
    assert!(bytes.len() <= max);
    bytes
}

#[test]
#[ignore = "saved full compiler discovery diagnostic; explicit NOX_COMPACTION_* paths required"]
fn saved_full_discovery_preserves_result_gas_frames_and_guest_checkpoints() {
    std::thread::Builder::new().stack_size(256 << 20).spawn(|| {
        let program = PathBuf::from(std::env::var_os("NOX_COMPACTION_PROGRAM").unwrap());
        let job = PathBuf::from(std::env::var_os("NOX_COMPACTION_JOB").unwrap());
        let output = PathBuf::from(std::env::var_os("NOX_COMPACTION_OUTPUT").unwrap());
        assert!(!output.exists() && !output.with_extension("dag").exists());
        let codec = crate::artifact::Limits { max_bytes: 16 << 20, max_nodes: 196_608, max_depth: 4096 };
        let mut ar = Reduction::<{ 1 << 22 }>::try_new_boxed().unwrap();
        assert!(ar.limit_allocations(3_145_728));
        let program = crate::artifact::decode(&mut ar, &read(&program, codec.max_bytes), codec).unwrap();
        let job = crate::artifact::decode(&mut ar, &read(&job, codec.max_bytes), codec).unwrap();
        let stage = atom(&mut ar, 2);
        let input = ar.pair(job, stage).unwrap();
        let mut fields = ar.tail(program).unwrap();
        for _ in 0..3 { fields = ar.tail(fields).unwrap(); }
        let formula = ar.head(fields).unwrap();
        assert_eq!(ar.count(), 159_037);
        let started = Instant::now();
        let mut callbacks = 0u64;
        let mut next_report = Duration::from_secs(60);
        let result = reduce_compacting_cached_controlled(&mut ar, input, formula, 1_000_000_000,
            CompactionLimits { max_frames: 65_536, max_total_allocations: 64_000_000, max_collection_work: 2_000_000_000 },
            &mut || {
                callbacks += 1;
                let elapsed = started.elapsed();
                if elapsed >= next_report {
                    std::eprintln!("compacting discovery: {}s, {callbacks} total checkpoints", elapsed.as_secs());
                    next_report += Duration::from_secs(60);
                }
                elapsed >= Duration::from_secs(1200)
            });
        let elapsed_micros = started.elapsed().as_micros();
        let result = match result {
            Ok(result) => result,
            Err(failure) => {
                std::fs::write(output, std::format!("{failure:#?}\n")).unwrap();
                panic!("compacting discovery failed: {failure:?}");
            }
        };
        let stats = result.stats;
        let Outcome::Ok(value, remaining) = result.outcome else { panic!("{:?}", result.outcome); };
        let bytes = crate::artifact::encode(&ar, value, codec).unwrap();
        std::fs::write(output.with_extension("dag"), bytes).unwrap();
        let mut words = Vec::new();
        let mut rest = value;
        for _ in 0..4 {
            words.push(ar.atom_value(ar.head(rest).unwrap()).unwrap().as_u64());
            rest = ar.tail(rest).unwrap();
        }
        words.push(ar.atom_value(rest).unwrap().as_u64());
        std::fs::write(&output, std::format!(
            "{{\n  \"stage\": 2,\n  \"remaining_budget\": {remaining},\n  \"charged_reductions\": {},\n  \"peak_frames\": {},\n  \"prefix_words\": {words:?},\n  \"elapsed_micros\": {elapsed_micros},\n  \"pinned_nodes\": {},\n  \"resident_nodes\": {},\n  \"peak_resident_nodes\": {},\n  \"total_allocations\": {},\n  \"reclaimed_nodes\": {},\n  \"collections\": {},\n  \"collection_work\": {},\n  \"scratch_bytes\": {},\n  \"evaluator_checkpoints\": {},\n  \"collection_checkpoints\": {},\n  \"total_checkpoints\": {callbacks}\n}}\n",
            1_000_000_000 - remaining, result.peak_frames, stats.pinned_nodes, stats.resident_nodes,
            stats.peak_resident_nodes, stats.total_allocations, stats.reclaimed_nodes, stats.collections,
            stats.collection_work, stats.scratch_bytes, stats.evaluator_checkpoints, stats.collection_checkpoints,
        )).unwrap();
        assert_eq!(remaining, 315_353_719);
        assert_eq!(result.peak_frames, 3259);
        assert_eq!(stats.evaluator_checkpoints, 566_412_397);
        assert_eq!(callbacks, stats.evaluator_checkpoints + stats.collection_checkpoints);
        assert_eq!(words, [2, 94, 0, 0, 0]);
        assert!(stats.collections > 0);
        assert!(stats.total_allocations > 3_145_728);
        assert!(stats.peak_resident_nodes <= 3_145_728);
    }).unwrap().join().unwrap();
}
