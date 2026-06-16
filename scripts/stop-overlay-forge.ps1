$ErrorActionPreference = "Stop"

$processes = @(Get-Process overlay-forge -ErrorAction SilentlyContinue)
if ($processes.Count -eq 0) {
    Write-Host "No existing overlay-forge process is running."
    exit 0
}

foreach ($process in $processes) {
    Write-Host "Stopping overlay-forge PID $($process.Id)..."
    Stop-Process -Id $process.Id -Force
}

$deadline = (Get-Date).AddSeconds(5)
do {
    Start-Sleep -Milliseconds 150
    $remaining = @(Get-Process overlay-forge -ErrorAction SilentlyContinue)
} while ($remaining.Count -gt 0 -and (Get-Date) -lt $deadline)

if ($remaining.Count -gt 0) {
    throw "overlay-forge process did not stop cleanly."
}

Write-Host "overlay-forge stopped."
