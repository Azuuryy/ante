$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
$target = Join-Path $repo 'target'
$output = Join-Path $target 'windows'
$binary = Join-Path $target 'release/ante.exe'
New-Item -ItemType Directory -Force -Path $output | Out-Null

foreach ($variant in @(@{ Features = ''; Name = 'ante.exe' }, @{ Features = 'dev-tools'; Name = 'ante-dev.exe' })) {
    $features = if ($variant.Features) { @('--features', $variant.Features) } else { @() }
    & cargo build --release --locked --manifest-path (Join-Path $repo 'Cargo.toml') -p ante-app --bin ante @features
    if ($LASTEXITCODE -ne 0) { throw "Cargo failed for $($variant.Name)" }
    Copy-Item -LiteralPath $binary -Destination (Join-Path $output $variant.Name) -Force
}
