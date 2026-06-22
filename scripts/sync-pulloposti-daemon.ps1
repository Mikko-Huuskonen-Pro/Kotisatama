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

function Get-FileSha256 {
    param([string]$Path)
    if (-not (Test-Path $Path)) { return $null }
    return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash
}

function Stop-PullopostiDaemonIfRunning {
    $procs = Get-Process -Name "pulloposti-daemon" -ErrorAction SilentlyContinue
    foreach ($proc in $procs) {
        Write-Host "Stopping pulloposti-daemon (PID $($proc.Id))..."
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
        Write-Host "pulloposti-daemon.exe already up to date: $Dest"
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
                Write-Host "pulloposti-daemon.exe is in use; stopping running instance..."
                Stop-PullopostiDaemonIfRunning
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
Stop Kotisatama / pulloposti-daemon and re-run sync, or rename .new -> .exe manually.
"@
    if ($lastError) {
        Write-Warning $lastError.Exception.Message
    }
}

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
    Install-BuiltDaemon -Source $built -Dest (Join-Path $OutputDir "pulloposti-daemon.exe")
} finally {
    if ($null -eq $previousCargoTargetDir) {
        Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
    } else {
        $env:CARGO_TARGET_DIR = $previousCargoTargetDir
    }
    Pop-Location
}
