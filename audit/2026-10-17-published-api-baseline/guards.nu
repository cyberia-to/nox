use support.nu *

# Tiny isolated Git fixture exercises the committed-input guard; no Cargo build.
def main [repo: string, cache: string] {
    let root = ($repo | path expand)
    let cache = ($cache | path expand)
    require (not ($cache | path exists)) 'guard fixture already exists'
    mkdir ($cache | path join rs)
    '[workspace]' | save ($cache | path join Cargo.toml)
    'version = 4' | save ($cache | path join Cargo.lock)
    '[package]' | save ($cache | path join rs Cargo.toml)
    'pub fn marker() {}' | save ($cache | path join rs lib.rs)
    '/.cargo/' | save ($cache | path join .gitignore)
    let audit = ($root | path join audit/2026-10-17-published-api-baseline)
    let out = ($audit | path join guards-1)
    require (not ($out | path exists)) 'guard receipts already exist'
    mkdir $out
    capture $out init [git init -q $cache] $root {} | ignore
    capture $out stage [git -C $cache add Cargo.toml Cargo.lock rs/Cargo.toml rs/lib.rs .gitignore] $root {} | ignore
    capture $out commit [git -C $cache -c user.name=Audit -c user.email=audit@example.invalid commit -q -m 'test: committed input guard fixture'] $root {} | ignore
    let original = (committed-owner $cache)
    mut results = [{case: clean, accepted: true}]
    mkdir ($cache | path join audit)
    'diagnostic-only' | save ($cache | path join audit source.rs)
    require ((committed-owner $cache) == $original) 'audit-only file unexpectedly changes compiler inventory'
    $results = ($results | append {case: audit-only, accepted: true})
    for path in [Cargo.toml Cargo.lock rs/Cargo.toml rs/lib.rs] {
        let target = ($cache | path join $path)
        let old = (open --raw $target)
        $old + "\n# altered\n" | save -f $target
        let refusal = (try { committed-owner $cache | ignore; '' } catch {|e| $e.msg })
        require ($refusal | str contains 'differs from HEAD') $'dirty input was accepted: ($path)'
        $results = ($results | append {case: $path, accepted: false, error: $refusal})
        $old | save -f $target
    }
    mkdir ($cache | path join .cargo)
    '[build]' | save ($cache | path join .cargo config.toml)
    let refusal = (try { committed-owner $cache | ignore; '' } catch {|e| $e.msg })
    require ($refusal | str contains 'inventory is not committed') 'ignored Cargo config was accepted'
    $results = ($results | append {case: ignored-config, accepted: false, error: $refusal})
    # Keep the negative fixture as found; final source identities bind the tool.
    {source: ([support.nu guards.nu] | each {|p| {path: $p, sha256: (digest ($audit | path join $p))}}),
        cases: $results} | to json | save ($out | path join result.json)
    $results | to json
}
