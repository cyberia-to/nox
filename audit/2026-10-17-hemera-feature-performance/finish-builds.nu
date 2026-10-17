use control.nu *
# Continue after the successful A build. Cargo -vv mixes build-script lines
# with JSON messages; retain those raw lines and parse only JSON objects.
def main [repo: path, cache: path] {
    let repo = ($repo | path expand)
    let cache = ($cache | path expand)
    let out = ($repo | path join audit/2026-10-17-hemera-feature-performance)
    require (not ($out | path join builds.json | path exists)) 'one fixed build epoch'
    let sources = (open ($out | path join sources.json))
    let registry = (open ($out | path join registry.json))
    let tools = (open ($out | path join tools.json))
    require ((open ($out | path join build-A.json)).exit_code == 0) 'existing A build succeeded'
    check-sources $sources
    check-registry $registry
    mut builds = []
    for cell in [A B C D P] {
        let run_env = (environment $cache $cell)
        let cwd = ($cache | path join cells $cell nox)
        let command = (bench-command $cache $cell true)
        let receipt_path = ($out | path join $'build-($cell).json')
        let receipt = (if ($receipt_path | path exists) { open $receipt_path } else {
            require (not ($cache | path join targets $cell | path exists)) 'new cell target must be absent'
            print $'building ($cell) once without measuring'
            capture $command $run_env $cwd $out $'build-($cell)'
        })
        require ($receipt.command == $command and $receipt.environment == $run_env and $receipt.cwd == $cwd) 'exact build command/env'
        require ($receipt.exit_code == 0) $'build prerequisite red: ($cell)'
        for stream in [stdout stderr] {
            require ((sha ($out | path join $'build-($cell).($stream)')) == ($receipt | get $'($stream)_sha256')) 'raw build stream'
        }
        let messages = (open --raw ($out | path join $'build-($cell).stdout') | lines
            | where {|l| $l | str starts-with '{'} | each {|l| $l | from json})
        let artifacts = ($messages | where {|r| $r.reason == 'compiler-artifact' and $r.target.name == reduction and $r.executable? != null})
        require (($artifacts | length) == 1 and not $artifacts.0.fresh) 'one newly compiled reduction executable'
        let binary = $artifacts.0.executable
        let tree_command = (cargo-command $cache $cell [tree --manifest-path ($cwd | path join Cargo.toml)
            -p cyber-nox --locked --offline -e features -i cyber-hemera])
        let tree_path = ($out | path join $'features-($cell).json')
        let tree = (if ($tree_path | path exists) { open $tree_path } else { capture $tree_command $run_env $cwd $out $'features-($cell)' })
        require ($tree.exit_code == 0 and $tree.command == $tree_command) 'feature tree prerequisite'
        let metadata_command = (cargo-command $cache $cell [metadata --manifest-path ($cwd | path join Cargo.toml)
            --format-version '1' --locked --offline])
        let metadata_path = ($out | path join $'metadata-($cell).json')
        let metadata = (if ($metadata_path | path exists) { open $metadata_path } else { capture $metadata_command $run_env $cwd $out $'metadata-($cell)' })
        require ($metadata.exit_code == 0 and $metadata.command == $metadata_command) 'metadata prerequisite'
        $builds = ($builds | append {cell: $cell, binary: $binary, sha256: (sha $binary),
            bytes: (open --raw $binary | into binary | bytes length), receipt: $receipt,
            features: $tree, metadata: $metadata})
    }
    check-sources $sources
    check-registry $registry
    for t in $tools.executable_files { require ((sha $t.path) == $t.sha256) 'tool drift during builds' }
    {builds: $builds, sources_equal: true, registry_equal: true, tools_equal: true,
        finished: (date now | format date '%+'), original_scripts: $tools.scripts,
        continuation_script_sha256: (sha ($out | path join finish-builds.nu))} | to json | save ($out | path join builds.json)
    print 'all five build-only cells frozen; no benchmark has executed'
}
