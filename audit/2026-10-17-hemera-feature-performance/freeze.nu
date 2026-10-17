use control.nu *
def main [repo: path, cache: path] {
    let repo = ($repo | path expand)
    let cache = ($cache | path expand)
    let out = ($repo | path join audit/2026-10-17-hemera-feature-performance)
    require (not ($out | path join freeze.json | path exists)) 'retain original pre-timing freeze'
    require (not ($out | path join timing-started.json | path exists)) 'timing remains unstarted'
    for cell in [A B C D P] { require (not ($cache | path join targets $cell criterion | path exists)) 'no Criterion timing state' }
    let scripts = ([verify.nu portable.nu control.nu timing.nu freeze.nu] | each {|p| {path: $p, sha256: (sha ($out | path join $p))}})
    {scripts: $scripts, source_base: '64d091e5ad3f277cce77b88ae0de6eea4c49b4df',
        branch_head: (^git -C $repo rev-parse HEAD | str trim), started: (date now | format date '%+')}
        | to json | save ($out | path join preflight-final-inputs.json)
    let run_env = (environment $cache A)
    let command = [(which nu | first | get path) ($out | path join verify.nu) $repo --live]
    let receipt = (capture $command $run_env $repo $out preflight-final)
    require ($receipt.exit_code == 0) 'final portable/live preflight'
    for s in $scripts { require ((sha ($out | path join $s.path)) == $s.sha256) 'verifier source changed during final replay' }
    cp ($out | path join README.md) ($out | path join preflight.md)
    cp ($out | path join preparation.md) ($out | path join preparation-at-freeze.md)
    cp ($repo | path join .claude/plans/hemera-feature-performance.md) ($out | path join plan-at-freeze.md)
    cp ($repo | path dirname | path join task.json) ($out | path join task-at-freeze.json)
    let files = (glob --no-dir ($out | path join '**/*')
        | where {|p| ($p | path relative-to $out) not-in [README.md preparation.md]}
        | each {|p| {path: ($p | path relative-to $out), sha256: (sha $p), bytes: (open --raw $p | into binary | bytes length)}} | sort-by path)
    {created: (date now | format date '%+'), source_base: '64d091e5ad3f277cce77b88ae0de6eea4c49b4df',
        branch_head: (^git -C $repo rev-parse HEAD | str trim), files: $files,
        files_sha256: ($files | to json --raw | hash sha256), schedule: [A1 B1 B2 A2 C1 D1 D2 C2 D3 C3 C4 D4 B3 A3 A4 B4],
        pause_seconds: 60, measured_runs_so_far: 0,
        mutable_reports: [README.md preparation.md],
        timing_approval: 'requires independent root freeze review plus a separately approved quiet window',
        per_era_feature_rows: 36, total_feature_rows: 72, temporal_rows: 72,
        threshold: 'candidate median / parent median > 1.05 OR candidate median 95% lower bound > parent median 95% upper bound'}
        | to json | save ($out | path join freeze.json)
    print {freeze_sha256: (sha ($out | path join freeze.json)), final_preflight: pass, measured_runs: 0}
}
