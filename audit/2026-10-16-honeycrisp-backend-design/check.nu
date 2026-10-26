def inputs [repo: string, honey: string, diagnostic: string, package: string] {
    let wrapper = ($repo | path dirname)
    let roots = [
        {name: nox, root: $repo, scopes: []}
        {name: honeycrisp, root: $honey, scopes: [Cargo.toml Cargo.lock acpu unimem]}
        {name: hemera, root: ($wrapper | path join hemera | path expand), scopes: []}
        {name: strata, root: ($wrapper | path join strata | path expand), scopes: []}
        {name: lens, root: ($wrapper | path join lens | path expand), scopes: []}
    ]
    let repositories = ($roots | each {|r|
        let files = (^git -C $r.root ls-files ...$r.scopes | lines | where {|p|
            (not ($p | str starts-with audit/) and not ($p | str starts-with .claude/) and
            (($p | str ends-with '.rs') or ($p | path basename) in [Cargo.toml Cargo.lock rust-toolchain.toml]))
        } | sort | each {|p| {path: $p, sha256: (open --raw ($r.root | path join $p) | hash sha256)}})
        {name: $r.name, root: $r.root, head: (^git -C $r.root rev-parse HEAD | str trim), files: $files}
    })
    let base = {repositories: $repositories, diagnostic: ([
        ($diagnostic | path join Cargo.toml)
        ($diagnostic | path join Cargo.lock)
        ($repo | path join audit/2026-10-16-honeycrisp-backend-design/probe.rs)
    ] | each {|p| {path: $p, sha256: (open --raw $p | hash sha256)}})}
    if $package == '' { $base } else {
        $base | insert package (^rg --files $package | lines | where {|p|
            ($p | str ends-with '.rs') or ($p | path basename) in [Cargo.toml Cargo.lock]
        } | sort | each {|p| {path: $p, sha256: (open --raw $p | hash sha256)}})
    }
}

def main [repo: string, honey: string, diagnostic: string, name: string, cwd: string, argv_json: string,
    --manifest: string = 'sources.json', --package: string = ''] {
    let root = ($repo | path expand)
    let honey = ($honey | path expand)
    let diagnostic = ($diagnostic | path expand)
    let out = ($root | path join audit/2026-10-16-honeycrisp-backend-design)
    let before = (inputs $root $honey $diagnostic $package)
    let hash = ($before | to json --raw | hash sha256)
    let path = ($out | path join $manifest)
    if ($path | path exists) {
        if (open $path).source_sha256 != $hash { error make {msg: 'diagnostic input epoch changed'} }
    } else { {source_sha256: $hash, inputs: $before} | to json | save $path }
    let argv = ($argv_json | from json)
    let started = (date now | format date '%+')
    cd $cwd
    let r = (do { ^($argv | first) ...($argv | skip 1) } | complete)
    if $before != (inputs $root $honey $diagnostic $package) { error make {msg: 'source changed during probe'} }
    $r.stdout | save ($out | path join $'($name).stdout')
    $r.stderr | save ($out | path join $'($name).stderr')
    {command: $argv, cwd: ($cwd | path expand), source_sha256: $hash, source_before_after: equal,
        started: $started, finished: (date now | format date '%+'), exit_code: $r.exit_code,
        rustc: (^rustc --version | str trim), cargo: (^cargo --version | str trim),
        rustc_path: $env.RUSTC, rustdoc_path: $env.RUSTDOC, target: $env.CARGO_TARGET_DIR,
        rustflags: $env.RUSTFLAGS, rustdocflags: $env.RUSTDOCFLAGS, jobs: $env.CARGO_BUILD_JOBS,
        stdout_sha256: ($r.stdout | hash sha256), stderr_sha256: ($r.stderr | hash sha256)
    } | to json | save ($out | path join $'($name).json')
    print {name: $name, exit_code: $r.exit_code}
    if $r.exit_code != 0 { exit $r.exit_code }
}
