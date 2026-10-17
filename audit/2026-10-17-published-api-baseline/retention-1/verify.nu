use support.nu *

def receipt [dir: string, name: string, argv: list<string>, expected: int, flags: any = null] {
    let row = (open ($dir | path join $'($name).json'))
    require ($row.command == $argv and $row.exit_code == $expected and $row.expected_exit_code == $expected) $'command/outcome mismatch: ($name)'
    for stream in [stdout stderr] {
        require ((digest ($dir | path join $'($name).($stream)')) == ($row | get $'($stream)_sha256')) $'stream mismatch: ($name)'
    }
    if $flags != null {
        require ($row.env.RUSTUP_TOOLCHAIN == '1.95.0' and $row.env.RUSTFLAGS == $flags and $row.env.RUSTDOCFLAGS == $flags) $'toolchain/flags mismatch: ($name)'
        require ($row.env.CARGO_BUILD_JOBS == '2' and $row.env.CARGO_INCREMENTAL == '0') $'build settings mismatch: ($name)'
        require (($row.env.RUSTC | path basename) == rustc and ($row.env.RUSTDOC | path basename) == rustdoc) $'compiler mismatch: ($name)'
    }
    $row
}

def policy [value: record] {
    require ($value == {required_exact_locked_tag: 'red-unresolved', replacement_authorized: false, release_ready: false}) 'gate authority changed'
}

# Verification reads preserved roots as historical data. Only repo/cache arguments
# determine live filesystem inputs; removed task checkouts are never consulted.
def main [repo: string, cache: string] {
    let root = ($repo | path expand)
    let audit = ($root | path join audit/2026-10-17-published-api-baseline)
    let cache = ($cache | path expand)
    let pair_path = ($audit | path join pair-2 identities.json)
    let pair = (open $pair_path)
    let api_dir = ($audit | path join api-2)
    let api = (open ($api_dir | path join result.json))
    let feature_dir = ($audit | path join features-2)
    let features = (open ($feature_dir | path join result.json))
    require ((owner-inputs $root) == $pair.owner_files) 'current owning source inventory/bytes differ'
    require (($pair.owner_files | to json --raw | hash sha256) == $pair.owner_files_sha256) 'owning inventory hash differs'
    let ancestor = (do { ^git -C $root merge-base --is-ancestor $pair.owner_head HEAD } | complete)
    require ($ancestor.exit_code == 0) 'recorded owner commit is not an ancestor'
    require ($api.pair == pair-2 and $features.pair == pair-2 and $api.pair_sha256 == (digest $pair_path) and $features.pair_sha256 == (digest $pair_path)) 'package/run linkage differs'
    for value in [$pair.policy $api.policy $features.policy] { policy $value }
    for dir in [$api_dir $feature_dir] {
        let source = (open ($dir | path join tooling.json))
        for file in $source { require ((digest ($audit | path join $file.path)) == $file.sha256) 'runner source changed' }
        require (($source | to json --raw | hash sha256) == (open ($dir | path join result.json) | get source_sha256)) 'runner identity differs'
    }
    let prefix = (^rustup which --toolchain 1.95.0 rustc | str trim | path dirname)
    for tool in (open ($api_dir | path join tools.json)) {
        let path = (if $tool.name == cargo-semver-checks { which cargo-semver-checks | first | get path } else { $prefix | path join $tool.name })
        require ((digest $path) == $tool.sha256) $'installed tool differs: ($tool.name)'
        receipt $api_dir $'tool-($tool.name)' [$tool.path --version] 0 '' | ignore
    }
    let bundle = (open ($audit | path join bundle.json))
    let capsule = ($audit | path join evidence.tar.gz)
    require ((digest $capsule) == $bundle.sha256) 'evidence capsule changed'
    require ($bundle.pair == pair-2 and $bundle.api == api-2) 'capsule linkage differs'
    let expected_members = [baseline-default.json current-default.json baseline-std.json current-std.json baseline.crate current.crate]
    require (($bundle.files | get path | sort) == ($expected_members | sort)) 'capsule inventory differs'
    require ((^tar -tzf $capsule | lines | sort) == ($expected_members | sort)) 'capsule archive inventory differs'
    require (not ($cache | path exists)) 'verification cache already exists'
    mkdir $cache
    let extraction = (do { ^tar -xzf $capsule -C $cache } | complete)
    require ($extraction.exit_code == 0) 'capsule extraction failed'
    for file in $bundle.files { require ((digest ($cache | path join $file.path)) == $file.sha256) 'capsule file changed' }
    require ($pair.registry_record.cksum == '7c807c1b53e66cea79e8e130ca938450029af3f096984a2839fee6443a171b3f' and not $pair.registry_record.yanked) 'registry identity differs'
    require ((digest ($audit | path join pair-2 registry-index.jsonl)) == $pair.index_sha256) 'recorded registry response changed'
    let index_row = (open --raw ($audit | path join pair-2 registry-index.jsonl) | lines | each {|r| $r | from json} | where vers == '0.2.0' | first)
    require ($index_row == $pair.registry_record) 'registry record differs'
    for side in [baseline current] {
        let input = ($pair | get $side)
        let archive = ($cache | path join $'($side).crate')
        require ((digest $archive) == $input.archive_sha256) 'package archive differs'
        let package = (extract-crate $archive ($cache | path join $side))
        require ((package-inputs $package) == $input.files) 'package inventory/bytes differ'
        let expected_revision = (if $side == baseline { '6ab6e1a303355a49aaef6e069bffe5bfb3efa2a2' } else { $pair.owner_head })
        require ($input.vcs.git.sha1 == $expected_revision and $input.vcs.path_in_vcs == rs) 'package VCS differs'
        require ((open ($package | path join .cargo_vcs_info.json)) == $input.vcs) 'packaged VCS differs'
        require ((check-rust-against-git $root $expected_revision $package) == $input.rust_files) 'package Rust differs from revision'
    }
    let package_receipt = (open ($audit | path join pair-2 package-current.json))
    receipt ($audit | path join pair-2) package-current [cargo package -p cyber-nox --locked --offline --allow-dirty] 0 '-Dwarnings' | ignore
    let recorded_root = $package_receipt.cwd
    mut targets = []
    for feature in [default std] {
        let feature_args = (if $feature == std { [--features std] } else { [] })
        for side in [baseline current] {
            let name = $'($side)-($feature)'
            let input = ($pair | get $side)
            let artifact = ($api.artifacts | where name == $name | first)
            require ((digest ($cache | path join $'($name).json')) == $artifact.sha256) 'explicit API artifact differs'
            let args = ([cargo rustdoc --manifest-path ($input.root | path join Cargo.toml) -p cyber-nox --lib --locked --offline]
                | append $feature_args | append [-- -Z unstable-options --output-format json])
            let row = (receipt $api_dir $name $args 0 '')
            require ($row.cwd == $input.root and $row.env.RUSTC_BOOTSTRAP == '1') 'API extraction context differs'
            require ($artifact.path == ($row.env.CARGO_TARGET_DIR | path join doc nox.json)) 'API artifact/target mismatch'
            $targets = ($targets | append $row.env.CARGO_TARGET_DIR)
            receipt $api_dir $'($name)-features' ([cargo tree --manifest-path ($input.root | path join Cargo.toml) --locked --offline -e features -p cyber-nox] | append $feature_args) 0 '' | ignore
        }
        let old = ($api.artifacts | where name == $'baseline-($feature)' | first)
        let new = ($api.artifacts | where name == $'current-($feature)' | first)
        let row = (receipt $api_dir $'compare-($feature)' [cargo-semver-checks check-release --current-rustdoc $new.path --baseline-rustdoc $old.path --release-type patch --color never] 0 '')
        require ($row.cwd == $recorded_root) 'comparison cwd differs'
        require (open --raw ($api_dir | path join $'compare-($feature).stderr') | str contains '229 checks: 229 pass, 31 skip') 'comparison summary differs'
    }
    require (($targets | uniq | length) == 4 and ($api.artifacts | length) == 4) 'API artifact targets are not separate'
    require (($features.cases | get name | sort) == ([baseline-default baseline-std current-default current-std] | sort)) 'consumer case inventory differs'
    for case in $features.cases {
        let feature_args = (if ($case.name | str ends-with '-std') { [--features std] } else { [] })
        let expected = (if $case.name == current-default { 101 } else { 0 })
        let row = (receipt $feature_dir $case.name ([cargo check --locked --offline] | append $feature_args) $expected '-Dwarnings')
        require ($row.cwd == $case.fixture and $case.expected_exit_code == $expected) 'consumer context differs'
        let side = ($case.name | split row '-' | first)
        for file in $case.inputs {
            let path = (if $file.path == lib.rs { $audit | path join std-surface.rs.txt } else { $feature_dir | path join $'($side)-($file.path).txt' })
            require ((digest $path) == $file.sha256) 'consumer input differs'
        }
        receipt $feature_dir $'($case.name)-features' ([cargo tree --locked --offline -e features] | append $feature_args) 0 '-Dwarnings' | ignore
    }
    for side in [baseline current] { receipt $feature_dir $'lock-($side)' [cargo generate-lockfile --offline] 0 '-Dwarnings' | ignore }
    let refusal = (open --raw ($feature_dir | path join current-default.stderr))
    require (($refusal | str contains 'error[E0277]') and ($refusal | str contains 'error[E0599]') and ($refusal | str contains 'to_hex') and ($refusal | str contains 'std::io::Read')) 'consumer refusal differs'
    let history = ($audit | path join required-tag-history)
    let provenance = (open ($history | path join provenance.json))
    require ((digest ($root | path join $provenance.index)) == $provenance.index_sha256) 'historical index differs'
    for file in $provenance.records { require ((digest ($history | path join ($file.path | path basename))) == $file.sha256) 'historical required-tag evidence changed' }
    let original = (open ($history | path join receipt.json))
    require ($original.exit_code == 101 and '--locked' in $original.argv) 'required-tag gate was relabelled'
    require ((^git -C $root show 'v0.2.0:Cargo.lock' | from toml | get package | where name == cyber-nox | first | get version) == '0.1.2') 'historical lock identity differs'
    {owner_head: $pair.owner_head, owner_files_sha256: $pair.owner_files_sha256,
        package_pair: (digest $pair_path), capsule: $bundle.sha256,
        api_checks: {default: '229 pass / 31 skip', std: '229 pass / 31 skip'},
        feature_witness: 'baseline default/std pass; current default refuses E0277/E0599; current std passes',
        required_exact_locked_tag: 'red-unresolved', release_ready: false} | to json
}
