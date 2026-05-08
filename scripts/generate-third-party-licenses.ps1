param(
    [string]$Config = "about.toml",
    [string]$Output = "THIRD_PARTY_LICENSES.txt"
)

$ErrorActionPreference = "Stop"

$tempJson = Join-Path "target" "cargo-about.json"

cargo about generate --format json --workspace -c $Config -o $tempJson

$about = Get-Content $tempJson -Raw | ConvertFrom-Json

$crates =
    $about.licenses |
    ForEach-Object { $_.used_by } |
    ForEach-Object { $_.crate } |
    Group-Object id |
    ForEach-Object { $_.Group[0] } |
    Sort-Object name, version

$lines = @(
    "Third-Party Crates and Licenses for NeoSTS"
    "=========================================="
    ""
    "Format: crate | version | license | url"
    ""
)

foreach ($crate in $crates) {
    $url = if ($crate.repository) {
        $crate.repository
    } elseif ($crate.homepage) {
        $crate.homepage
    } elseif ($crate.documentation) {
        $crate.documentation
    } else {
        ""
    }

    $lines += "$($crate.name) | $($crate.version) | $($crate.license) | $url"
}

Set-Content -Path $Output -Value $lines -Encoding UTF8
