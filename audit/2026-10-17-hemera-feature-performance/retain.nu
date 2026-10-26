use control.nu *
def main [repo: path, cache: path] {
    let repo = ($repo | path expand)
    let cache = ($cache | path expand)
    let out = ($repo | path join audit/2026-10-17-hemera-feature-performance)
    let stage = ($cache | path join portable)
    require (not ($stage | path exists)) 'portable retention already exists'
    require (not ($out | path join inputs.tar.gz | path exists)) 'retain capsule epoch'
    mkdir ($stage | path join archives) ($stage | path join binaries)
    let s = (open ($out | path join sources.json))
    let registry = (open ($out | path join registry.json))
    let builds = (open ($out | path join builds.json))
    let code = (open ($out | path join code.json))
    check-sources $s
    check-registry $registry
    for name in [nox hemera strata lens] {
        let a = ($s.cells | where cell == A and repo == $name | first)
        let d = ($s.cells | where cell == D and repo == $name | first)
        let historical = ($cache | path join historical-objects objects $a.archive_sha256)
        let current = ($cache | path join archives $'current-($name).tar')
        require ((sha $historical) == $a.archive_sha256 and (sha $current) == $d.archive_sha256) 'retained source archives'
        cp $historical ($stage | path join archives $'historical-($name).tar')
        cp $current ($stage | path join archives $'current-($name).tar')
    }
    for cell in [B C] {
        mkdir ($stage | path join controls $cell rs)
        cp ($cache | path join cells $cell nox rs/Cargo.toml) ($stage | path join controls $cell rs/Cargo.toml)
        if $cell == B { cp ($cache | path join cells B nox CLAUDE.md) ($stage | path join controls B CLAUDE.md) }
    }
    cp --recursive ($cache | path join registry-archives) $stage
    cp --recursive ($cache | path join sections) $stage
    mkdir ($stage | path join checksums) ($stage | path join cargo-home) ($stage | path join helpers)
    cp ($registry.cargo_home | path join config.toml) ($stage | path join cargo-home config.toml)
    for p in $registry.packages { cp ($p.source_root | path join .cargo-checksum.json) ($stage | path join checksums $'($p.name)-($p.version).json') }
    for b in $builds.builds {
        require ((sha $b.binary) == $b.sha256) 'retained binary identity'
        mkdir ($stage | path join binaries $b.cell)
        cp $b.binary ($stage | path join binaries $b.cell reduction)
    }
    for h in $code.helpers {
        require ((sha $h.path) == $h.sha256) 'retained helper identity'
        cp $h.path ($stage | path join helpers ($h.path | path basename))
    }
    let entries = (glob --no-dir ($stage | path join '**/*') | each {|p|
        {path: ($p | path relative-to $stage), sha256: (sha $p), bytes: (open --raw $p | into binary | bytes length),
            executable: (ls --long $p | get 0.mode | str contains x)}
    } | sort-by path)
    let archive = ($out | path join inputs.tar.gz)
    let result = (do { with-env {COPYFILE_DISABLE: '1'} { ^tar -czf $archive -C $stage . } } | complete)
    require ($result.exit_code == 0 and $result.stderr == '') 'portable capsule creation'
    {command: [nu audit/2026-10-17-hemera-feature-performance/retain.nu $repo $cache],
        source_base: $s.base, script_sha256: (sha ($out | path join retain.nu)), created: (date now | format date '%+'),
        archive_sha256: (sha $archive), entries: $entries,
        entries_sha256: ($entries | to json --raw | hash sha256),
        contents: 'exact historical/current source archives, treatment files, registry archives/checksums, binaries, every raw Mach-O section and audit helper binaries'}
        | to json | save ($out | path join inputs.json)
    print 'portable input/binary/section capsule retained'
}
