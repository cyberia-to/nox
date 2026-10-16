def verify-file [path: string, expected: string] {
    if (open --raw $path | hash sha256) != $expected {
        error make {msg: $'bytes changed: ($path)'}
    }
}
def main [repo: string] {
    cd ($repo | path expand)
    let audit = 'audit/2026-10-16-honeycrisp-backend-design'
    for name in [sources.json sources-package.json] {
        let manifest = (open ($audit | path join $name))
        if ($manifest.inputs | to json --raw | hash sha256) != $manifest.source_sha256 {
            error make {msg: 'manifest digest mismatch'}
        }
        for repo in $manifest.inputs.repositories {
            let matches = if $repo.name == nox {
                (^git -C $repo.root merge-base --is-ancestor $repo.head HEAD | complete).exit_code == 0
            } else { (^git -C $repo.root rev-parse HEAD | str trim) == $repo.head }
            if not $matches {
                error make {msg: $'revision changed: ($repo.name)'}
            }
            for file in $repo.files {
                verify-file ($repo.root | path join $file.path) $file.sha256
            }
        }
        for file in $manifest.inputs.diagnostic {
            verify-file $file.path $file.sha256
        }
        if 'package' in ($manifest.inputs | columns) {
            for file in $manifest.inputs.package {
                verify-file $file.path $file.sha256
            }
        }
    }
    for name in [registry-feature source-debug source-release acpu-package package-consumer] {
        let receipt = (open ($audit | path join $'($name).json'))
        let expected_code = if $name == registry-feature { 101 } else { 0 }
        let manifest_name = if $name == package-consumer { 'sources-package.json' } else { 'sources.json' }
        let manifest = (open ($audit | path join $manifest_name))
        if $receipt.exit_code != $expected_code or $receipt.source_before_after != equal or $receipt.source_sha256 != $manifest.source_sha256 {
            error make {msg: $'unexpected result or source identity: ($name)'}
        }
        for stream in [stdout stderr] {
            verify-file ($audit | path join $'($name).($stream)') ($receipt | get $'($stream)_sha256')
        }
    }
    print 'source identities and all five receipt streams verified'
}
