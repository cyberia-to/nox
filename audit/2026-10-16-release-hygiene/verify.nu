def main [repo: string] {
    let root = ($repo | path expand)
    cd $root
    let out = 'audit/2026-10-16-release-hygiene'
    let manifest = (open ($out | path join sources-reviewed.json))
    if ($manifest.inputs | to json --raw | hash sha256) != $manifest.source_sha256 {
        error make {msg: 'source manifest digest mismatch'}
    }
    for input in $manifest.inputs {
        let dir = (if $input.repo == nox { $root } else { $input.root })
        if $input.repo == nox {
            # The receipt names the pre-commit base; exact file hashes below
            # identify the candidate after commit/merge as well as before it.
            let ancestry = (do { ^git -C $dir merge-base --is-ancestor $input.head HEAD } | complete)
            if $ancestry.exit_code != 0 {
                error make {msg: 'owning source no longer descends from recorded base'}
            }
        } else if (^git -C $dir rev-parse HEAD | str trim) != $input.head {
            error make {msg: $'revision changed: ($input.repo)'}
        }
        for file in $input.files {
            if (open --raw ($dir | path join $file.path) | hash sha256) != $file.sha256 {
                error make {msg: $'source bytes changed: ($input.repo)/($file.path)'}
            }
        }
    }
    let receipts = (glob ($out | path join '**/*.json') | where {|p|
        let value = (open $p)
        ($value | describe | str starts-with record) and ('exit_code' in ($value | columns))
    })
    for path in $receipts {
        let receipt = (open $path)
        let stem = ($path | str replace --regex '\.json$' '')
        for stream in [stdout stderr] {
            let stream_path = $'($stem).($stream)'
            let actual = (if ($stream_path | path exists) {
                open --raw $stream_path | hash sha256
            } else if ($'($stream_path).gz' | path exists) {
                ^gzip -dc $'($stream_path).gz' | hash sha256
            } else { '' | hash sha256 })
            if $actual != ($receipt | get $'($stream)_sha256') {
                error make {msg: $'stream mismatch: ($stream_path)'}
            }
        }
    }
    let bridge = (open ($out | path join format-bridge-reviewed.json))
    if (open --raw ($out | path join nonformat-reviewed.patch) | hash sha256) != $bridge.nonformat_patch_sha256 {
        error make {msg: 'non-format patch digest mismatch'}
    }
    print {source_sha256: $manifest.source_sha256, checked_receipts: ($receipts | length), result: pass}
}
