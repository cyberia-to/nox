# Run after the source-bracketed packaging and normalized-thumb commands.
def main [repo: string, package: string] {
    let root = ($repo | path expand)
    let pkg = ($package | path expand)
    cd $root
    let out = 'audit/2026-10-16-release-hygiene'
    let sources = (^git ls-files rs | lines | where {|f| $f | str ends-with '.rs'} | sort | each {|f|
        let relative = ($f | str replace 'rs/' '')
        let owning = (open --raw $f | hash sha256)
        let packaged = (open --raw ($pkg | path join $relative) | hash sha256)
        if $owning != $packaged { error make {msg: $'package source mismatch: ($f)'} }
        {path: $relative, owning_and_package_sha256: $owning}
    })
    let manifest = (open --raw ($pkg | path join Cargo.toml))
    let lock = (open --raw ($pkg | path join Cargo.lock) | from toml)
    {head: (^git rev-parse HEAD | str trim),
        source_sha256: (open ($out | path join sources-reviewed.json)).source_sha256,
        command: ['nu' 'audit/2026-10-16-release-hygiene/package-proof.nu' $root $pkg],
        package_root: $pkg, crate_sha256: (open --raw $'($pkg).crate' | hash sha256),
        normalized_manifest_sha256: ($manifest | hash sha256),
        lock_sha256: (open --raw ($pkg | path join Cargo.lock) | hash sha256),
        vcs: (open ($pkg | path join .cargo_vcs_info.json)),
        registry_core: ($lock.package | where name in [cyber-hemera strata-nebu]),
        rust_sources: $sources,
        limitation: 'Dirty local package at occupied 0.2.0; verified packaging is not a release or publication. Package identities captured after the gates; owning source closure is bracketed by each receipt.'
    } | to json | save ($out | path join package-identity.json)
    $manifest | save ($out | path join normalized-manifest.toml.txt)
}
