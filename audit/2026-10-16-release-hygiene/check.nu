# Exact local Rust/manifest closure; dependency roots resolve through the recorded wrapper.
def sources [repo: string] {
    let wrapper = ($repo | path dirname)
    [nox hemera strata lens] | each {|name|
        let p = (if $name == nox { $repo } else { $wrapper | path join $name | path expand })
        let paths = (^git -C $p ls-files | lines | where {|f|
            (not ($f | str starts-with audit/) and not ($f | str starts-with .claude/) and
            (($f | str ends-with '.rs') or ($f | path basename) in [Cargo.toml Cargo.lock rust-toolchain.toml]))
        } | sort)
        {repo: $name, root: $p, head: (^git -C $p rev-parse HEAD | str trim),
            files: ($paths | each {|f| {path: $f, sha256: (open --raw ($p | path join $f) | into binary | hash sha256)}})}
    }
}
def main [repo: string, name: string, argv_json: string, --manifest: string = 'sources.json'] {
    let dir = ($repo | path expand)
    cd $dir
    let out = 'audit/2026-10-16-release-hygiene'
    let before = (sources $dir)
    let hash = ($before | to json --raw | hash sha256)
    let manifest_path = ($out | path join $manifest)
    if ($manifest_path | path exists) {
        if (open $manifest_path).source_sha256 != $hash { error make {msg: 'frozen source changed'} }
    } else {
        {source_sha256: $hash, inputs: $before} | to json | save $manifest_path
    }
    let argv = ($argv_json | from json)
    let started = (date now | format date '%+')
    let r = (do { ^($argv | first) ...($argv | skip 1) } | complete)
    if $before != (sources $dir) { error make {msg: 'source changed during gate'} }
    $r.stdout | save ($out | path join $'($name).stdout')
    $r.stderr | save ($out | path join $'($name).stderr')
    {head: (^git rev-parse HEAD | str trim), source_sha256: $hash, source_before_after: equal,
        command: $argv, cwd: $dir, started: $started, finished: (date now | format date '%+'),
        exit_code: $r.exit_code, cargo_target: $env.CARGO_TARGET_DIR,
        rustc: (^rustc --version | str trim), cargo: (^cargo --version | str trim),
        rustc_path: $env.RUSTC, rustdoc_path: $env.RUSTDOC,
        rustflags: $env.RUSTFLAGS, rustdocflags: $env.RUSTDOCFLAGS, jobs: $env.CARGO_BUILD_JOBS,
        stdout_sha256: ($r.stdout | hash sha256), stderr_sha256: ($r.stderr | hash sha256)
    } | to json | save ($out | path join $'($name).json')
    print {name: $name, exit_code: $r.exit_code}
    if $r.exit_code != 0 { exit $r.exit_code }
}
