# Synkkaa missa-olen-daemon suljetusta reposta Kotisatama-buildiin.
#
# Käyttö (PowerShell, Kotisatama-repon juuressa):
#   .\scripts\sync-missa-olen-daemon.ps1
#   $env:KOTISATAMA_MISSA_OLEN_BIN = ".\bin\missa-olen-daemon.exe"

param(
    [string]$ClosedRepoRoot = "",
    [string]$OutputDir = (Join-Path $PSScriptRoot "..\bin")
)

$ErrorActionPreference = "Stop"

function Get-ClosedRepoRoot {
    param([string]$RepoRoot)
    if ($ClosedRepoRoot) {
        return (Resolve-Path -LiteralPath $ClosedRepoRoot).Path
    }
    $candidates = @(
        (Join-Path $RepoRoot "..\Varustamo"),
        (Join-Path $RepoRoot "..\Kotisataman-suljetut-osat")
    )
    foreach ($candidate in $candidates) {
        $daemon = Join-Path $candidate "Missa-olen\daemon\Cargo.toml"
        if (Test-Path $daemon) {
            return (Resolve-Path -LiteralPath $candidate).Path
        }
    }
    return $candidates[1]
}

function Get-FileSha256 {
    param([string]$Path)
    if (-not (Test-Path $Path)) { return $null }
    return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash
}

function Stop-MissaOlenDaemonIfRunning {
    $procs = Get-Process -Name "missa-olen-daemon" -ErrorAction SilentlyContinue
    foreach ($proc in $procs) {
        Write-Host "Stopping missa-olen-daemon (PID $($proc.Id))..."
        Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
    }
    if ($procs) {
        Start-Sleep -Milliseconds 500
    }
}

function Install-BuiltDaemon {
    param(
        [string]$Source,
        [string]$Dest
    )
    if ((Test-Path $Dest) -and ((Get-FileSha256 $Source) -eq (Get-FileSha256 $Dest))) {
        Write-Host "missa-olen-daemon.exe already up to date: $Dest"
        return
    }

    $lastError = $null
    for ($attempt = 0; $attempt -lt 3; $attempt++) {
        try {
            Copy-Item -LiteralPath $Source -Destination $Dest -Force
            Write-Host "Synced -> $Dest"
            return
        } catch {
            $lastError = $_
            if ($attempt -eq 0) {
                Write-Host "missa-olen-daemon.exe is in use; stopping running instance..."
                Stop-MissaOlenDaemonIfRunning
            } else {
                Start-Sleep -Milliseconds 500
            }
        }
    }

    $staging = "$Dest.new"
    Copy-Item -LiteralPath $Source -Destination $staging -Force
    Write-Warning @"
Could not replace locked file: $Dest
New binary saved as: $staging
Stop Kotisatama / missa-olen-daemon and re-run sync, or rename .new -> .exe manually.
"@
    if ($lastError) {
        Write-Warning $lastError.Exception.Message
    }
}

$repoRoot = Split-Path $PSScriptRoot -Parent
$closed = Get-ClosedRepoRoot -RepoRoot $repoRoot
$daemonDir = Join-Path $closed "Missa-olen\daemon"

if (-not (Test-Path $daemonDir)) {
    Write-Error "Missä olen daemon not found: $daemonDir"
}

New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

Push-Location $daemonDir
$previousCargoTargetDir = $env:CARGO_TARGET_DIR
try {
    $localTarget = Join-Path $daemonDir "target"
    $env:CARGO_TARGET_DIR = $localTarget
    cargo build --release
    $built = Join-Path $localTarget "release\missa-olen-daemon.exe"
    if (-not (Test-Path $built)) {
        $built = Join-Path $localTarget "release\missa-olen-daemon"
    }
    Install-BuiltDaemon -Source $built -Dest (Join-Path $OutputDir "missa-olen-daemon.exe")
} finally {
    if ($null -eq $previousCargoTargetDir) {
        Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
    } else {
        $env:CARGO_TARGET_DIR = $previousCargoTargetDir
    }
    Pop-Location
}
