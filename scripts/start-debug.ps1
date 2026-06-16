$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$tauriRoot = Join-Path $repoRoot "src-tauri"
$exePath = Join-Path $tauriRoot "target\debug\overlay-forge.exe"
$stopScript = Join-Path $PSScriptRoot "stop-overlay-forge.ps1"

& $stopScript

Push-Location $tauriRoot
try {
    cargo build
} finally {
    Pop-Location
}

if (-not (Test-Path $exePath)) {
    throw "Debug executable was not found: $exePath"
}

$process = Start-Process `
    -FilePath $exePath `
    -WorkingDirectory $tauriRoot `
    -WindowStyle Hidden `
    -PassThru

Write-Host "Started overlay-forge PID $($process.Id)."
