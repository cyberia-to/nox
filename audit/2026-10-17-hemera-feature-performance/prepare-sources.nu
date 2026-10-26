def require [ok: bool, msg: string] { if not $ok { error make {msg: $msg} } }
def sha [p: path] { open --raw $p | hash sha256 }
def object [index: table, cache: path, name: string] {
    let r = ($index | where path == $name | first)
    let p = ($cache | path join historical-objects objects $r.sha256)
    require ((sha $p) == $r.sha256) $'historical object ($name)'
    require ((open --raw $p | into binary | bytes length) == $r.bytes) 'historical size'
    $p
}
def untar [archive: path, dest: path] {
    require (not ($dest | path exists)) $'export already exists: ($dest)'
    let names = (do { ^tar -tf $archive } | complete)
    let modes = (do { ^tar -tvf $archive } | complete)
    require ($names.exit_code == 0 and $modes.exit_code == 0) 'archive readable'
    require (not ($names.stdout | lines | any {|p| $p =~ '(^/|(^|/)\.\.(/|$))'})) 'archive path'
    require (not ($modes.stdout | lines | any {|p| not ($p =~ '^[-d]')})) 'archive file kind'
    mkdir $dest
    let r = (do { ^tar -xf $archive -C $dest } | complete)
    require ($r.exit_code == 0) $r.stderr
}
def inventory [root: path, expected: table] {
    let paths = (glob --no-dir ($root | path join '**/*') | each {|p| $p | path relative-to $root} | sort)
    require ($paths == ($expected.path | sort)) $'full file set: ($root)'
    $expected | each {|e|
        let p = ($root | path join $e.path)
        let b = (open --raw $p | into binary)
        let mode = (if (ls --long $p | get 0.mode | str contains x) { '100755' } else { '100644' })
        let blob = ($b | ^git hash-object --stdin | str trim)
        let row = {path: $e.path, mode: $mode, blob: $blob, bytes: ($b | bytes length), sha256: ($b | hash sha256)}
        require ($mode == $e.mode and $blob == $e.blob) $'export Git identity ($root)/($e.path)'
        if 'sha256' in ($e | columns) { require ($row == $e) 'historical complete body/mode row' }
        $row
    }
}
def main [repo: path, cache: path] {
    let repo = ($repo | path expand)
    let cache = ($cache | path expand)
    cd $repo
    let out = ($repo | path join audit/2026-10-17-hemera-feature-performance)
    require (not ($out | path join sources.json | path exists)) 'one frozen export epoch'
    let index = (open audit/2026-10-13-hemera-no-std/index.json)
    let historical = (open --raw (object $index $cache code/support/plan/support/committed-inputs.json) | from json)
    let freeze = (open --raw (object $index $cache code/support/execution/source-freeze.json) | from json)
    let patch = (object $index $cache code/support/execution/source.patch)
    require ((sha $patch) == '387f9ccee4480bbfeb957ee9a2cbb7d959ac808db5635a3f296ee51b001f3899') 'accepted patch'
    cp $patch ($out | path join historical.patch)
    let pins = [
        {name: nox, revision: '64d091e5ad3f277cce77b88ae0de6eea4c49b4df'}
        {name: hemera, revision: '1c385e54e18885075f0dd12a0407ad0234e1bebf'}
        {name: strata, revision: '56aedb2d12b3126c601eb333419136d403614dbb'}
        {name: lens, revision: '8b26cef6587bee6a686f2cf163f6a8496931ee6a'}
    ]
    mkdir ($cache | path join archives)
    mut records = []
    for name in [nox hemera strata lens] {
        let h = ($historical | get $name)
        let archive = (object $index $cache $'code/support/plan/support/($name)-source.tar')
        require ((sha $archive) == $h.source_archive_sha256) 'historical archive identity'
        for cell in [A B P] {
            let root = ($cache | path join cells $cell $name)
            untar $archive $root
            let rows = (inventory $root $h.entries)
            $records = ($records | append {cell: $cell, repo: $name, root: $root, revision: $h.revision,
                archive_sha256: (sha $archive), entries: $rows})
        }
        let pin = ($pins | where name == $name | first)
        let source = (if $name == nox { $repo } else { $repo | path dirname | path join $name | path expand })
        let tree = (^git -c core.quotePath=false -C $source ls-tree -r $pin.revision | lines
            | parse --regex '^(?<mode>[0-9]+) (?<kind>[^ ]+) (?<blob>[0-9a-f]+)\t(?<path>.+)$')
        require ($tree | all {|r| $r.kind == blob and $r.mode in ['100644' '100755']}) 'regular committed source files'
        let tar = ($cache | path join archives $'current-($name).tar')
        let r = (do { ^git -C $source archive --format=tar --output $tar $pin.revision } | complete)
        require ($r.exit_code == 0) $r.stderr
        for cell in [C D] {
            let root = ($cache | path join cells $cell $name)
            untar $tar $root
            let rows = (inventory $root ($tree | select path mode blob))
            $records = ($records | append {cell: $cell, repo: $name, root: $root, revision: $pin.revision,
                archive_sha256: (sha $tar), entries: $rows})
        }
    }
    # Reproduce the reviewed patch by exact accepted bodies, checking every row.
    for p in [CLAUDE.md rs/Cargo.toml] {
        let suffix = (if $p == rs/Cargo.toml { '.txt' } else { '' })
        cp --force (object $index $cache $'code/support/execution/final-source/($p)($suffix)') ($cache | path join cells B nox $p)
    }
    let b = ($records | where cell == B and repo == nox | first)
    let b_expected = ($freeze.source_roots.nox | each {|r|
        let p = ($b.root | path join $r.path)
        {path: $r.path, mode: $r.mode, blob: (open --raw $p | ^git hash-object --stdin | str trim), bytes: $r.bytes, sha256: $r.sha256}
    })
    let b_rows = (inventory $b.root $b_expected)
    $records = ($records | each {|r| if $r.cell == B and $r.repo == nox { $r | update entries $b_rows } else { $r }})
    let c = ($records | where cell == C and repo == nox | first)
    let manifest = ($c.root | path join rs/Cargo.toml)
    let before = (open --raw $manifest)
    require (($before | str contains 'std = ["hemera/std"]') and ($before | str contains ', default-features = false')) 'current feature policy'
    let after = ($before | str replace 'std = ["hemera/std"]' 'std = []' | str replace ', default-features = false' '')
    $after | save --force $manifest
    let c_rows = ($c.entries | each {|r|
        if $r.path == rs/Cargo.toml {
            $r | update sha256 ($after | hash sha256) | update bytes ($after | into binary | bytes length)
                | update blob ($after | ^git hash-object --stdin | str trim)
        } else { $r }
    })
    $records = ($records | each {|r| if $r.cell == C and $r.repo == nox { $r | update entries $c_rows } else { $r }})
    for pair in [[A B historical] [C D current] [B D era]] {
        for name in [nox hemera strata lens] {
            let a = ($cache | path join cells $pair.0 $name)
            let b = ($cache | path join cells $pair.1 $name)
            let result = (do { ^git diff --no-index --no-ext-diff -- $a $b } | complete)
            require ($result.exit_code in [0 1]) 'full source diff'
            $result.stdout | save ($out | path join $'($pair.2)-($name).patch')
            require ($result.stderr == '') 'source diff stderr'
        }
    }
    for r in $records { inventory $r.root $r.entries | ignore }
    {base: '64d091e5ad3f277cce77b88ae0de6eea4c49b4df', exported_at: (date now | format date '%+'),
        command: ['nu' 'audit/2026-10-17-hemera-feature-performance/prepare-sources.nu' $repo $cache],
        script_sha256: (sha ($out | path join prepare-sources.nu)), historical_patch_sha256: (sha $patch),
        cells: $records, cells_sha256: ($records | to json --raw | hash sha256)
    } | to json | save ($out | path join sources.json)
    print 'exact A/B/C/D and unmeasured A-path control P source inventories frozen'
}
