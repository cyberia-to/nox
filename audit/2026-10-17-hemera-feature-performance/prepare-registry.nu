def require [ok: bool, msg: string] { if not $ok { error make {msg: $msg} } }
def sha [p: path] { open --raw $p | hash sha256 }
def main [repo: path, cache: path, cargo_cache: path] {
    let repo = ($repo | path expand)
    let cache = ($cache | path expand)
    let cargo_cache = ($cargo_cache | path expand)
    let out = ($repo | path join audit/2026-10-17-hemera-feature-performance)
    let registry = ($cache | path join frozen-registry)
    let cargo_home = ($cache | path join cargo-home)
    require (not ($out | path join registry.json | path exists)) 'one registry freeze epoch'
    require (not ($registry | path exists)) 'registry already exists'
    let locks = ([A B C D P] | each {|c|
        let p = ($cache | path join cells $c nox Cargo.lock)
        {cell: $c, path: $p, sha256: (sha $p), lock: (open --raw $p | from toml)}
    })
    let packages = ($locks.lock.package | flatten | where {|r| $r.source? != null}
        | select name version source checksum | uniq | sort-by name version)
    require (($packages | uniq-by name version | length) == ($packages | length)) 'one exact version/checksum per package'
    require ($packages | all {|p| $p.source == 'registry+https://github.com/rust-lang/crates.io-index'}) 'known registry only'
    let archives = ($cache | path join registry-archives)
    mkdir $registry $cargo_home $archives
    mut records = []
    for p in $packages {
        let stem = $'($p.name)-($p.version)'
        let found = (glob ($cargo_cache | path join registry cache '*' $'($stem).crate'))
        require (($found | length) == 1) $'one cached archive for ($stem)'
        require ((sha $found.0) == $p.checksum) $'locked archive checksum ($stem)'
        let archive = ($archives | path join $'($stem).crate')
        cp $found.0 $archive
        let names = (do { ^tar -tzf $archive } | complete)
        let modes = (do { ^tar -tvzf $archive } | complete)
        require ($names.exit_code == 0 and $modes.exit_code == 0) 'crate readable'
        require (not ($names.stdout | lines | any {|f| ($f =~ '(^/|(^|/)\.\.(/|$))') or not ($f | str starts-with $'($stem)/')})) 'crate path'
        require (not ($modes.stdout | lines | any {|f| not ($f =~ '^[-d]')})) 'crate regular files'
        let r = (do { ^tar -xzf $archive -C $registry } | complete)
        require ($r.exit_code == 0) $r.stderr
        let root = ($registry | path join $stem)
        let expected = ($names.stdout | lines | where {|f| not ($f | str ends-with '/')} | each {|f| $f | str substring (($stem | str length) + 1)..} | sort)
        let files = (glob --no-dir ($root | path join '**/*') | each {|f| {path: ($f | path relative-to $root), sha256: (sha $f)}} | sort-by path)
        require ($files.path == $expected) 'exact archive file set'
        let check = {files: ($files | transpose --ignore-titles --header-row --as-record), package: $p.checksum}
        $check | to json | save ($root | path join .cargo-checksum.json)
        $records = ($records | append ($p | merge {archive: $archive, source_root: $root, files: $files,
            checksum_file_sha256: (sha ($root | path join .cargo-checksum.json))}))
    }
    let index = (open ($repo | path join audit/2026-10-13-hemera-no-std/index.json))
    let entry = ($index | where path == code/support/plan/support/cargo-home-config.toml.txt | first)
    let old = ($cache | path join historical-objects objects $entry.sha256)
    require ((sha $old) == $entry.sha256) 'historical Cargo config identity'
    let old_config = (open --raw $old)
    let config = ($old_config | str replace '/tmp/nox-no-std-registry' $registry)
    require ($config != $old_config) 'explicit registry path relocation'
    $config | save ($cargo_home | path join config.toml)
    {command: [nu audit/2026-10-17-hemera-feature-performance/prepare-registry.nu $repo $cache $cargo_cache],
        source_base: '64d091e5ad3f277cce77b88ae0de6eea4c49b4df', created: (date now | format date '%+'),
        script_sha256: (sha ($out | path join prepare-registry.nu)),
        locks: ($locks | reject lock), packages: $records,
        packages_sha256: ($records | to json --raw | hash sha256),
        original_config_sha256: $entry.sha256, original_config: $old_config,
        cargo_home: $cargo_home, config: $config, config_sha256: ($config | hash sha256)
    } | to json | save ($out | path join registry.json)
    print $'frozen ($records | length) locked registry archives and exact extracted bodies'
}
