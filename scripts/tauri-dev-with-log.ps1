$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$logDir = Join-Path $repoRoot "logs\tauri-dev"
$timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$logPath = Join-Path $logDir "tauri-dev_$timestamp.log"
$latestPathFile = Join-Path $logDir "latest.txt"
$tauriCommand = Join-Path $repoRoot "node_modules\.bin\tauri.cmd"

if (-not (Test-Path $tauriCommand)) {
    $tauriCommand = "tauri"
}

New-Item -ItemType Directory -Force -Path $logDir | Out-Null
Set-Content -Path $latestPathFile -Value $logPath -Encoding UTF8

@(
    "Overlay Forge tauri dev log"
    "Started: $(Get-Date -Format o)"
    "Working directory: $repoRoot"
    "Command: $tauriCommand dev"
    ""
) | Set-Content -Path $logPath -Encoding UTF8

Write-Host "Overlay Forge tauri dev log: $logPath"

$cmd = "/d /s /c `"`"$tauriCommand`" dev >> `"$logPath`" 2>&1`""
$process = Start-Process `
    -FilePath "cmd.exe" `
    -ArgumentList $cmd `
    -WorkingDirectory $repoRoot `
    -NoNewWindow `
    -PassThru

$printedLineCount = 0

function Write-NewLogLines {
    if (-not (Test-Path $logPath)) {
        return
    }

    $lines = @(Get-Content -Path $logPath)
    if ($lines.Count -le $printedLineCount) {
        return
    }

    for ($index = $printedLineCount; $index -lt $lines.Count; $index++) {
        Write-Host $lines[$index]
    }
    $script:printedLineCount = $lines.Count
}

try {
    while (-not $process.HasExited) {
        Write-NewLogLines
        Start-Sleep -Milliseconds 500
    }
    Write-NewLogLines
    $exitCode = $process.ExitCode
} finally {
    if ($process -and -not $process.HasExited) {
        $process.Kill()
    }
    "" | Add-Content -Path $logPath
    "Finished: $(Get-Date -Format o)" | Add-Content -Path $logPath
    Write-Host "Overlay Forge tauri dev log saved: $logPath"
}

exit $exitCode
