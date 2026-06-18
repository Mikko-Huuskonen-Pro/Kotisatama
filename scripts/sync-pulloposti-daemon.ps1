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
try {
    $env:CARGO_TARGET_DIR = Join-Path $daemonDir "target"
    cargo build --release
    $built = Join-Path $env:CARGO_TARGET_DIR "release\pulloposti-daemon.exe"
    if (-not (Test-Path $built)) {
        $built = Join-Path $env:CARGO_TARGET_DIR "release\pulloposti-daemon"
    }
    Copy-Item $built (Join-Path $OutputDir "pulloposti-daemon.exe") -Force
    Write-Host "Synced -> $(Join-Path $OutputDir 'pulloposti-daemon.exe')"
} finally {
    Pop-Location
}
