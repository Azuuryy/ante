$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
$manifest = Join-Path $repo 'experiments/engine-evaluation/Cargo.toml'
$target = Join-Path $repo 'experiments/engine-evaluation/target'
$output = Join-Path $target 'windows'
$binary = Join-Path $target 'release/ante-mvp.exe'
New-Item -ItemType Directory -Force -Path $output | Out-Null

foreach ($variant in @(@{ Features = 'mvp'; Name = 'ante.exe' }, @{ Features = 'dev-tools'; Name = 'ante-dev.exe' })) {
    & cargo build --release --locked --manifest-path $manifest --target-dir $target --features $variant.Features --bin ante-mvp
    if ($LASTEXITCODE -ne 0) { throw "Cargo failed for $($variant.Name)" }
    Copy-Item -LiteralPath $binary -Destination (Join-Path $output $variant.Name) -Force
}
