# Rakenna Kotisatama Win11 -testiversio paikallisesti.
#
# Vastaa .github/workflows/windows-desktop-build.yml -polkua, mutta nopeampi iterointi:
#   - bootstrap vain kerran (tai -SkipBootstrap)
#   - valmis dist/ + zip + run-test.ps1
#
# Käyttö (PowerShell, repon juuressa):
#   .\scripts\build-win11-test.ps1
#   .\scripts\build-win11-test.ps1 -SkipBootstrap          # toinen build samalla koneella
#   .\scripts\build-win11-test.ps1 -SkipBootstrap -Run     # build + käynnistä
#   .\scripts\build-win11-test.ps1 -CargoTargetDir C:\kt\target
#
# Ensimmainen ajo (mach bootstrap + release build) voi kestaa 1-2 h. Seuraavat ~30-60 min.

param(
    [switch]$SkipBootstrap,
    [switch]$SkipTests,
    [switch]$SkipMeilisearch,
    [switch]$SkipPulloposti,
    [switch]$NoPackage,
    [switch]$Run,
    [string]$CargoTargetDir = "",
    [string]$VsInstallPath = "",
    [string]$MeilisearchVersion = "1.12.8"
)

$ErrorActionPreference = "Stop"

function Write-Step([string]$Message) {
    Write-Host ""
    Write-Host "==> $Message" -ForegroundColor Cyan
}

function Find-VsInstallPath {
    if ($VsInstallPath -and (Test-Path (Join-Path $VsInstallPath "Common7\Tools\Microsoft.VisualStudio.DevShell.dll"))) {
        return $VsInstallPath
    }
    foreach ($candidate in @(
            "C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools",
            "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools",
            "C:\Program Files\Microsoft Visual Studio\2022\Community",
            "C:\Program Files\Microsoft Visual Studio\2022\Professional"
        )) {
        if (Test-Path (Join-Path $candidate "Common7\Tools\Microsoft.VisualStudio.DevShell.dll")) {
            return $candidate
        }
    }
    return $null
}

function Enter-BuildEnvironment {
    $vs = Find-VsInstallPath
    if (-not $vs) {
        throw "Visual Studio Build Tools not found. Install Build Tools with C++ workload or pass -VsInstallPath."
    }

    Import-Module (Join-Path $vs "Common7\Tools\Microsoft.VisualStudio.DevShell.dll")
    Enter-VsDevShell -VsInstallPath $vs -SkipAutomaticLocation -DevCmdArguments "-arch=amd64" | Out-Null

    $llvm = "C:\Program Files\LLVM\bin"
    if (Test-Path $llvm) {
        $env:PATH = "$llvm;$env:PATH"
    }
}

function Ensure-Uv {
    if (Get-Command uv -ErrorAction SilentlyContinue) { return }
    Write-Step "Installing uv (Python runner for mach)"
    python -m pip install --upgrade pip
    python -m pip install uv
}

function Invoke-KotisatamaTests {
    Write-Step "Kotisatama unit tests"
    cargo test -p kotisatama-pulloposti -p kotisatama-whitelist -p kotisatama-search -p kotisatama-report
}

function Ensure-Meilisearch {
    param([string]$BinDir)
    $dest = Join-Path $BinDir "meilisearch.exe"
    if (Test-Path $dest) {
        Write-Host "Meilisearch already present: $dest"
        return
    }

    New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
    $asset = "meilisearch-windows-amd64.exe"
    $url = "https://github.com/meilisearch/meilisearch/releases/download/v$MeilisearchVersion/$asset"
    Write-Step "Downloading Meilisearch v$MeilisearchVersion"
    Invoke-WebRequest -Uri $url -OutFile $dest
    Write-Host "Installed: $dest"
}

function Sync-PullopostiDaemon {
    param([string]$RepoRoot)
    $closed = Join-Path $RepoRoot "..\Kotisataman-suljetut-osat"
    if (-not (Test-Path (Join-Path $closed "Pulloposti\daemon\Cargo.toml"))) {
        Write-Warning "Suljettu repo ei loydy - Pulloposti daemon ohitetaan."
        return
    }
    & (Join-Path $RepoRoot "scripts\sync-pulloposti-daemon.ps1") -ClosedRepoRoot $closed
}

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

    # Kopioi kaikki runtime-tiedostot (DLL:t, plugin-DLL:t) release-kansiosta.
    Get-ChildItem -Path $ReleaseDir -File | Where-Object {
        $_.Extension -in @(".exe", ".dll", ".json", ".pak") -or $_.Name -match "^(servoshell|servo)$"
    } | ForEach-Object {
        Copy-Item -Force $_.FullName (Join-Path $DistDir $_.Name)
    }

    $bin = Find-ServoshellBinary -ReleaseDir $ReleaseDir
    if (-not $bin) {
        Write-Host "Release directory contents:"
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
    if (-not (Test-Path $whitelist)) {
        Copy-Item (Join-Path $configDst "whitelist.example.json") $whitelist
    }
}

function New-TestPackage {
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

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $RepoRoot

if (-not (Test-Path (Join-Path $RepoRoot "mach"))) {
    throw "Run from Kotisatama repo (mach not found)."
}

$sw = [System.Diagnostics.Stopwatch]::StartNew()
Enter-BuildEnvironment

$ServoTargetDir = $null
if ($CargoTargetDir) {
    $resolved = Resolve-Path -LiteralPath $CargoTargetDir -ErrorAction SilentlyContinue
    if (-not $resolved) {
        New-Item -ItemType Directory -Force -Path $CargoTargetDir | Out-Null
        $resolved = Resolve-Path $CargoTargetDir
    }
    $ServoTargetDir = $resolved.Path
    $env:CARGO_TARGET_DIR = $ServoTargetDir
    Write-Host "CARGO_TARGET_DIR=$ServoTargetDir"
}

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

$BinDir = Join-Path $RepoRoot "bin"
if (-not $SkipMeilisearch) {
    Ensure-Meilisearch -BinDir $BinDir
}

if (-not $SkipPulloposti) {
    Write-Step "Pulloposti daemon (valinnainen)"
    Sync-PullopostiDaemon -RepoRoot $RepoRoot
}

Write-Step "mach build --release (tama kestaa kauimmin)"
& .\mach build --release

$releaseDir = Get-ServoshellReleaseDir -RepoRoot $RepoRoot -ServoTargetDir $ServoTargetDir
Write-Host "Using release dir: $releaseDir"

$distDir = Join-Path $RepoRoot "dist"

if (-not $NoPackage) {
    New-TestPackage -RepoRoot $RepoRoot -ReleaseDir $releaseDir -DistDir $distDir -BinDir $BinDir
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
