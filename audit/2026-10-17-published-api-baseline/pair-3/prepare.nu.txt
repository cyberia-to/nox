use support.nu *

# Creates a fresh, explicitly diagnostic package pair. Existing outputs refuse reuse.
def main [repo: string, cache: string, epoch: string] {
    let root = ($repo | path expand)
    let cache = ($cache | path expand)
    let out = ($root | path join audit/2026-10-17-published-api-baseline $epoch)
    require (not ($out | path exists) and not ($cache | path exists)) 'epoch/cache already exists'
    require (($epoch | str replace -r '^[a-z0-9-]+$' '') == '') 'invalid epoch name'
    mkdir $out $cache
    let source = (committed-owner $root)
    [support.nu prepare.nu] | each {|p| {path: $p, sha256: (digest ($root | path join audit/2026-10-17-published-api-baseline $p))}}
    | to json | save ($out | path join tooling.json)
    let head = (^git -C $root rev-parse HEAD | str trim)
    let prefix = (^rustup which --toolchain 1.95.0 rustc | str trim | path dirname)
    let environment = {
        PATH: ([$prefix ($env.HOME | path join .cargo/bin) /opt/homebrew/bin /usr/bin /bin] | str join (char esep)),
        RUSTUP_TOOLCHAIN: '1.95.0', RUSTC: ($prefix | path join rustc), RUSTDOC: ($prefix | path join rustdoc),
        CARGO_TARGET_DIR: ($cache | path join package-target), CARGO_BUILD_JOBS: '2', CARGO_INCREMENTAL: '0',
        RUSTFLAGS: '-Dwarnings', RUSTDOCFLAGS: '-Dwarnings'
    }
    let index = ($cache | path join registry-index.jsonl)
    capture $out acquisition-index [curl --fail --location --silent --show-error --output $index 'https://index.crates.io/cy/be/cyber-nox'] $root {} | ignore
    let published = (open --raw $index | lines | each {|line| $line | from json} | where vers == '0.2.0' | first)
    require ($published.cksum == '7c807c1b53e66cea79e8e130ca938450029af3f096984a2839fee6443a171b3f' and not $published.yanked) 'unexpected published identity'
    let archive = ($cache | path join published.crate)
    capture $out acquisition-archive [curl --fail --location --silent --show-error --output $archive 'https://static.crates.io/crates/cyber-nox/cyber-nox-0.2.0.crate'] $root {} | ignore
    require ((digest $archive) == $published.cksum) 'published archive checksum mismatch'
    let baseline = (extract-crate $archive ($cache | path join baseline))
    let tag = (^git -C $root rev-parse 'v0.2.0^{}' | str trim)
    require ($tag == '6ab6e1a303355a49aaef6e069bffe5bfb3efa2a2') 'immutable tag target changed'
    let baseline_vcs = (open ($baseline | path join .cargo_vcs_info.json))
    require ($baseline_vcs.git.sha1 == $tag and $baseline_vcs.path_in_vcs == rs) 'archive VCS does not match peeled tag'
    let baseline_rust = (check-rust-against-git $root $tag $baseline)
    capture $out package-current [cargo package -p cyber-nox --locked --offline --allow-dirty] $root $environment | ignore
    let current_archive = ($environment.CARGO_TARGET_DIR | path join package cyber-nox-0.2.0.crate)
    let current = (extract-crate $current_archive ($cache | path join current))
    let current_vcs = (open ($current | path join .cargo_vcs_info.json))
    require ($current_vcs.git.sha1 == $head and $current_vcs.path_in_vcs == rs) 'current package VCS mismatch'
    let current_rust = (check-rust-against-git $root $head $current)
    require ($source == (committed-owner $root)) 'owning compiler inputs changed'
    let identity = {owner_head: $head, owner_files: $source, owner_files_sha256: ($source | to json --raw | hash sha256),
        environment: $environment, registry_record: $published, index_sha256: (digest $index),
        baseline: {root: $baseline, archive: $archive, archive_sha256: (digest $archive), vcs: $baseline_vcs,
            files: (package-inputs $baseline), rust_files: $baseline_rust},
        current: {root: $current, archive: $current_archive, archive_sha256: (digest $current_archive), vcs: $current_vcs,
            files: (package-inputs $current), rust_files: $current_rust},
        policy: {required_exact_locked_tag: 'red-unresolved', replacement_authorized: false, release_ready: false}}
    $identity | to json | save ($out | path join identities.json)
    open --raw $index | save ($out | path join registry-index.jsonl)
    {out: $out, baseline: $baseline, current: $current} | to json
}
