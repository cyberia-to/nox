use support.nu *

# Independently replay the preserved explicit artifacts after verify.nu extraction.
def main [repo: path, verified_cache: path] {
    let root = ($repo | path expand)
    let cache = ($verified_cache | path expand)
    let audit = ($root | path join audit/2026-10-17-published-api-baseline)
    let out = ($audit | path join root-replay)
    require (not ($out | path exists)) 'root replay already exists'
    mkdir $out
    let frozen = (open ($audit | path join tooling-final.json))
    require (($frozen.files | to json --raw | hash sha256) == $frozen.sha256) 'tool manifest'
    for file in $frozen.files {
        require ((digest ($audit | path join $file.path)) == $file.sha256) 'tool source changed'
    }
    let environment = {RUSTUP_TOOLCHAIN: '1.95.0', RUSTFLAGS: '', RUSTDOCFLAGS: ''}
    for feature in [default std] {
        capture $out $feature [cargo-semver-checks check-release
            --current-rustdoc ($cache | path join $'current-($feature).json')
            --baseline-rustdoc ($cache | path join $'baseline-($feature).json')
            --release-type patch --color never] $root $environment | ignore
        require (open --raw ($out | path join $'($feature).stderr')
            | str contains '229 checks: 229 pass, 31 skip') 'comparison summary'
    }
    {tooling_sha256: $frozen.sha256, runner_sha256: (digest ($audit | path join root-replay.nu)),
        source_sha256: ((committed-owner $root) | to json --raw | hash sha256),
        compared_capsule_sha256: (digest ($audit | path join evidence.tar.gz)),
        default: '229 pass / 31 skip', std: '229 pass / 31 skip'}
    | to json | save ($out | path join result.json)
}
