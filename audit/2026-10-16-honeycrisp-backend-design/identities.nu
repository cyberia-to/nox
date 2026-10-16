def main [repo: string, honey: string, registry: string, archive: string, package: string] {
    let root = ($repo | path expand)
    let honey = ($honey | path expand)
    let registry = ($registry | path expand)
    let package = ($package | path expand)
    cd $root
    let out = 'audit/2026-10-16-honeycrisp-backend-design'
    let locked = (open --raw Cargo.lock | from toml | get package | where name == acpu | first)
    let indexed = (open --raw ($out | path join registry-index.jsonl) | lines | each {|x| $x | from json}
        | where vers == '0.2.0' | first)
    let actual = (open --raw $archive | hash sha256)
    if $actual != $locked.checksum or $actual != $indexed.cksum {
        error make {msg: 'published package checksum mismatch'}
    }
    let entries = (^tar -tzf $archive | lines | where {|p| not ($p | str ends-with '/')})
    for entry in $entries {
        let relative = ($entry | str replace 'acpu-0.2.0/' '')
        if (open --raw ($registry | path join $relative) | hash sha256) != (^tar -xOzf $archive $entry | hash sha256) {
            error make {msg: $'registry extraction changed: ($relative)'}
        }
    }
    let sources = (^git -C $honey ls-files acpu | lines | where {|p| $p | str ends-with '.rs'} | sort | each {|p|
        let hash = (open --raw ($honey | path join $p) | hash sha256)
        let relative = ($p | str replace 'acpu/' '')
        if (open --raw ($package | path join $relative) | hash sha256) != $hash {
            error make {msg: $'normalized package source differs: ($p)'}
        }
        {path: $relative, owning_and_package_sha256: $hash}
    })
    {command: ['nu' 'audit/2026-10-16-honeycrisp-backend-design/identities.nu' $root $honey $registry $archive $package],
        nox_head: (^git rev-parse HEAD | str trim), honeycrisp_head: (^git -C $honey rev-parse HEAD | str trim),
        published: {version: '0.2.0', checksum: $actual, vcs: (open ($registry | path join .cargo_vcs_info.json)),
            extracted_files_verified: ($entries | length),
            index: {url: 'https://index.crates.io/ac/pu/acpu', sha256: (open --raw ($out | path join registry-index.jsonl) | hash sha256)},
            field_exports_sha256: (open --raw ($registry | path join src/field/mod.rs) | hash sha256)},
        local_package: {archive_sha256: (open --raw $'($package).crate' | hash sha256),
            manifest_sha256: (open --raw ($package | path join Cargo.toml) | hash sha256),
            lock_sha256: (open --raw ($package | path join Cargo.lock) | hash sha256),
            vcs: (open ($package | path join .cargo_vcs_info.json)), sources: $sources},
        limitation: 'Current source package still declares occupied 0.2.0. Source and normalized-package diagnostics have separate recorded locks; neither overrides production dependency identity.'
    } | to json | save ($out | path join identities.json)
    open --raw ($package | path join Cargo.toml) | save ($out | path join normalized-manifest.toml.txt)
}
