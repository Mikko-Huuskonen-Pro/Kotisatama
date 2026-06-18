# Synkkaa pulloposti-daemon suljetusta reposta Kotisatama-buildiin.
#
# Käyttö (PowerShell, Kotisatama-repon juuressa):
#   .\scripts\sync-pulloposti-daemon.ps1
#   $env:KOTISATAMA_PULLOPOSTI_BIN = ".\bin\pulloposti-daemon.exe"

param(
    [string]$ClosedRepoRoot = (Join-Path (Split-Path $PSScriptRoot -Parent) "..\Kotisataman-suljetut-osat"),
    [string]$OutputDir = (Join-Path $PSScriptRoot "..\bin")
)

$ErrorActionPreference = "Stop"
$daemonDir = Join-Path $ClosedRepoRoot "Pulloposti\daemon"

if (-not (Test-Path $daemonDir)) {
    Write-Error "Pulloposti daemon not found: $daemonDir"
}

New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

Push-Location $daemonDir
$previousCargoTargetDir = $env:CARGO_TARGET_DIR
try {
    $localTarget = Join-Path $daemonDir "target"
    $env:CARGO_TARGET_DIR = $localTarget
    cargo build --release
    $built = Join-Path $localTarget "release\pulloposti-daemon.exe"
    if (-not (Test-Path $built)) {
        $built = Join-Path $localTarget "release\pulloposti-daemon"
    }
    Copy-Item $built (Join-Path $OutputDir "pulloposti-daemon.exe") -Force
    Write-Host "Synced -> $(Join-Path $OutputDir 'pulloposti-daemon.exe')"
} finally {
    if ($null -eq $previousCargoTargetDir) {
        Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
    } else {
        $env:CARGO_TARGET_DIR = $previousCargoTargetDir
    }
    Pop-Location
}
