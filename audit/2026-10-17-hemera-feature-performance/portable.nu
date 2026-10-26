use control.nu [require sha]
def untar [archive: path, dest: path, gzip: bool] {
    let names = (if $gzip { ^tar -tzf $archive | complete } else { ^tar -tf $archive | complete })
    let types = (if $gzip { ^tar -tvzf $archive | complete } else { ^tar -tvf $archive | complete })
    require ($names.exit_code == 0 and $types.exit_code == 0) 'portable archive readable'
    require (not ($names.stdout | lines | any {|p| $p =~ '(^/|(^|/)\.\.(/|$))'})) 'portable archive path'
    require (not ($types.stdout | lines | any {|p| not ($p =~ '^[-d]')})) 'portable regular files'
    mkdir $dest
    let result = (if $gzip { ^tar -xzf $archive -C $dest | complete } else { ^tar -xf $archive -C $dest | complete })
    require ($result.exit_code == 0) 'portable archive extraction'
}
export def load-portable [out: path] {
    let manifest = (open ($out | path join inputs.json))
    require ((sha ($out | path join inputs.tar.gz)) == $manifest.archive_sha256) 'portable capsule identity'
    require (($manifest.entries | to json --raw | hash sha256) == $manifest.entries_sha256) 'portable index identity'
    let root = (^mktemp -d /tmp/nox-feature-performance-verify.XXXXXX | str trim)
    require ($root | str starts-with '/tmp/nox-feature-performance-verify.') 'temporary verification directory'
    try {
        let payload = ($root | path join payload)
        untar ($out | path join inputs.tar.gz) $payload true
        let paths = (glob --no-dir ($payload | path join '**/*') | each {|p| $p | path relative-to $payload} | sort)
        require ($paths == $manifest.entries.path) 'portable complete file set'
        for e in $manifest.entries {
            let p = ($payload | path join $e.path)
            require ((sha $p) == $e.sha256 and (open --raw $p | into binary | bytes length) == $e.bytes) 'portable payload body'
            require ((ls --long $p | get 0.mode | str contains x) == $e.executable) 'portable payload mode'
        }
        let original_sources = (open ($out | path join sources.json))
        require (($original_sources.cells | to json --raw | hash sha256) == $original_sources.cells_sha256) 'original source manifest'
        let cells = ($original_sources.cells | each {|c|
            let era = (if $c.cell in [A B P] { 'historical' } else { 'current' })
            let archive = ($payload | path join archives $'($era)-($c.repo).tar')
            require ((sha $archive) == $c.archive_sha256) 'portable source archive identity'
            let dest = ($root | path join cells $c.cell $c.repo)
            untar $archive $dest false
            if $c.repo == nox and $c.cell in [B C] {
                cp --force ($payload | path join controls $c.cell rs/Cargo.toml) ($dest | path join rs/Cargo.toml)
                if $c.cell == B { cp --force ($payload | path join controls B CLAUDE.md) ($dest | path join CLAUDE.md) }
            }
            $c | update root $dest
        })
        let sources = ($original_sources | upsert cells $cells | update cells_sha256 ($cells | to json --raw | hash sha256))
        let original_registry = (open ($out | path join registry.json))
        require (($original_registry.packages | to json --raw | hash sha256) == $original_registry.packages_sha256) 'original registry manifest'
        let packages = ($original_registry.packages | each {|p|
            let stem = $'($p.name)-($p.version)'
            let archive = ($payload | path join registry-archives $'($stem).crate')
            let dest = ($root | path join registry)
            untar $archive $dest true
            cp ($payload | path join checksums $'($stem).json') ($dest | path join $stem .cargo-checksum.json)
            $p | update archive $archive | update source_root ($dest | path join $stem)
        })
        let registry = ($original_registry | update packages $packages
            | update packages_sha256 ($packages | to json --raw | hash sha256)
            | update cargo_home ($payload | path join cargo-home))
        {root: $root, payload: $payload, sources: $sources, registry: $registry}
    } catch {|e|
        print --stderr ($e | to nuon)
        rm --recursive $root
        error make {msg: $e.msg}
    }
}
