use control.nu *
def main [repo: path, cache: path, authorization: path] {
    let repo = ($repo | path expand)
    let cache = ($cache | path expand)
    let out = ($repo | path join audit/2026-10-17-hemera-feature-performance)
    let freeze = (open ($out | path join freeze.json))
    let consent = (open ($authorization | path expand))
    require ($consent.root_approved and $consent.quiet_window_approved) 'root freeze and quiet-window approval required'
    require ($consent.freeze_sha256 == (sha ($out | path join freeze.json))) 'approval of exact freeze'
    require (not ($out | path join timing-started.json | path exists)) 'finite schedule already started; never replace runs'
    require (not ($out | path join runs | path exists)) 'run evidence must start absent'
    for f in $freeze.files { require ((sha ($out | path join $f.path)) == $f.sha256) 'frozen runner/evidence drift' }
    let sources = (open ($out | path join sources.json))
    let registry = (open ($out | path join registry.json))
    let tools = (open ($out | path join tools.json))
    let code = (open ($out | path join code.json))
    let builds = (open ($out | path join builds.json))
    check-sources $sources
    check-registry $registry
    for t in ($tools.executable_files | append $code.helpers) { require ((sha $t.path) == $t.sha256) 'tool drift before timing' }
    let schedule = [A1 B1 B2 A2 C1 D1 D2 C2 D3 C3 C4 D4 B3 A3 A4 B4]
    require ($freeze.schedule == $schedule and $freeze.pause_seconds == 60) 'fixed two-block schedule'
    let cases = [encode_particle_atom encode_tree_64 hash/pattern_hash hash/pattern_hash_chain_64
        jet_merkle_verify pattern_add pattern_inv pattern_look_null pattern_mul]
    for cell in [A B C D] {
        require (not ($cache | path join targets $cell criterion | path exists)) 'no pre-existing timed Criterion state'
    }
    {started: (date now | format date '%+'), freeze_sha256: (sha ($out | path join freeze.json)),
        authorization_sha256: (sha ($authorization | path expand)), schedule: $schedule,
        source_inputs_checked: true, registry_checked: true, tools_checked: true}
        | to json | save ($out | path join timing-started.json)
    mkdir ($out | path join runs)
    mut receipts = []
    for entry in ($schedule | enumerate) {
        if $entry.index == 8 {
            let began = (date now | format date '%+')
            sleep 60sec
            {started: $began, finished: (date now | format date '%+'), requested_seconds: 60}
                | to json | save ($out | path join inter-block-idle.json)
        }
        let cell = ($entry.item | str substring 0..0)
        let run_env = (environment $cache $cell)
        let cwd = ($cache | path join cells $cell nox)
        let b = ($builds.builds | where cell == $cell | first)
        require ((sha $b.binary) == $b.sha256) 'binary drift before run'
        let command = (bench-command $cache $cell false)
        let run_dir = ($out | path join runs $entry.item)
        let helper_command = [($cache | path join helpers measure) $run_dir --] | append $command
        print $'scheduled run ($entry.index + 1)/16: ($entry.item)'
        let receipt = (capture $helper_command $run_env $cwd ($out | path join runs) $entry.item)
        let criterion = ($run_env.CARGO_TARGET_DIR | path join criterion)
        let criterion_cases = (glob ($criterion | path join '**/new/estimates.json') | each {|p|
            {case: ($p | path relative-to $criterion | str replace '/new/estimates.json' ''),
                estimates_modified: (ls --long $p | get 0.modified | format date '%+'),
                sample_modified: (ls --long ($p | path dirname | path join sample.json) | get 0.modified | format date '%+'),
                benchmark: (open ($p | path dirname | path join benchmark.json))}
        } | sort-by case)
        if ($criterion | path exists) { cp --recursive $criterion ($run_dir | path join criterion) }
        let files = (glob --no-dir ($run_dir | path join '**/*') | each {|p|
            {path: ($p | path relative-to $run_dir), sha256: (sha $p), bytes: (open --raw $p | into binary | bytes length)}
        } | sort-by path)
        let raw_stderr = (open --raw ($run_dir | path join stderr))
        let rebuilt = ($raw_stderr | str contains 'Compiling ')
        let binary_equal = ((sha $b.binary) == $b.sha256)
        let row = {run: $entry.item, order: ($entry.index + 1), cell: $cell, command: $command,
            helper_receipt: $receipt, binary_sha256: $b.sha256, binary_unchanged: $binary_equal,
            compilation_observed: $rebuilt, criterion_cases: $criterion_cases,
            files: $files, files_sha256: ($files | to json --raw | hash sha256)}
        $row | to json | save ($run_dir | path join receipt.json)
        $receipts = ($receipts | append $row)
        require ($receipt.exit_code == 0 and $binary_equal and not $rebuilt) 'run is indeterminate; preserved without replacement'
        require ($criterion_cases.case == $cases) 'exact complete case set'
        require ($criterion_cases | all {|c|
            ($c.estimates_modified | into datetime) >= ($receipt.started | into datetime)
                and ($c.sample_modified | into datetime) >= ($receipt.started | into datetime)
        }) 'stale Criterion epoch'
        for case in $cases {
            let base = ($run_dir | path join criterion $case new)
            let estimates = (open ($base | path join estimates.json))
            let sample = (open ($base | path join sample.json))
            require (($sample.iters | length) == 20 and ($sample.times | length) == 20) 'original sample count'
            require ($estimates.median.confidence_interval.confidence_level == 0.95) 'original median interval'
        }
    }
    check-sources $sources
    check-registry $registry
    for t in ($tools.executable_files | append $code.helpers) { require ((sha $t.path) == $t.sha256) 'tool drift after timing' }
    {finished: (date now | format date '%+'), runs: $receipts, source_inputs_equal: true,
        registry_equal: true, tools_equal: true, fixed_schedule_complete: true}
        | to json | save ($out | path join timing.json)
    print 'finite sixteen-run schedule complete; all raw epochs retained'
}
