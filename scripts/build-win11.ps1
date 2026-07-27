# Rakenna Kotisatama Windows 11 -desktop (servoshell).
#
# Kaytto (PowerShell, repon juuressa):
#   .\scripts\build-win11.ps1
#   .\scripts\build-win11.ps1 -SkipBootstrap -Run
#   .\scripts\build-win11.ps1 -CargoTargetDir C:\kt\target
#   .\scripts\build-win11.ps1 -MediaStack dummy   # vain hatatilanteessa (ei videotoistoa)
#   .\scripts\build-win11.ps1 -IncludeVarustamo  # Varustamo parkkeerattu oletuksena
#
# Oletus: --media-stack=gstreamer (Kanta/Radiant ym. HTML5-video).
# Ensimmainen ajo (bootstrap + release) voi kestaa 1-2 h. Seuraavat ~30-60 min.

param(
    [switch]$SkipBootstrap,
    [switch]$SkipTests,
    [switch]$SkipMeilisearch,
    [switch]$SkipPulloposti,
    [switch]$SkipWhitelistSync,
    # Varustamo parkkeerattu oletuksena; synkkaa vain tarvittaessa.
    [switch]$IncludeVarustamo,
    [switch]$NoPackage,
    [switch]$Run,
    [ValidateSet("gstreamer", "dummy")]
    [string]$MediaStack = "gstreamer",
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

Remove-Item Env:HTTP_PROXY,Env:HTTPS_PROXY,Env:ALL_PROXY,Env:http_proxy,Env:https_proxy,Env:all_proxy -ErrorAction SilentlyContinue
$env:NO_PROXY = "*"
$env:no_proxy = "*"

Write-Host "Lokit: $logFile"
Write-Host "RUST_LOG=$($env:RUST_LOG)"

# Taysi lista: preferoi suurinta tiedostoa (stub ~3KB, cache ~600KB).
if (-not $env:KOTISATAMA_WHITELIST_PATH) {
    $RepoRootGuess = Split-Path $Root -Parent
    $candidates = @(
        (Join-Path $Root "config\whitelist.json"),
        (Join-Path $Root "index-data\cache\whitelist.json"),
        (Join-Path $RepoRootGuess "index-data\cache\whitelist.json")
    )
    $best = $null
    $bestSize = 0
    foreach ($wl in $candidates) {
        if (-not (Test-Path -LiteralPath $wl)) { continue }
        $len = (Get-Item -LiteralPath $wl).Length
        if ($len -gt $bestSize) {
            $bestSize = $len
            $best = $wl
        }
    }
    if ($best) { $env:KOTISATAMA_WHITELIST_PATH = $best }
}
if ($env:KOTISATAMA_WHITELIST_PATH) {
    Write-Host "Whitelist: $env:KOTISATAMA_WHITELIST_PATH"
} else {
    Write-Warning "Whitelist-polku puuttuu - selain voi kayttaa pienta stub-listaa."
}

if (-not $env:KOTISATAMA_MEILISEARCH_BIN) {
    $ms = Join-Path $Root "bin\meilisearch.exe"
    if (Test-Path $ms) { $env:KOTISATAMA_MEILISEARCH_BIN = $ms }
}

if (-not $env:KOTISATAMA_PULLOPOSTI_BIN) {
    $pp = Join-Path $Root "bin\pulloposti-daemon.exe"
    if (Test-Path $pp) { $env:KOTISATAMA_PULLOPOSTI_BIN = $pp }
}

if (-not $env:KOTISATAMA_MISSA_OLEN_BIN) {
    $mo = Join-Path $Root "bin\missa-olen-daemon.exe"
    if (Test-Path $mo) { $env:KOTISATAMA_MISSA_OLEN_BIN = $mo }
}

# Varustamo parkkeerattu: alä aseta registry-envia automaattisesti.
# Takaisin kayttoon: $env:KOTISATAMA_VARUSTAMO = "1" ennen kaynnistysta.
if ($env:KOTISATAMA_VARUSTAMO -in @("1","true","TRUE","yes","on")) {
    if (-not $env:KOTISATAMA_VARUSTAMO_REGISTRY) {
        $vr = Join-Path $Root "config\varustamo\registry.json"
        if (Test-Path $vr) { $env:KOTISATAMA_VARUSTAMO_REGISTRY = $vr }
    }
}

if (-not $env:KOTISATAMA_SEARCH_DOCUMENTS) {
    $docs = Join-Path $Root "config\search-index\documents.json"
    if (Test-Path $docs) { $env:KOTISATAMA_SEARCH_DOCUMENTS = $docs }
}

$env:KOTISATAMA_DATA_DIR = Join-Path $Root "index-data"
New-Item -ItemType Directory -Force -Path $env:KOTISATAMA_DATA_DIR | Out-Null
if (-not $env:KOTISATAMA_MEILISEARCH_DB) {
    $env:KOTISATAMA_MEILISEARCH_DB = Join-Path $env:KOTISATAMA_DATA_DIR "meilisearch"
}

$exe = Join-Path $Root "servoshell.exe"
if (-not (Test-Path $exe)) { throw "servoshell.exe missing in $Root" }

$quotedArgs = @($args | ForEach-Object { '"' + ($_ -replace '"', '\"') + '"' })
$cmdLine = '"' + $exe + '"'
if ($quotedArgs.Count -gt 0) { $cmdLine += " " + ($quotedArgs -join " ") }
$cmdLine += " 2>&1"
& cmd.exe /d /c $cmdLine | Tee-Object -FilePath $logFile
exit $LASTEXITCODE
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
    # Copy-Item -Recurse destiin joka ON JO olemassa nestaa config\config\.
    if (Test-Path -LiteralPath $configDst) {
        Remove-Item -LiteralPath $configDst -Recurse -Force -ErrorAction SilentlyContinue
    }
    if (-not (Test-Path -LiteralPath $configSrc)) {
        throw "config/ puuttuu: $configSrc"
    }
    Copy-Item -Recurse -Force $configSrc $configDst
    if (-not (Test-Path -LiteralPath $configDst)) {
        throw "config-kopiointi epaonnistui -> $configDst"
    }

    $whitelist = Join-Path $configDst "whitelist.json"
    $cache = Join-Path $RepoRoot "index-data\cache\whitelist.json"
    # Valitse suurin olemassa oleva lista (stub config/whitelist.json ~3KB, taysi cache ~600KB).
    $candidates = @()
    if ($env:KOTISATAMA_WHITELIST_PATH -and (Test-Path -LiteralPath $env:KOTISATAMA_WHITELIST_PATH)) {
        $candidates += $env:KOTISATAMA_WHITELIST_PATH
    }
    if (Test-Path -LiteralPath $cache) {
        $candidates += $cache
    }
    $full = $null
    $bestSize = 0
    foreach ($c in ($candidates | Select-Object -Unique)) {
        # Alä yrita kopioida tiedostoa itseensa.
        if ([string]::Equals((Resolve-Path -LiteralPath $c).Path, (Resolve-Path -LiteralPath $whitelist -ErrorAction SilentlyContinue).Path, [System.StringComparison]::OrdinalIgnoreCase)) {
            continue
        }
        $size = (Get-Item -LiteralPath $c).Length
        if ($size -gt $bestSize) {
            $bestSize = $size
            $full = $c
        }
    }

    if ($full) {
        Copy-Item -Force -LiteralPath $full $whitelist
        $env:KOTISATAMA_WHITELIST_PATH = $whitelist
        $bytes = (Get-Item -LiteralPath $whitelist).Length
        Write-Host ('Whitelist pakettiin: {0} size={1}' -f $whitelist, $bytes)
        if ($bytes -lt 50000) {
            Write-Warning ('Paketissa on pieni whitelist size={0}. Tarkista suljettu repo / index-data/cache.' -f $bytes)
        }
    } elseif (-not (Test-Path -LiteralPath $whitelist)) {
        $example = Join-Path $configDst "whitelist.example.json"
        if (Test-Path -LiteralPath $example) {
            Copy-Item -Force $example $whitelist
            Write-Warning "Taysi whitelist puuttuu - kaytossa example/stub."
        }
    } else {
        $bytes = (Get-Item -LiteralPath $whitelist).Length
        if ($bytes -lt 50000) {
            Write-Warning ('Paketissa on stub-whitelist size={0}. Synkaa suljettu repo tai index-data/cache.' -f $bytes)
        }
    }
}

function Test-GStreamerReady {
    param(
        [string]$RepoRoot,
        [string]$ServoTargetDir
    )
    # Sama polku kuin python/servo/platform/windows.py (mach bootstrap).
    # util.get_target_dir() = CARGO_TARGET_DIR tai <repo>/target.
    $roots = @()
    if ($ServoTargetDir) {
        $roots += (Join-Path $ServoTargetDir "dependencies\gstreamer\1.0\msvc_X86_64")
    }
    if ($env:CARGO_TARGET_DIR) {
        $roots += (Join-Path $env:CARGO_TARGET_DIR "dependencies\gstreamer\1.0\msvc_X86_64")
    }
    if ($RepoRoot) {
        $roots += (Join-Path $RepoRoot "target\dependencies\gstreamer\1.0\msvc_X86_64")
    }
    $roots += "C:\gstreamer\1.0\msvc_X86_64"
    if ($env:GSTREAMER_1_0_ROOT_MSVC_X86_64) {
        $roots = @($env:GSTREAMER_1_0_ROOT_MSVC_X86_64) + $roots
    }
    foreach ($root in ($roots | Where-Object { $_ } | Select-Object -Unique)) {
        if (
            (Test-Path (Join-Path $root "bin\ffi-7.dll")) -and
            (Test-Path (Join-Path $root "lib\pkgconfig\gobject-2.0.pc"))
        ) {
            return $root
        }
    }
    return $null
}

function Stop-ProcessesLockingPath {
    param([string]$Path)
    if (-not (Test-Path $Path)) { return }
    $root = (Resolve-Path -LiteralPath $Path).Path
    Get-CimInstance Win32_Process -ErrorAction SilentlyContinue |
        Where-Object {
            $_.ExecutablePath -and
            $_.ExecutablePath.StartsWith($root, [System.StringComparison]::OrdinalIgnoreCase)
        } |
        ForEach-Object {
            Write-Host "Suljetaan dist-lukko: $($_.Name) (PID $($_.ProcessId))"
            Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue
        }
}

function Remove-StaleDistSnapshots {
    param([string]$RepoRoot)
    Get-ChildItem -LiteralPath $RepoRoot -Directory -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -match '^dist-\d{8}-\d{6}$' } |
        ForEach-Object {
            try {
                Stop-ProcessesLockingPath -Path $_.FullName
                Remove-Item -LiteralPath $_.FullName -Recurse -Force -ErrorAction Stop
                Write-Host "Poistettu vanha snapshot: $($_.Name)"
            } catch {
                Write-Warning "Snapshotia ei voitu poistaa ($($_.Name)): $($_.Exception.Message)"
            }
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
    # Aina pakataan repo/dist - ei timestamp-fallbackeja (ne kerääntyvät).
    Remove-StaleDistSnapshots -RepoRoot $RepoRoot
    if (Test-Path $DistDir) {
        Stop-ProcessesLockingPath -Path $DistDir
        Start-Sleep -Milliseconds 400
        try {
            Remove-Item -LiteralPath $DistDir -Recurse -Force -ErrorAction Stop
        } catch {
            Write-Warning "Dist on lukittu - paivitetaan paikoilleen. Sulje run-test/servoshell jos binary ei paivity."
        }
    }
    Copy-ReleaseArtifacts -ReleaseDir $ReleaseDir -DistDir $DistDir
    $resourcesDst = Join-Path $DistDir "resources"
    if (Test-Path -LiteralPath $resourcesDst) {
        Remove-Item -LiteralPath $resourcesDst -Recurse -Force -ErrorAction SilentlyContinue
    }
    Copy-Item -Recurse -Force (Join-Path $RepoRoot "resources") $resourcesDst
    Copy-ConfigTree -RepoRoot $RepoRoot -DistDir $DistDir
    if (-not (Test-Path -LiteralPath (Join-Path $DistDir "config\whitelist.json"))) {
        throw "Packaging failed: dist/config/whitelist.json missing after Copy-ConfigTree"
    }
    if (Test-Path $BinDir) {
        $binDst = Join-Path $DistDir "bin"
        if (Test-Path -LiteralPath $binDst) {
            Remove-Item -LiteralPath $binDst -Recurse -Force -ErrorAction SilentlyContinue
        }
        Copy-Item -Recurse -Force $BinDir $binDst
    }
    Write-RunScript -DistDir $DistDir
    $sha = (git -C $RepoRoot rev-parse --short HEAD).Trim()
    $zip = Join-Path $RepoRoot "kotisatama-win11-$sha.zip"
    if (Test-Path $zip) { Remove-Item -Force $zip }
    try {
        Compress-Archive -Path (Join-Path $DistDir "*") -DestinationPath $zip -Force -ErrorAction Stop
    } catch {
        Write-Warning "Zip-paketin luonti ohitettiin: $($_.Exception.Message)"
        $zip = "(ei luotu)"
    }
    Write-Host ""
    Write-Host "Package ready:" -ForegroundColor Green
    Write-Host "  Folder: $DistDir"
    Write-Host "  Zip:    $zip"
    Write-Host "  Run:    $DistDir\run-test.ps1"
    return $DistDir
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

Write-Step "Varustamo registry (valinnainen)"
if ($IncludeVarustamo) {
    Sync-VarustamoRegistry -RepoRoot $RepoRoot
} else {
    Write-Host "Varustamo: ohitetaan (parkkeerattu; -IncludeVarustamo)"
}

Write-Step "Missä olen daemon (valinnainen)"
Sync-MissaOlenDaemon -RepoRoot $RepoRoot

if ($MediaStack -eq "gstreamer") {
    $gstRoot = Test-GStreamerReady -RepoRoot $RepoRoot -ServoTargetDir $ServoTargetDir
    if (-not $gstRoot) {
        throw @"
GStreamer puuttuu - HTML5-video (esim. Kanta / Radiant Media Player) ei toimi ilman sita.
Aja ilman -SkipBootstrap (tai: .\mach bootstrap --yes), sitten uudelleen.
Hatatilassa: -MediaStack dummy (ei videotoistoa).
"@
    }
    Write-Host "GStreamer: $gstRoot"
} else {
    Write-Warning "MediaStack=dummy: ei videotoistoa (Radiant/HTML5). Kayta vain jos GStreamer ei ole saatavilla."
}

Write-Step "mach build --release --media-stack=$MediaStack"
& .\mach build --release "--media-stack=$MediaStack"
if ($LASTEXITCODE -ne 0) {
    throw "mach build --release --media-stack=$MediaStack failed with exit code $LASTEXITCODE"
}

$releaseDir = Get-ServoshellReleaseDir -RepoRoot $RepoRoot -ServoTargetDir $ServoTargetDir
Write-Host "Using release dir: $releaseDir"

$distDir = Join-Path $RepoRoot "dist"
if (-not $NoPackage) {
    $distDir = New-DesktopPackage -RepoRoot $RepoRoot -ReleaseDir $releaseDir -DistDir $distDir -BinDir $BinDir
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
