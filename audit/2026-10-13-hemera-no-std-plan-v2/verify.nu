def payload [lookup: record, root: path, name: string] {
    open --raw ($root | path join objects ($lookup | get $name)) | into binary
}
def json [lookup: record, root: path, name: string] {
    payload $lookup $root $name | decode utf-8 | from json
}
def object-path [lookup: record, root: path, name: string] {
    $root | path join objects ($lookup | get $name)
}
def checked-tar [archive: path, dest: path, compressed: bool, allow_links: bool = false] {
    let names = if $compressed { ^tar -tzf $archive | complete } else { ^tar -tf $archive | complete }
    let types = if $compressed { ^tar -tvzf $archive | complete } else { ^tar -tvf $archive | complete }
    if $names.exit_code != 0 or $types.exit_code != 0 or ($names.stdout | lines | any {|p| $p =~ '(^/|(^|/)\.\.(/|$))'}) or ($types.stdout | lines | any {|p| not ($p =~ (if $allow_links { '^[-dl]' } else { '^[-d]' }))}) {
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
        let review = [("Nox Hemera no_std feature boundary — frozen PLAN v2\n" | into binary) $parts] | bytes collect
        if $review != (payload $lookup $dest plan/review.txt) or ($review | hash sha256) != $capsule.review_sha256 or ($review | bytes length) != $capsule.review_bytes or ($sections | length) != $capsule.sections { error make {msg: 'full review reconstruction'} }
        if (json $lookup $dest plan/omissions.json | get files) != ($support | where {|r| not ($r.file in $sections.file)}) { error make {msg: 'exact omissions'} }
        let inputs = ((json $lookup $dest plan/support/committed-inputs.json) | merge (json $lookup $dest plan/support/extra-exports.json))
        mut source_count = 0
        for row in ($inputs | transpose name info) {
            let tar = (object-path $lookup $dest $'plan/support/($row.name)-source.tar')
            if (open --raw $tar | hash sha256) != $row.info.source_archive_sha256 { error make {msg: 'source archive identity'} }
            let dir = ($dest | path join sources $row.name)
            let names = (checked-tar $tar $dir false ($row.name == trident))
            if ($names | sort) != ($row.info.entries.path | sort) { error make {msg: 'source archive set'} }
            for file in $row.info.entries {
                let path = ($dir | path join $file.path)
                let stat = (ls --long $path | get 0)
                let b = if $file.mode == '120000' {
                    let target = (^readlink $path | complete)
                    if $stat.type != symlink or $target.exit_code != 0 { error make {msg: 'expected exact symlink'} }
                    $target.stdout | str trim --right --char "\n" | into binary
                } else { open --raw $path | into binary }
                let blob = ($b | ^git hash-object --stdin | complete)
                if ($file.mode != '120000' and ($stat.type != file or ($stat.mode | str contains 'x') != ($file.mode == '100755'))) or not ($file.mode in ['100644' '100755' '120000']) or ($b | bytes length) != $file.bytes or ($b | hash sha256) != $file.sha256 or $blob.exit_code != 0 or ($blob.stdout | str trim) != $file.blob { error make {msg: $'exact Git body/mode ($row.name)/($file.path)'} }
                $source_count = $source_count + 1
            }
        }
        if $source_count != 1287 or $inputs.nox.revision != $capsule.parent { error make {msg: 'source epochs'} }
        let old = (open --raw ($dest | path join sources nox rs Cargo.toml))
        let proposed = ($old | str replace 'std = []' 'std = ["hemera/std"]' | str replace 'hemera = { package = "cyber-hemera", version = "0.3", path = "../../hemera/rs" }' 'hemera = { package = "cyber-hemera", version = "0.3", path = "../../hemera/rs", default-features = false }')
        if ($proposed | into binary) != (payload $lookup $dest plan/support/proposed-Cargo.toml.txt) or ($old | into binary) != (payload $lookup $dest plan/support/nox/rs/Cargo.toml.txt) or $manifest.production_changes_applied { error make {msg: 'exact unapplied proposal'} }
        let records = ($support | where {|r| $r.file =~ '^support/records/[^/]+/receipt.json$'})
        if ($records | length) != 67 { error make {msg: 'record count'} }
        let runners = ([capture.rb capture-initial.rb parent-matrix.rb parent-matrix-corrected.rb extra-parent.rb parent-api-json.rb package-h264-parent-corrected.rb] | each {|p| $lookup | get $'plan/support/($p)'})
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
# Proposed policy change is only the exact appended durable gate.
let claude = (open --raw ($dest | path join sources nox CLAUDE.md) | into binary)
let gate = (payload $lookup $dest plan/support/proposed-gate.md)
if ([$claude $gate] | bytes collect) != (payload $lookup $dest plan/support/proposed-CLAUDE.md.txt) { error make {msg: 'exact named-gate append'} }
let cfg = (json $lookup $dest plan/support/feature-audit/hemera-cfg-sites.json)
mut sites = []
for entry in ($inputs.hemera.entries | where {|r| ($r.path | str ends-with '.rs') or ($r.path | str ends-with '.toml')}) {
    let body = (open --raw ($dest | path join sources hemera $entry.path))
    for row in ($body | lines | enumerate) {
        if ($row.item =~ '#\[cfg[^\n]*(feature\s*=\s*"std"|hemera_portable)') or ($entry.path == 'rs/Cargo.toml' and ($row.item | str contains 'cfg(hemera_portable)')) {
            $sites = ($sites | append {path: $entry.path, line: ($row.index + 1), text: ($row.item | str trim), source_sha256: ($body | hash sha256)})
        }
    }
}
if $sites != ($cfg.sites | select path line text source_sha256) or ($sites | length) != 18 { error make {msg: 'complete cfg inventory'} }
let reverse = (json $lookup $dest plan/support/topology/reverse-manifests.json)
for r in $reverse {
    if ($r.body | hash sha256) != $r.sha256 { error make {msg: 'manifest text hash'} }
    if $r.repository in [trident nox] {
        if (open --raw ($dest | path join sources $r.repository $r.path)) != $r.body { error make {msg: 'manifest versus exact source archive'} }
    }
}
let inherited = [
    ['38-parent-thumb-build' 101 "error[E0463]: can't find crate for `std`"]
    ['47-parent-docs' 101 'error: unresolved link to `11`']
    ['56-parent-honeycrisp' 101 'error[E0425]: cannot find function `ntt_forward` in module `acpu::field`']
    ['57-parent-clippy-correct-driver' 101 'error: this expression creates a reference which is immediately dereferenced by the compiler']
    ['59-parent-trident-prerequisite' 101 'error: failed to select a version for the requirement `clap = "^4"`']
    ['64-parent-api-json-tag' 101 'error: cannot update the lock file']
]
for row in $inherited {
    let r = (json $lookup $dest $'plan/support/records/($row.0)/receipt.json')
    let b = (payload $lookup $dest $'plan/support/records/($row.0)/stderr' | decode utf-8)
    if $r.exit_code != $row.1 or not ($b | str contains $row.2) { error make {msg: $'retained red ($row.0)'} }
}
let green = ['39-parent-host-no-default' '40-parent-workspace-debug' '41-parent-workspace-release' '42-parent-std' '43-parent-parallel' '44-parent-brakedown' '48-parent-nightly-doc' '49-parent-reduction-benchmark' '50-parent-h264-release' '51-parent-package-host' '52-parent-cli-build' '53-parent-cli-test' '54-parent-std-feature-tree' '55-parent-wgpu' '65-parent-package-h264-lock' '66-parent-package-h264-debug' '67-parent-package-h264-release']
for n in $green {
    if (json $lookup $dest $'plan/support/records/($n)/receipt.json' | get exit_code) != 0 { error make {msg: $'retained green ($n)'} }
}
if (json $lookup $dest plan/support/records/58-parent-fmt-correct-tool/receipt.json | get exit_code) != 1 { error make {msg: 'retained format red'} }
for n in 19..37 {
    let rows = ($records | where {|r| $r.file | str starts-with $'support/records/($n)-'})
    if ($rows | length) != 1 { error make {msg: 'initial harness receipt coverage'} }
    let r = (json $lookup $dest $'plan/($rows.0.file)')
    let b = (payload $lookup $dest $'plan/($rows.0.file | path dirname)/stderr' | decode utf-8)
    if $r.exit_code != 101 or not ($b | str contains 'no such command: `+') { error make {msg: 'historical +toolchain failure'} }
}
for n in ['66-parent-package-h264-debug' '67-parent-package-h264-release'] {
    if not (payload $lookup $dest $'plan/support/records/($n)/stdout' | decode utf-8 | str contains 'test result: ok. 4 passed; 0 failed; 0 ignored;') { error make {msg: 'registry four controls'} }
}
for r in (json $lookup $dest plan/support/h264-package-parent-inputs.json) {
    let b = (payload $lookup $dest $'plan/support/h264-package-parent/($r.file)')
    if ($b | hash sha256) != $r.sha256 or ($b | bytes length) != $r.bytes { error make {msg: 'package harness identity'} }
}
for p in [src/h264.rs src/h264/chroma_dc_formula.rs src/h264/tests.rs src/lib.rs tests.rs original-h264_chroma_dc.rs.txt] {
    if (payload $lookup $dest $'plan/support/h264-probe/($p)') != (payload $lookup $dest $'plan/support/h264-package-parent/($p)') { error make {msg: 'same actual H264 kernel'} }
}
let api = (json $lookup $dest plan/support/api-json/inputs.json)
if ($api.completed | length) != 2 { error make {msg: 'two successful JSON emissions'} }
let roots = ($api.completed | each {|r|
    let b = (payload $lookup $dest $'plan/support/api-json/($r.role).json')
    let result = (json $lookup $dest $'plan/support/records/($r.record)/receipt.json')
    if ($b | hash sha256) != $r.sha256 or ($b | bytes length) != $r.bytes or $result.exit_code != 0 { error make {msg: 'actual emitted JSON bytes'} }
    $result.env.CARGO_TARGET_DIR
})
if ($roots | uniq | length) != 2 { error make {msg: 'distinct actual rustdoc roots'} }
let bench = (json $lookup $dest plan/support/benchmark/summary.json)
if ($bench.cases | length) != 9 { error make {msg: 'nine parent benchmark cases'} }
for r in $bench.cases {
    let b = (payload $lookup $dest $'plan/($r.file)')
    let j = ($b | decode utf-8 | from json)
    if ($b | hash sha256) != $r.sha256 or $j.median != $r.median or $j.mean != $r.mean { error make {msg: 'exact raw benchmark estimates'} }
}
let old_review = (json $lookup $dest plan/support/prior-review/stdout.json)
if $old_review.result != (payload $lookup $dest plan/support/prior-review/verdict.md | decode utf-8) or not ($old_review.result | str contains 'REQUEST CHANGES') { error make {msg: 'historical rejection preserved'} }
let completion = (json $lookup $dest plan/support/prior-review/completion.json)
if (payload $lookup $dest plan/support/prior-review/stdout.json | hash sha256) != $completion.stdout_sha256 or (payload $lookup $dest plan/support/prior-review/stderr.txt | hash sha256) != $completion.stderr_sha256 { error make {msg: 'prior review raw binding'} }
let prior = (json $lookup $dest plan/support/prior-review/plan-v1-manifest.json)
if (payload $lookup $dest plan/support/prior-review/plan-v1-review.txt | hash sha256) != $prior.review_sha256 { error make {msg: 'unchanged v1 input'} }
        for who in [author root] {
            let verified = (json $lookup $dest $'verification/($who).json')
            if $verified.status != PASS or $verified.review_sha256 != $capsule.review_sha256 { error make {msg: 'historical independent reconstruction receipt'} }
        }
        print ({status: PASS, logical_payloads: ($index | length), unique_objects: ($objects | length), review_bytes: ($review | bytes length), review_sha256: $capsule.review_sha256, sections: ($sections | length), support_files: ($support | length), source_entries: $source_count, records: ($records | length), raw_streams: (2 * ($records | length)), packaged_rust: ($packaged_rust | length), production_changes_applied: false, cfg_sites: ($sites | length), reverse_manifest_bodies: ($reverse | length), benchmark_cases: ($bench.cases | length), independent_parent_json_roots: $roots} | to json)
    } catch {|e|
        rm --recursive --force $dest
        error make {msg: $e.msg}
    }
    rm --recursive --force $dest
}
