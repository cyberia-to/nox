use control.nu *
def main [repo: path, cache: path] {
    let repo = ($repo | path expand)
    let cache = ($cache | path expand)
    let out = ($repo | path join audit/2026-10-17-hemera-feature-performance)
    let sources = (open ($out | path join sources.json))
    let registry = (open ($out | path join registry.json))
    require (not ($out | path join builds.json | path exists)) 'one fixed build epoch'
    require (not ($cache | path join targets | path exists)) 'each target must start absent'
    check-sources $sources
    check-registry $registry
    let inherited = ($env | select --optional PATH RUSTUP_HOME RUSTUP_TOOLCHAIN CARGO_HOME CARGO_TARGET_DIR
        RUSTC RUSTDOC RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER RUSTFLAGS RUSTDOCFLAGS
        CARGO_ENCODED_RUSTFLAGS CARGO_ENCODED_RUSTDOCFLAGS CARGO_BUILD_TARGET CARGO_BUILD_JOBS
        CARGO_INCREMENTAL CARGO_NET_OFFLINE CC CXX LD AR SDKROOT MACOSX_DEPLOYMENT_TARGET DEVELOPER_DIR)
    let run_env = (environment $cache A)
    let rustup = (which rustup | first | get path)
    let cargo = (^$rustup which --toolchain 1.95.0 cargo | str trim)
    let clang = (^xcrun --find clang | str trim)
    let ld = (^xcrun --find ld | str trim)
    let objdump = (^xcrun --find llvm-objdump | str trim)
    let nm = (^xcrun --find llvm-nm | str trim)
    let sdk = (^xcrun --show-sdk-path | str trim)
    let sysroot = ($run_env.RUSTC | path dirname | path dirname)
    let tool_paths = ([$rustup $cargo $run_env.RUSTC $run_env.RUSTDOC $clang $ld $objdump $nm
        (which nu | first | get path) /usr/bin/env /usr/bin/xcrun]
        | append (glob --no-dir ($sysroot | path join lib '*.dylib'))
        | append (glob --no-dir ($sysroot | path join lib/rustlib/aarch64-apple-darwin/lib '*'))
        | append (glob --no-dir ($sdk | path join SDKSettings.*)) | uniq | sort)
    let tool_files = ($tool_paths | each {|p| {path: $p, sha256: (sha $p)}})
    let observations = ([
        {name: rustc, command: [$run_env.RUSTC -Vv]}, {name: rustdoc, command: [$run_env.RUSTDOC -Vv]},
        {name: cargo, command: [$rustup run 1.95.0 cargo -Vv]}, {name: clang, command: [$clang --version]},
        {name: linker, command: [$ld -v]}, {name: xcode, command: [/usr/bin/xcodebuild -version]},
        {name: sdk, command: [/usr/bin/xcrun --show-sdk-version]}, {name: os, command: [/usr/bin/sw_vers]},
        {name: hardware, command: [/usr/sbin/sysctl hw.model hw.machine hw.memsize hw.ncpu hw.physicalcpu hw.logicalcpu machdep.cpu.brand_string]},
        {name: kernel, command: [/usr/bin/uname -a]}
    ] | each {|r| capture $r.command $run_env $repo $out $'tool-($r.name)'})
    let scripts = ([build.nu control.nu prepare-sources.nu prepare-registry.nu] | each {|p| {path: $p, sha256: (sha ($out | path join $p))}})
    {inherited: $inherited, clean_environment: $run_env, executable_files: $tool_files,
        observations: $observations, sdk_path: $sdk, scripts: $scripts,
        sources_sha256: (sha ($out | path join sources.json)), registry_sha256: (sha ($out | path join registry.json)),
        env_policy: 'env -i clears all inherited variables; the exact recorded allowlist is applied per cell'}
        | to json | save ($out | path join tools.json)
    mut builds = []
    for cell in [A B C D P] {
        let run_env = (environment $cache $cell)
        let cwd = ($cache | path join cells $cell nox)
        let command = (bench-command $cache $cell true)
        print $'building ($cell) once without measuring'
        let receipt = (capture $command $run_env $cwd $out $'build-($cell)')
        require ($receipt.exit_code == 0) $'build prerequisite red: ($cell); see retained raw streams'
        let messages = (open --raw ($out | path join $'build-($cell).stdout') | lines | each {|l| $l | from json})
        let binaries = ($messages | where {|r| $r.reason == 'compiler-artifact' and $r.target.name == reduction and $r.executable? != null} | get executable)
        require (($binaries | length) == 1) 'one reduction benchmark executable'
        let binary = $binaries.0
        let tree_command = (cargo-command $cache $cell [tree --manifest-path ($cwd | path join Cargo.toml)
            -p cyber-nox --locked --offline -e features -i cyber-hemera])
        let tree = (capture $tree_command $run_env $cwd $out $'features-($cell)')
        require ($tree.exit_code == 0) 'feature tree prerequisite'
        let metadata_command = (cargo-command $cache $cell [metadata --manifest-path ($cwd | path join Cargo.toml)
            --format-version '1' --locked --offline])
        let metadata = (capture $metadata_command $run_env $cwd $out $'metadata-($cell)')
        require ($metadata.exit_code == 0) 'metadata prerequisite'
        $builds = ($builds | append {cell: $cell, binary: $binary, sha256: (sha $binary),
            bytes: (open --raw $binary | into binary | bytes length), receipt: $receipt,
            features: $tree, metadata: $metadata})
    }
    check-sources $sources
    check-registry $registry
    for t in $tool_files { require ((sha $t.path) == $t.sha256) 'tool drift during builds' }
    {builds: $builds, sources_equal: true, registry_equal: true, tools_equal: true,
        finished: (date now | format date '%+'), scripts: $scripts} | to json | save ($out | path join builds.json)
    print 'all five build-only cells frozen; no benchmark has executed'
}
