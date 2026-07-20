$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$devPort = 1420

function Get-DevPortListeners {
    return @(
        Get-NetTCPConnection `
            -LocalPort $devPort `
            -State Listen `
            -ErrorAction SilentlyContinue
    )
}

$processes = @(Get-Process overlay-forge -ErrorAction SilentlyContinue)
if ($processes.Count -eq 0) {
    Write-Host "No existing overlay-forge process is running."
} else {
    foreach ($process in $processes) {
        Write-Host "Stopping overlay-forge PID $($process.Id)..."
        Stop-Process -Id $process.Id -Force
    }
}

$deadline = (Get-Date).AddSeconds(5)
do {
    Start-Sleep -Milliseconds 150
    $remaining = @(Get-Process overlay-forge -ErrorAction SilentlyContinue)
} while ($remaining.Count -gt 0 -and (Get-Date) -lt $deadline)

if ($remaining.Count -gt 0) {
    throw "overlay-forge process did not stop cleanly."
}

if ($processes.Count -gt 0) {
    Write-Host "overlay-forge stopped."
}

# Stopping the desktop process causes Tauri to shut down Vite asynchronously.
# Give it a moment to release the port before treating the listener as stale.
$deadline = (Get-Date).AddSeconds(5)
do {
    $listeners = Get-DevPortListeners
    if ($listeners.Count -eq 0) {
        exit 0
    }
    Start-Sleep -Milliseconds 150
} while ((Get-Date) -lt $deadline)

foreach ($listener in $listeners) {
    $owner = Get-CimInstance `
        -ClassName Win32_Process `
        -Filter "ProcessId = $($listener.OwningProcess)" `
        -ErrorAction SilentlyContinue

    $isOverlayForgeVite = (
        $owner `
        -and $owner.Name -eq "node.exe" `
        -and $owner.CommandLine -like "*$repoRoot*" `
        -and $owner.CommandLine -match "vite"
    )

    if (-not $isOverlayForgeVite) {
        $ownerDescription = if ($owner) {
            "$($owner.Name) (PID $($owner.ProcessId))"
        } else {
            "PID $($listener.OwningProcess)"
        }
        throw "Port $devPort is in use by $ownerDescription, which is not an Overlay Forge Vite process."
    }

    Write-Host "Stopping stale Overlay Forge Vite server PID $($owner.ProcessId)..."
    Stop-Process -Id $owner.ProcessId -Force
}

$deadline = (Get-Date).AddSeconds(5)
do {
    Start-Sleep -Milliseconds 150
    $listeners = Get-DevPortListeners
} while ($listeners.Count -gt 0 -and (Get-Date) -lt $deadline)

if ($listeners.Count -gt 0) {
    throw "Port $devPort was not released after stopping the stale Overlay Forge Vite server."
}

Write-Host "Overlay Forge development port $devPort is available."
