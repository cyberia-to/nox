use support.nu *

# Retain the exact compared artifacts and both package bodies in one small capsule.
def main [repo: string, pair: string, api: string, cache: string] {
    let root = ($repo | path expand)
    let audit = ($root | path join audit/2026-10-17-published-api-baseline)
    let inputs = (open ($audit | path join $pair identities.json))
    let result = (open ($audit | path join $api result.json))
    let cache = ($cache | path expand)
    let bundle = ($audit | path join evidence.tar.gz)
    require (not ($cache | path exists) and not ($bundle | path exists)) 'retention destination exists'
    mkdir $cache
    mut files = []
    for artifact in $result.artifacts {
        require ((digest $artifact.path) == $artifact.sha256) 'rustdoc artifact changed'
        let name = $'($artifact.name).json'
        cp $artifact.path ($cache | path join $name)
        $files = ($files | append {path: $name, sha256: $artifact.sha256})
    }
    for side in [baseline current] {
        let input = ($inputs | get $side)
        require ((digest $input.archive) == $input.archive_sha256) 'package archive changed'
        let name = $'($side).crate'
        cp $input.archive ($cache | path join $name)
        $files = ($files | append {path: $name, sha256: $input.archive_sha256})
    }
    capture $audit retain [tar -czf $bundle -C $cache ...($files | get path)] $root {} | ignore
    {pair: $pair, api: $api, files: $files, sha256: (digest $bundle)}
    | to json | save ($audit | path join bundle.json)
}
