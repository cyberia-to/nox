def payload [lookup: record, root: path, name: string] {
    open --raw ($root | path join objects ($lookup | get $name)) | into binary
}
def json [lookup: record, root: path, name: string] {
    payload $lookup $root $name | decode utf-8 | from json
}
def object-path [lookup: record, root: path, name: string] {
    $root | path join objects ($lookup | get $name)
}
def checked-tar [archive: path, dest: path, compressed: bool] {
    let names = if $compressed { ^tar -tzf $archive | complete } else { ^tar -tf $archive | complete }
    let types = if $compressed { ^tar -tvzf $archive | complete } else { ^tar -tvf $archive | complete }
    if $names.exit_code != 0 or $types.exit_code != 0 or ($names.stdout | lines | any {|p| $p =~ '(^/|(^|/)\.\.(/|$))'}) or ($types.stdout | lines | any {|p| not ($p =~ '^[-d]')}) {
        error make {msg: 'unsafe or unreadable tar entries'}
    }
    mkdir $dest
    let result = if $compressed { ^tar -xzf $archive -C $dest | complete } else { ^tar -xf $archive -C $dest | complete }
    if $result.exit_code != 0 { error make {msg: $result.stderr} }
    $names.stdout | lines | where {|p| not ($p | str ends-with '/')}
}
def main [audit: path = '.'] {
    let audit = ($audit | path expand)
    let capsule = (open ($audit | path join capsule.json))
    let index = (open ($audit | path join index.json))
    let archive = ($audit | path join evidence.tar.gz)
    if (open --raw $archive | hash sha256) != $capsule.archive_sha256 or (open --raw ($audit | path join index.json) | hash sha256) != $capsule.index_sha256 {
        error make {msg: 'capsule archive/index hash'}
    }
    if ($index | length) != $capsule.logical_payloads or ($index.path | uniq | length) != $capsule.logical_payloads { error make {msg: 'logical inventory'} }
    let lookup = ($index | select path sha256 | transpose --ignore-titles --header-row --as-record)
    let sizes = ($index | select path bytes | transpose --ignore-titles --header-row --as-record)
    let objects = ($index.sha256 | uniq | sort)
    let groups = ($index | group-by sha256)
    if ($objects | length) != $capsule.unique_objects or ($objects | any {|h| not ($h =~ '^[0-9a-f]{64}$')}) { error make {msg: 'object inventory'} }
    let made = (^mktemp -d /tmp/nox-no-std-verify.XXXXXX | complete)
    if $made.exit_code != 0 { error make {msg: $made.stderr} }
    let dest = ($made.stdout | str trim)
    try {
        let entries = (checked-tar $archive $dest true)
        if ($entries | sort) != ($objects | each {|h| $'objects/($h)'}) { error make {msg: 'outer archive set'} }
        for h in $objects {
            let p = ($dest | path join objects $h)
            let b = (open --raw $p | into binary)
            if ($p | path type) != file or ($b | hash sha256) != $h or ($groups | get $h | any {|r| $r.bytes != ($b | bytes length)}) { error make {msg: 'object bytes'} }
        }
        let manifest = (json $lookup $dest plan/manifest.json)
        let support = (json $lookup $dest plan/support-index.json)
        if ($support | length) != $manifest.support_files { error make {msg: 'support count'} }
        for r in $support {
            if ($lookup | get $'plan/($r.file)') != $r.sha256 or ($sizes | get $'plan/($r.file)') != $r.bytes { error make {msg: 'support identity'} }
        }
        let sections = (json $lookup $dest plan/sections.json)
        let parts = ($sections | enumerate | each {|v|
            let s = $v.item
            let b = (payload $lookup $dest $'plan/($s.file)')
            if $s.number != ($v.index + 1) or ($b | bytes length) != $s.bytes or ($b | hash sha256) != $s.sha256 { error make {msg: 'section'} }
            let head = $"\n===== ($s.number): ($s.title) =====\nfile: ($s.file)\nbytes: ($s.bytes); sha256: ($s.sha256)\n"
            let extra = if 'source' in ($s | columns) {
                let full = (payload $lookup $dest $'plan/($s.source)')
                if ($full | hash sha256) != $s.source_sha256 or ($full | bytes at $s.byte_offset..($s.byte_offset + $s.byte_length - 1)) != $b { error make {msg: 'exact excerpt'} }
                $"excerpt of ($s.source) byte offset ($s.byte_offset), length ($s.byte_length); full sha256: ($s.source_sha256)\n"
            } else { '' }
            [($head | into binary) ($extra | into binary) ("----- BEGIN -----\n" | into binary) $b ("\n----- END -----\n" | into binary)] | bytes collect
        } | bytes collect)
        let review = [("Nox Hemera no_std feature boundary — frozen PLAN v1\n" | into binary) $parts] | bytes collect
        if $review != (payload $lookup $dest plan/review.txt) or ($review | hash sha256) != $capsule.review_sha256 or ($review | bytes length) != $capsule.review_bytes or ($sections | length) != $capsule.sections { error make {msg: 'full review reconstruction'} }
        if (json $lookup $dest plan/omissions.json | get files) != ($support | where {|r| not ($r.file in $sections.file)}) { error make {msg: 'exact omissions'} }
        let inputs = (json $lookup $dest plan/support/committed-inputs.json)
        mut source_count = 0
        for row in ($inputs | transpose name info) {
            let tar = (object-path $lookup $dest $'plan/support/($row.name)-source.tar')
            if (open --raw $tar | hash sha256) != $row.info.source_archive_sha256 { error make {msg: 'source archive identity'} }
            let dir = ($dest | path join sources $row.name)
            let names = (checked-tar $tar $dir false)
            if ($names | sort) != ($row.info.entries.path | sort) { error make {msg: 'source archive set'} }
            for file in $row.info.entries {
                let path = ($dir | path join $file.path)
                let stat = (ls --long $path | get 0)
                let b = (open --raw $path | into binary)
                let blob = ($b | ^git hash-object --stdin | complete)
                if $stat.type != file or ($stat.mode | str contains 'x') != ($file.mode == '100755') or not ($file.mode in ['100644' '100755']) or ($b | bytes length) != $file.bytes or ($b | hash sha256) != $file.sha256 or $blob.exit_code != 0 or ($blob.stdout | str trim) != $file.blob { error make {msg: $'exact Git body/mode ($row.name)/($file.path)'} }
                $source_count = $source_count + 1
            }
        }
        if $source_count != 536 or $inputs.nox.revision != $capsule.parent { error make {msg: 'source epochs'} }
        let old = (open --raw ($dest | path join sources nox rs Cargo.toml))
        let proposed = ($old | str replace 'std = []' 'std = ["hemera/std"]' | str replace 'hemera = { package = "cyber-hemera", version = "0.3", path = "../../hemera/rs" }' 'hemera = { package = "cyber-hemera", version = "0.3", path = "../../hemera/rs", default-features = false }')
        if ($proposed | into binary) != (payload $lookup $dest plan/support/proposed-Cargo.toml.txt) or ($old | into binary) != (payload $lookup $dest plan/support/nox/rs/Cargo.toml.txt) or $manifest.production_changes_applied { error make {msg: 'exact unapplied proposal'} }
        let records = ($support | where {|r| $r.file =~ '^support/records/[^/]+/receipt.json$'})
        if ($records | length) != 18 { error make {msg: 'record count'} }
        let runners = [($lookup | get 'plan/support/capture.rb') ($lookup | get 'plan/support/capture-initial.rb')]
        for entry in $records {
            let r = (json $lookup $dest $'plan/($entry.file)')
            let parent = ($entry.file | path dirname)
            if not ($r.runner_sha256 in $runners) { error make {msg: 'capture runner identity'} }
            for stream in [stdout stderr] {
                let b = (payload $lookup $dest $'plan/($parent)/($stream)')
                let expected = ($r | get $stream)
                if ($b | hash sha256) != $expected.sha256 or ($b | bytes length) != $expected.bytes { error make {msg: 'raw stream'} }
            }
        }
        for n in ['11-parent-thumb-isolated' '14-parent-package-thumb'] {
            let r = (json $lookup $dest $'plan/support/records/($n)/receipt.json')
            let err = (payload $lookup $dest $'plan/support/records/($n)/stderr' | decode utf-8)
            if $r.exit_code != 101 or not ($err | str contains "error[E0463]: can't find crate for `std`") { error make {msg: 'retained thumb failure'} }
        }
        for pair in [['13-parent-host-lib' 161] ['16-parent-h264-locked' 4]] {
            let r = (json $lookup $dest $'plan/support/records/($pair.0)/receipt.json')
            let out = (payload $lookup $dest $'plan/support/records/($pair.0)/stdout' | decode utf-8)
            if $r.exit_code != 0 or not ($out | str contains $'test result: ok. ($pair.1) passed; 0 failed; 0 ignored;') { error make {msg: 'measured host control'} }
        }
        if (json $lookup $dest plan/support/records/12-parent-package/receipt.json | get exit_code) != 0 { error make {msg: 'parent package command'} }
        mut package_names = []
        for pair in [[parent-package.crate package] [published-nox-0.2.0.crate published]] {
            let names = (checked-tar (object-path $lookup $dest $'plan/support/($pair.0)') ($dest | path join $pair.1) true)
            if ($names | any {|p| not ($p | str starts-with 'cyber-nox-0.2.0/')}) { error make {msg: 'package namespace'} }
            if $pair.1 == package { $package_names = $names }
        }
        let packaged_rust = ($package_names | where {|p| $p | str ends-with '.rs'})
        if ($packaged_rust | length) != 48 { error make {msg: 'package Rust scope'} }
        for p in $packaged_rust {
            let relative = ($p | str replace 'cyber-nox-0.2.0/' '')
            if (open --raw ($dest | path join package $p) | into binary) != (open --raw ($dest | path join sources nox rs $relative) | into binary) { error make {msg: 'unchanged package Rust'} }
        }
        let bridge = (json $lookup $dest plan/support/topology/nox-published-origin.json)
        for r in $bridge.files {
            let old_body = (open --raw ($dest | path join published cyber-nox-0.2.0 $r.path) | into binary)
            let body = (open --raw ($dest | path join sources nox rs $r.path) | into binary)
            if ($old_body | hash sha256) != $r.published_sha256 or ($body | hash sha256) != $r.origin_sha256 or ($old_body == $body) != $r.equal { error make {msg: 'published/current source bridge'} }
        }
        for n in [Cargo.toml Cargo.lock] {
            if (open --raw ($dest | path join package cyber-nox-0.2.0 $n) | into binary) != (payload $lookup $dest $'plan/support/package/($n).txt') { error make {msg: 'exact normalized metadata'} }
        }
        let controls = (json $lookup $dest plan/support/h264-probe-inputs.json)
        for r in $controls {
            let relative = ($r.output | str replace '/tmp/nox-no-std-h264-probe/' '')
            let b = (payload $lookup $dest $'plan/support/h264-probe/($relative)')
            if ($b | hash sha256) != $r.output_sha256 or $r.revision != $capsule.kadek { error make {msg: 'frozen actual kernel control'} }
            if ($r.source | str starts-with 'kadek-ref/src/') and ($b | hash sha256) != $r.source_sha256 { error make {msg: 'unchanged kernel source'} }
        }
        let original = (payload $lookup $dest plan/support/h264-probe/original-h264_chroma_dc.rs.txt | decode utf-8)
        let start = ($original | str index-of 'fn config(')
        let end = ($original | str index-of "#[test]\nfn every_independent_ffmpeg_vector_matches_the_interpreter")
        let part = ($original | str substring $start..<($end))
        let control = ("use crate::h264::{chroma_dc_420, ChromaDc420, ChromaDc420Scratch, ChromaDcError};\n\n" + $part | into binary)
        if $control != (payload $lookup $dest plan/support/h264-probe/tests.rs) or ($original | hash sha256) != ($controls | where source == 'kadek-ref/tests/h264_chroma_dc.rs' | get 0.source_sha256) { error make {msg: 'exact independent literal extraction'} }
        for who in [author root] {
            let verified = (json $lookup $dest $'verification/($who).json')
            if $verified.status != PASS or $verified.review_sha256 != $capsule.review_sha256 { error make {msg: 'historical independent reconstruction receipt'} }
        }
        print ({status: PASS, logical_payloads: ($index | length), unique_objects: ($objects | length), review_bytes: ($review | bytes length), review_sha256: $capsule.review_sha256, sections: ($sections | length), support_files: ($support | length), source_entries: $source_count, records: ($records | length), raw_streams: (2 * ($records | length)), packaged_rust: ($packaged_rust | length), production_changes_applied: false} | to json)
    } catch {|e|
        rm --recursive --force $dest
        error make {msg: $e.msg}
    }
    rm --recursive --force $dest
}
