# Rakenna Kotisatama Windows 11 -desktop (servoshell).
#
# Kaytto (PowerShell, repon juuressa):
#   .\scripts\build-win11.ps1
#   .\scripts\build-win11.ps1 -SkipBootstrap -Run
#   .\scripts\build-win11.ps1 -CargoTargetDir C:\kt\target
#
# Ensimmainen ajo (bootstrap + release) voi kestaa 1-2 h. Seuraavat ~30-60 min.

param(
    [switch]$SkipBootstrap,
    [switch]$SkipTests,
    [switch]$SkipMeilisearch,
    [switch]$SkipPulloposti,
    [switch]$SkipWhitelistSync,
    [switch]$NoPackage,
    [switch]$Run,
    [string]$CargoTargetDir = "",
    [string]$VsInstallPath = "",
    [string]$MeilisearchVersion = "1.12.8"
)

$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "lib\build-common.ps1")

function Find-ServoshellBinary {
    param([string]$ReleaseDir)
    foreach ($name in @("servoshell.exe", "servo.exe", "servoshell", "servo")) {
        $path = Join-Path $ReleaseDir $name
        if (Test-Path $path) { return $path }
    }
    return $null
}

function Get-ServoshellReleaseDir {
    param(
        [string]$RepoRoot,
        [string]$ServoTargetDir
    )
    $candidates = @()
    if ($ServoTargetDir) {
        $candidates += (Join-Path $ServoTargetDir "release")
    }
    $candidates += (Join-Path $RepoRoot "target\release")
    if ($env:CARGO_TARGET_DIR) {
        $candidates += (Join-Path $env:CARGO_TARGET_DIR "release")
    }
    foreach ($dir in ($candidates | Select-Object -Unique)) {
        if (Find-ServoshellBinary -ReleaseDir $dir) {
            return (Resolve-Path $dir).Path
        }
    }
    throw "servoshell.exe not found. Checked: $($candidates -join ', ')"
}

function Copy-ReleaseArtifacts {
    param(
        [string]$ReleaseDir,
        [string]$DistDir
    )
    New-Item -ItemType Directory -Force -Path $DistDir | Out-Null
    Get-ChildItem -Path $ReleaseDir -File | Where-Object {
        $_.Extension -in @(".exe", ".dll", ".json", ".pak") -or $_.Name -match "^(servoshell|servo)$"
    } | ForEach-Object {
        Copy-Item -Force $_.FullName (Join-Path $DistDir $_.Name)
    }
    if (-not (Find-ServoshellBinary -ReleaseDir $ReleaseDir)) {
        Get-ChildItem $ReleaseDir | Format-Table -AutoSize
        throw "servoshell.exe not found under $ReleaseDir"
    }
}

function Write-RunScript {
    param([string]$DistDir)
    $runPath = Join-Path $DistDir "run-test.ps1"
    $content = @'
# Kaynnista Kotisatama testiversio (Win11).
# Kaytto: .\run-test.ps1
#         .\run-test.ps1 https://example.com

$Root = $PSScriptRoot
$env:PATH = (Join-Path $Root "bin") + ";" + $env:PATH

$logDir = Join-Path $Root "logs"
New-Item -ItemType Directory -Force -Path $logDir | Out-Null
$logFile = Join-Path $logDir ("kotisatama-{0:yyyyMMdd-HHmmss}.log" -f (Get-Date))

if (-not $env:RUST_LOG) { $env:RUST_LOG = "info,servoshell=debug,kotisatama=debug" }
if (-not $env:RUST_BACKTRACE) { $env:RUST_BACKTRACE = "1" }

Write-Host "Lokit: $logFile"
Write-Host "RUST_LOG=$($env:RUST_LOG)"

if (-not $env:KOTISATAMA_WHITELIST_PATH) {
    $wl = Join-Path $Root "config\whitelist.json"
    if (Test-Path $wl) { $env:KOTISATAMA_WHITELIST_PATH = $wl }
}

if (-not $env:KOTISATAMA_MEILISEARCH_BIN) {
    $ms = Join-Path $Root "bin\meilisearch.exe"
    if (Test-Path $ms) { $env:KOTISATAMA_MEILISEARCH_BIN = $ms }
}

if (-not $env:KOTISATAMA_PULLOPOSTI_BIN) {
    $pp = Join-Path $Root "bin\pulloposti-daemon.exe"
    if (Test-Path $pp) { $env:KOTISATAMA_PULLOPOSTI_BIN = $pp }
}

if (-not $env:KOTISATAMA_SEARCH_DOCUMENTS) {
    $docs = Join-Path $Root "config\search-index\documents.json"
    if (Test-Path $docs) { $env:KOTISATAMA_SEARCH_DOCUMENTS = $docs }
}

$env:KOTISATAMA_DATA_DIR = Join-Path $Root "index-data"
New-Item -ItemType Directory -Force -Path $env:KOTISATAMA_DATA_DIR | Out-Null

$exe = Join-Path $Root "servoshell.exe"
if (-not (Test-Path $exe)) { throw "servoshell.exe missing in $Root" }

& $exe @args 2>&1 | Tee-Object -FilePath $logFile
'@
    Set-Content -Path $runPath -Value $content -Encoding UTF8
}

function Copy-ConfigTree {
    param(
        [string]$RepoRoot,
        [string]$DistDir
    )
    $configSrc = Join-Path $RepoRoot "config"
    $configDst = Join-Path $DistDir "config"
    Copy-Item -Recurse -Force $configSrc $configDst
    $whitelist = Join-Path $configDst "whitelist.json"
    if ($env:KOTISATAMA_WHITELIST_PATH -and (Test-Path $env:KOTISATAMA_WHITELIST_PATH)) {
        Copy-Item -Force $env:KOTISATAMA_WHITELIST_PATH $whitelist
    } elseif (-not (Test-Path $whitelist)) {
        Copy-Item (Join-Path $configDst "whitelist.example.json") $whitelist
    }
}

function New-DesktopPackage {
    param(
        [string]$RepoRoot,
        [string]$ReleaseDir,
        [string]$DistDir,
        [string]$BinDir
    )
    Write-Step "Packaging dist/"
    if (Test-Path $DistDir) { Remove-Item -Recurse -Force $DistDir }
    Copy-ReleaseArtifacts -ReleaseDir $ReleaseDir -DistDir $DistDir
    Copy-Item -Recurse -Force (Join-Path $RepoRoot "resources") (Join-Path $DistDir "resources")
    Copy-ConfigTree -RepoRoot $RepoRoot -DistDir $DistDir
    if (Test-Path $BinDir) {
        Copy-Item -Recurse -Force $BinDir (Join-Path $DistDir "bin")
    }
    Write-RunScript -DistDir $DistDir
    $sha = (git -C $RepoRoot rev-parse --short HEAD).Trim()
    $zip = Join-Path $RepoRoot "kotisatama-win11-$sha.zip"
    if (Test-Path $zip) { Remove-Item -Force $zip }
    Compress-Archive -Path (Join-Path $DistDir "*") -DestinationPath $zip -Force
    Write-Host ""
    Write-Host "Package ready:" -ForegroundColor Green
    Write-Host "  Folder: $DistDir"
    Write-Host "  Zip:    $zip"
    Write-Host "  Run:    $DistDir\run-test.ps1"
}

# --- main ---

$RepoRoot = Get-KotisatamaRepoRoot -ScriptRoot $PSScriptRoot
Set-Location $RepoRoot
if (-not (Test-Path (Join-Path $RepoRoot "mach"))) {
    throw "Run from Kotisatama repo (mach not found)."
}

$sw = [System.Diagnostics.Stopwatch]::StartNew()
Enter-BuildEnvironment -VsInstallPath $VsInstallPath
$ServoTargetDir = Set-CargoTargetDir -CargoTargetDir $CargoTargetDir
Ensure-Uv

if (-not $SkipBootstrap) {
    Write-Step "mach bootstrap (GStreamer, LLVM - kerran riittaa)"
    & .\mach bootstrap --yes
} else {
    Write-Host "Skipping bootstrap (-SkipBootstrap)"
}

if (-not $SkipTests) {
    Invoke-KotisatamaTests
}

if (-not $SkipWhitelistSync) {
    Write-Step "Whitelist (suljettu repo)"
    Sync-Whitelist -RepoRoot $RepoRoot
}

$BinDir = Join-Path $RepoRoot "bin"
if (-not $SkipMeilisearch) {
    Ensure-MeilisearchDesktop -BinDir $BinDir -MeilisearchVersion $MeilisearchVersion
}

if (-not $SkipPulloposti) {
    Write-Step "Pulloposti daemon (valinnainen)"
    Sync-PullopostiDaemon -RepoRoot $RepoRoot
}

Write-Step "mach build --release"
& .\mach build --release

$releaseDir = Get-ServoshellReleaseDir -RepoRoot $RepoRoot -ServoTargetDir $ServoTargetDir
Write-Host "Using release dir: $releaseDir"

$distDir = Join-Path $RepoRoot "dist"
if (-not $NoPackage) {
    New-DesktopPackage -RepoRoot $RepoRoot -ReleaseDir $releaseDir -DistDir $distDir -BinDir $BinDir
}

$sw.Stop()
Write-Host ""
Write-Host ("Valmis ajassa {0:g}." -f $sw.Elapsed) -ForegroundColor Green

if ($Run) {
    if ($NoPackage) {
        Write-Step "mach run"
        & .\mach run
    } else {
        Write-Step "run-test.ps1"
        & (Join-Path $distDir "run-test.ps1")
    }
}
