use control.nu *
def compiler-file [p: string] {
    ($p | str ends-with '.rs') or ($p | path basename) in [Cargo.toml Cargo.lock rust-toolchain.toml config config.toml]
}
def main [repo: path] {
    let repo = ($repo | path expand)
    let out = ($repo | path join audit/2026-10-17-hemera-feature-performance)
    require (not ($out | path join preflight-summary.json | path exists)) 'retain preflight summary epoch'
    let source = (open ($out | path join sources.json))
    let builds = (open ($out | path join builds.json))
    let code = (open ($out | path join code.json))
    let bridge = ([nox hemera strata lens] | each {|name|
        let a = ($source.cells | where cell == B and repo == $name | first)
        let b = ($source.cells | where cell == D and repo == $name | first)
        let paths = ($a.entries.path | append $b.entries.path | uniq | sort | where {|p| compiler-file $p})
        {repo: $name, historical_revision: $a.revision, current_revision: $b.revision,
            changed: ($paths | each {|p|
                let before = ($a.entries | where path == $p)
                let after = ($b.entries | where path == $p)
                if $before != $after { {path: $p, historical: $before, current: $after} }
            })}
    })
    let cfgs = ($builds.builds | each {|b|
        let commands = (open --raw ($out | path join $'build-($b.cell).stderr') | lines
            | where {|l| ($l | str contains 'Running ') and ($l | str contains '--crate-name ')})
        $commands | str join "\n" | save ($out | path join $'rustc-commands-($b.cell).txt')
        let m = (open --raw ($out | path join $'metadata-($b.cell).stdout') | from json)
        {cell: $b.cell, resolved_features: ($m.resolve.nodes | select id features),
            own_rustc_commands: ($commands | where {|l| $l =~ '--crate-name (cyber_hemera|nebu|nox|reduction) '})}
    })
    let a = ($source.cells | where cell == B and repo == nox | first | get root | path join rs/benches/reduction.rs)
    let b = ($source.cells | where cell == D and repo == nox | first | get root | path join rs/benches/reduction.rs)
    let diff = (do { ^git diff --no-index --no-ext-diff -- $a $b } | complete)
    require ($diff.exit_code == 1 and $diff.stderr == '') 'retained benchmark formatting diff'
    $diff.stdout | save ($out | path join benchmark-era.patch)
    let comparisons = ($code.comparisons | each {|r|
        {pair: $'($r.parent)/($r.candidate)',
            changed_sections: ($r.sections | where raw_bytes_equal == false | select segment section),
            relevant_spans: ($r.relevant_functions | length),
            unique_name_matches: ($r.relevant_functions | where {|f| ($f.candidate_matches | length) == 1} | length),
            raw_equal_spans: ($r.relevant_functions | where {|f| ($f.candidate_matches | any {|m| $m.bytes_equal})} | length),
            moved_spans: ($r.relevant_functions | where {|f| ($f.candidate_matches | any {|m| $m.address != $f.parent_address})} | length)}
    })
    {command: [nu audit/2026-10-17-hemera-feature-performance/summarize-preflight.nu $repo],
        source_base: $source.base, script_sha256: (sha ($out | path join summarize-preflight.nu)),
        source_era_bridge: $bridge, cfgs: $cfgs, binaries: ($builds.builds | select cell sha256 bytes),
        code_comparisons: $comparisons, benchmark_format_exact: (open ($out | path join benchmark-format.json)).exact_current_bytes,
        interpretation: 'The identical-source A/P path control changes raw text/layout, so A/B or C/D byte differences do not alone isolate feature effects; B/D spans multiple source changes.'}
        | to json | save ($out | path join preflight-summary.json)
    print ($comparisons | to json)
}
