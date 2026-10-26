use control.nu *
use portable.nu load-portable
def stream-sha [out: path, stem: string, stream: string] {
    let p = ($out | path join $'($stem).($stream)')
    if ($p | path exists) { sha $p } else { ^gzip -dc $'($p).gz' | hash sha256 }
}
def row [s: record, cell: string, repo: string] { $s.cells | where cell == $cell and repo == $repo | first }
def median [out: path, run: string, case: string] {
    open ($out | path join runs $run criterion $case new estimates.json) | get median
}
def compare [out: path, first: string, second: string, case: string] {
    let a = (median $out $first $case)
    let b = (median $out $second $case)
    let ratio = $b.point_estimate / $a.point_estimate
    let disjoint = $b.confidence_interval.lower_bound > $a.confidence_interval.upper_bound
    {case: $case, parent: $first, candidate: $second, parent_median: $a, candidate_median: $b,
        ratio: $ratio, change_percent: (($ratio - 1) * 100), lower_exceeds_upper: $disjoint,
        threshold_red: ($ratio > 1.05 or $disjoint)}
}
def verify [repo: path, evidence: record, timed: bool, live: bool] {
    let repo = ($repo | path expand)
    let out = ($repo | path join audit/2026-10-17-hemera-feature-performance)
    let s = $evidence.sources
    let registry = $evidence.registry
    let tools = (open ($out | path join tools.json))
    let builds = (open ($out | path join builds.json))
    let code = (open ($out | path join code.json))
    check-sources $s
    check-registry $registry
    require (($s.cells | length) == 20) 'four source owners in five exports'
    for name in [nox hemera strata lens] {
        require ((row $s A $name).entries == (row $s P $name).entries) 'identical-source path control'
        for pair in [[A B] [C D]] {
            let a = (row $s $pair.0 $name)
            let b = (row $s $pair.1 $name)
            require ($a.entries.path == $b.entries.path) 'matched complete path set'
            let changed = ($a.entries | zip $b.entries | where {|p| $p.0 != $p.1} | each {|p| $p.0.path})
            let expected = (if $name != nox { [] } else if $pair.0 == A { [CLAUDE.md rs/Cargo.toml] } else { [rs/Cargo.toml] })
            require ($changed == $expected) 'exact treatment source boundary'
            if $name == nox {
                let inherited = (open ($a.root | path join rs/Cargo.toml))
                let shipped = (open ($b.root | path join rs/Cargo.toml))
                require (($inherited | update features.std ['hemera/std'] | upsert dependencies.hemera.default-features false) == $shipped) 'only two manifest feature edits'
            }
        }
    }
    let old = (open ($out | path join original-replay.json))
    let old_result = ($old.result.stdout | from json)
    require ($old.result.exit_code == 0 and $old_result.logical_payloads == 921 and $old_result.records == 103
        and $old_result.initial_threshold_cases == 5 and $old_result.repeat_threshold_cases == 6) 'original evidence reds retained'
    require ((open ($out | path join benchmark-format.json)).exact_current_bytes) 'benchmark formatter bridge'
    if $live {
        for t in ($tools.executable_files | append $code.helpers) { require ((sha $t.path) == $t.sha256) 'live frozen executable identity' }
        check-sources (open ($out | path join sources.json))
        check-registry (open ($out | path join registry.json))
    }
    for h in $code.helpers { require ((sha ($evidence.payload | path join helpers ($h.path | path basename))) == $h.sha256) 'portable helper identity' }
    for b in $builds.builds {
        let binary = ($evidence.payload | path join binaries $b.cell reduction)
        require ((sha $binary) == $b.sha256) 'portable benchmark identity'
        if $live { require ((sha $b.binary) == $b.sha256) 'live frozen benchmark identity' }
        let inspected = ($code.inspections | where cell == $b.cell | first)
        let body = (open --raw $binary | into binary)
        for section in ($inspected.sections | where file_backed) {
            let part = ($evidence.payload | path join sections $b.cell $'($section.segment).($section.section).bin')
            require ((sha $part) == $section.sha256) 'portable raw section identity'
            require (($body | bytes at $section.file_offset..($section.file_offset + $section.bytes - 1) | hash sha256) == $section.sha256) 'section belongs to exact binary'
        }
        let text = ($inspected.sections | where segment == __TEXT and section == __text | first)
        let raw_text = (open --raw ($evidence.payload | path join sections $b.cell __TEXT.__text.bin) | into binary)
        for f in $inspected.functions {
            let offset = $f.address - $text.address
            require (($raw_text | bytes at $offset..($offset + $f.span_to_next_start - 1) | hash sha256) == $f.code_sha256) 'exact function-start span'
        }
        let normalized = ($out | path join $'disassembly-($b.cell).normalized.txt.gz')
        require ((^gzip -dc $normalized | hash sha256) == $inspected.normalized_disassembly_sha256) 'explicit normalized disassembly view'
        require ($b.receipt.exit_code == 0) 'build result'
        require ($b.receipt.environment.RUSTC == $tools.clean_environment.RUSTC
            and $b.receipt.environment.RUSTDOC == $tools.clean_environment.RUSTDOC) 'absolute compiler pins'
        let tree = (open --raw ($out | path join $'features-($b.cell).stdout'))
        require (($tree | str contains 'cyber-hemera feature "std"') == ($b.cell in [A C P])) 'feature contrast'
    }
    for p in (glob ($out | path join '*.json')) {
        let r = (open $p)
        if ('command' in ($r | columns)) and ('stdout_sha256' in ($r | columns)) {
            let stem = ($p | path basename | str replace --regex '\.json$' '')
            for stream in [stdout stderr] {
                require ((stream-sha $out $stem $stream) == ($r | get $'($stream)_sha256')) $'receipt stream ($stem)/($stream)'
            }
        }
    }
    if not $timed {
        require (not ($out | path join timing-started.json | path exists)) 'preflight precedes timing'
        print ({status: pass, scope: 'portable source/build/binary/code preflight; timing unmeasured', live_tools_checked: $live,
            source_exports: 5, registry_archives: ($registry.packages | length),
            unchanged_old_reds: [5 6], binaries: ($builds.builds | select cell sha256)} | to json)
        return
    }
    let freeze = (open ($out | path join freeze.json))
    for f in $freeze.files { require ((sha ($out | path join $f.path)) == $f.sha256) 'frozen artifact drift' }
    let timing = (open ($out | path join timing.json))
    require ($timing.runs.run == $freeze.schedule) 'exact finite run schedule'
    require ((open ($out | path join timing-started.json)).freeze_sha256 == (sha ($out | path join freeze.json))) 'timing binds exact preflight freeze'
    let idle = (open ($out | path join inter-block-idle.json))
    require ($idle.requested_seconds == 60 and (($idle.finished | into datetime) - ($idle.started | into datetime)) >= 60sec) 'fixed inter-block idle'
    require (($timing.runs.7.helper_receipt.finished | into datetime) <= ($idle.started | into datetime)
        and ($idle.finished | into datetime) <= ($timing.runs.8.helper_receipt.started | into datetime)) 'idle separates fixed blocks'
    let cases = [encode_particle_atom encode_tree_64 hash/pattern_hash hash/pattern_hash_chain_64
        jet_merkle_verify pattern_add pattern_inv pattern_look_null pattern_mul]
    for r in $timing.runs {
        let base = ($out | path join runs $r.run)
        require ((open ($base | path join receipt.json)) == $r) 'run receipt equals index'
        require ((open ($out | path join runs $'($r.run).json')) == $r.helper_receipt) 'helper receipt equals index'
        let b = ($builds.builds | where cell == $r.cell | first)
        let expected = ($b.receipt.command | drop 3 | append [-- --warm-up-time '1' --measurement-time '2' --sample-size '20' --noplot])
        require ($r.command == $expected and ($r.helper_receipt.command | skip 3) == $expected) 'unchanged original benchmark argv'
        require ($r.helper_receipt.environment == $b.receipt.environment and $r.helper_receipt.cwd == $b.receipt.cwd) 'unchanged build/timing environment'
        let process = (open ($base | path join process.json))
        require ($process.exit_code == 0 and $process.sampler_ok and $process.sample_interval_seconds == 5) 'fixed sampler and successful measured child'
        require ($r.binary_unchanged and not $r.compilation_observed and $r.helper_receipt.exit_code == 0) 'timed binary execution only'
        require (($r.files | to json --raw | hash sha256) == $r.files_sha256) 'run inventory identity'
        for f in $r.files { require ((sha ($base | path join $f.path)) == $f.sha256) 'run payload identity' }
        let actual_cases = (glob ($base | path join criterion '**/new/estimates.json')
            | each {|p| $p | path relative-to ($base | path join criterion) | str replace '/new/estimates.json' ''} | sort)
        require ($actual_cases == $cases and $r.criterion_cases.case == $cases) 'exact nine-case inventory'
        require ($r.criterion_cases | all {|c|
            ($c.estimates_modified | into datetime) >= ($r.helper_receipt.started | into datetime)
                and ($c.sample_modified | into datetime) >= ($r.helper_receipt.started | into datetime)
        }) 'fresh per-run Criterion estimates and samples'
        for case in $cases {
            let sample = (open ($base | path join criterion $case new sample.json))
            require (($sample.iters | length) == 20 and ($sample.times | length) == 20) 'all original samples'
            require ((median $out $r.run $case).confidence_interval.confidence_level == 0.95) 'median 95 percent interval'
        }
    }
    let features = ([[A B] [C D]] | each {|pair| 1..4 | each {|n|
        let a = $'($pair.0)($n)'
        let b = $'($pair.1)($n)'
        {parent: $a, candidate: $b, rows: ($cases | each {|c| compare $out $a $b $c})}
    }} | flatten)
    let controls = ([A B C D] | each {|cell| [[1 2] [3 4]] | each {|pair|
        let a = $'($cell)($pair.0)'
        let b = $'($cell)($pair.1)'
        {parent: $a, candidate: $b, rows: ($cases | each {|c| compare $out $a $b $c})}
    }} | flatten)
    print ({status: pass, scope: 'arithmetic and inventory replay; host usability requires independent telemetry review',
        features: $features, temporal_controls: $controls, feature_rows: ($features.rows | flatten | length),
        temporal_rows: ($controls.rows | flatten | length),
        current_crossings: ($features | where {|r| $r.parent | str starts-with C} | get rows | flatten | where threshold_red | length),
        temporal_crossings: ($controls | get rows | flatten | where threshold_red | length),
        old_performance_gate: 'red/unverified'} | to json)
}
def main [repo: path, --timed, --live] {
    let repo = ($repo | path expand)
    let evidence = (load-portable ($repo | path join audit/2026-10-17-hemera-feature-performance))
    try { verify $repo $evidence $timed $live } catch {|e|
        rm --recursive $evidence.root
        error make {msg: $e.msg}
    }
    rm --recursive $evidence.root
}
