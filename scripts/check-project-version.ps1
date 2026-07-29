[CmdletBinding()]
param(
    [switch]$RequireTag
)

$ErrorActionPreference = "Stop"

$repositoryRoot = Split-Path -Parent $PSScriptRoot

function Get-RequiredMatch {
    param(
        [Parameter(Mandatory = $true)]
        [string]$RelativePath,
        [Parameter(Mandatory = $true)]
        [string]$Pattern,
        [Parameter(Mandatory = $true)]
        [string]$Label
    )

    $absolutePath = Join-Path $repositoryRoot $RelativePath
    $content = Get-Content -LiteralPath $absolutePath -Raw
    $match = [regex]::Match($content, $Pattern, [System.Text.RegularExpressions.RegexOptions]::Multiline)
    if (-not $match.Success) {
        throw "Could not find $Label in $RelativePath."
    }

    return $match.Groups[1].Value
}

$packageJson = Get-Content -LiteralPath (Join-Path $repositoryRoot "package.json") -Raw | ConvertFrom-Json
$tauriConfig = Get-Content -LiteralPath (Join-Path $repositoryRoot "src-tauri/tauri.conf.json") -Raw | ConvertFrom-Json
$expectedVersion = [string]$packageJson.version

if ($expectedVersion -notmatch '^\d+\.\d+\.\d+$') {
    throw "package.json version '$expectedVersion' is not in MAJOR.MINOR.PATCH form."
}

$observedVersions = [ordered]@{
    "package-lock.json root" = Get-RequiredMatch -RelativePath "package-lock.json" -Pattern '^\{\s*\r?\n\s*"name"\s*:\s*"overlay-forge",\s*\r?\n\s*"version"\s*:\s*"([^"]+)"' -Label "root version"
    "package-lock.json package" = Get-RequiredMatch -RelativePath "package-lock.json" -Pattern '(?s)"packages"\s*:\s*\{\s*""\s*:\s*\{\s*"name"\s*:\s*"overlay-forge",\s*"version"\s*:\s*"([^"]+)"' -Label "empty-package version"
    "src-tauri/Cargo.toml" = Get-RequiredMatch -RelativePath "src-tauri/Cargo.toml" -Pattern '(?s)^\[package\].*?^version\s*=\s*"([^"]+)"' -Label "package version"
    "src-tauri/Cargo.lock" = Get-RequiredMatch -RelativePath "src-tauri/Cargo.lock" -Pattern '(?s)^\[\[package\]\]\s*\r?\nname\s*=\s*"overlay-forge"\s*\r?\nversion\s*=\s*"([^"]+)"' -Label "overlay-forge package version"
    "src-tauri/tauri.conf.json" = [string]$tauriConfig.version
    "README.md" = Get-RequiredMatch -RelativePath "README.md" -Pattern '^Current stable app release:\s*(\d+\.\d+\.\d+)\s*$' -Label "current stable release"
    "docs/PROJECT_OVERVIEW.md" = Get-RequiredMatch -RelativePath "docs/PROJECT_OVERVIEW.md" -Pattern '^Overlay Forge\s+(\d+\.\d+\.\d+)\s*$' -Label "current stable release"
}

$mismatches = @()
foreach ($entry in $observedVersions.GetEnumerator()) {
    if ($entry.Value -ne $expectedVersion) {
        $mismatches += "$($entry.Key) reports $($entry.Value)"
    }
}

$changelog = Get-Content -LiteralPath (Join-Path $repositoryRoot "CHANGELOG.md") -Raw
$escapedVersion = [regex]::Escape($expectedVersion)
if ($changelog -notmatch "(?m)^## Unreleased\r?\n(?:\r?\n)*## $escapedVersion - \d{4}-\d{2}-\d{2}\s*$") {
    $mismatches += "CHANGELOG.md does not list $expectedVersion immediately after an empty Unreleased section"
}

if ($RequireTag) {
    $tagObjectType = git -C $repositoryRoot cat-file -t "refs/tags/v$expectedVersion" 2>$null
    if ($LASTEXITCODE -ne 0 -or $tagObjectType -ne "tag") {
        $mismatches += "annotated Git tag v$expectedVersion does not exist"
    }
}

if ($mismatches.Count -gt 0) {
    throw "Version consistency check failed: $($mismatches -join '; ')."
}

Write-Output "Overlay Forge version $expectedVersion is consistent across project metadata and documentation."
if ($RequireTag) {
    Write-Output "Annotated version tag v$expectedVersion exists locally."
}
