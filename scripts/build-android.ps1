# Rakenna Kotisatama Android APK (aarch64, servoshell EGL).
#
# Kaytto (PowerShell, repon juuressa):
#   .\scripts\build-android.ps1
#   .\scripts\build-android.ps1 -SkipBootstrap
#   .\scripts\build-android.ps1 -SkipBootstrap -Install -Usb
#
# Tulos:
#   target\aarch64-linux-android\checked-release\servoapp.apk
#
# Vaatimukset:
#   - Visual Studio Build Tools (host-tyokalut)
#   - JDK 17+ (JAVA_HOME)
#   - Android SDK/NDK (mach bootstrap asentaa)
#   - adb (asennukseen, valinnainen)

param(
    [switch]$SkipBootstrap,
    [switch]$SkipTests,
    [switch]$SkipMeilisearch,
    [switch]$SkipWhitelistSync,
    [switch]$SkipWikiSync,
    [switch]$InstallNdk,
    [switch]$Install,
    [switch]$Usb,
    [switch]$Emulator,
    [string]$CargoTargetDir = "",
    [string]$VsInstallPath = "",
    [string]$Target = "aarch64-linux-android",
    [string]$Profile = "checked-release"
)

$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "lib\build-common.ps1")

function Test-AndroidPrerequisites {
    if (-not $env:JAVA_HOME) {
        $java = Get-Command java -ErrorAction SilentlyContinue
        if (-not $java) {
            Write-Warning "JAVA_HOME ei ole asetettu. Asenna JDK 17+ tai Android Studio."
        }
    }
    if ($Install -or $Usb -or $Emulator) {
        if (-not (Get-Command adb -ErrorAction SilentlyContinue)) {
            throw "adb ei loydy PATHissa. Asenna Android platform-tools tai kayta Android Studioa."
        }
    }
}

# --- main ---

$RepoRoot = Get-KotisatamaRepoRoot -ScriptRoot $PSScriptRoot
Set-Location $RepoRoot
if (-not (Test-Path (Join-Path $RepoRoot "mach"))) {
    throw "Run from Kotisatama repo (mach not found)."
}

$sw = [System.Diagnostics.Stopwatch]::StartNew()
Test-AndroidPrerequisites
Test-AndroidCrossBuildHost
Enter-BuildEnvironment -VsInstallPath $VsInstallPath
Ensure-AndroidSdk -RepoRoot $RepoRoot -InstallNdk:$InstallNdk
Set-CargoTargetDir -CargoTargetDir $CargoTargetDir | Out-Null
Ensure-Uv

if (-not $SkipBootstrap) {
    Write-Step "mach bootstrap (Android NDK/SDK, LLVM - kerran riittaa)"
    & .\mach bootstrap --yes
} else {
    Write-Host "Skipping bootstrap (-SkipBootstrap)"
}

if (-not $SkipTests) {
    Invoke-KotisatamaTests
}

if (-not $SkipWhitelistSync) {
    Write-Step "Whitelist APK-asseteihin"
    Sync-Whitelist -RepoRoot $RepoRoot
}

if (-not $SkipWikiSync) {
    $wikiSync = Join-Path $RepoRoot "scripts\sync-android-wiki-test-data.ps1"
    if (Test-Path $wikiSync) {
        Write-Step "Wiki-testidata APK:hen (Meilisearch dump + snapshots)"
        try {
            & $wikiSync
        } catch {
            Write-Warning "Wiki sync skipped: $($_.Exception.Message)"
            Write-Warning "Offline Wikipedia search will be limited until you run: .\scripts\sync-android-wiki-test-data.ps1"
        }
    }
}

$cachedWhitelist = Join-Path $RepoRoot "index-data\cache\whitelist.json"
$fallbackWhitelist = Join-Path $RepoRoot "config\whitelist.json"
if (Test-Path $cachedWhitelist) {
    $env:KOTISATAMA_WHITELIST_PATH = $cachedWhitelist
} elseif (-not (Test-Path $fallbackWhitelist)) {
    Write-Warning "Whitelistia ei loydy. Aja sync-whitelist.ps1 tai palauta config\whitelist.json."
}

if (-not $SkipMeilisearch) {
    Ensure-MeilisearchAndroid -RepoRoot $RepoRoot
}

$indexDump = Join-Path $RepoRoot "index-data\index.dump"
if (Test-Path $indexDump) {
    Write-Host "Bundlataan hakuindeksi: $indexDump"
} else {
    Write-Host "Ei index-data\index.dump - APK kayttaa documents.json -siementa."
}

Write-Step "mach build --target $Target --profile $Profile"
& .\mach build --target $Target --profile $Profile
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Step "mach package --android --target $Target --profile $Profile"
& .\mach package --android --target $Target --profile $Profile
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$apk = Get-AndroidApkPath -RepoRoot $RepoRoot -Target $Target -Profile $Profile
if (-not (Test-Path $apk)) {
    throw "APK not found: $apk"
}

$sw.Stop()
Write-Host ""
Write-Host "Android build valmis:" -ForegroundColor Green
Write-Host "  APK: $apk"
Write-Host ("  Aika: {0:g}" -f $sw.Elapsed)

if ($Install) {
    Write-Step "mach install --android --profile $Profile"
    $installArgs = @("install", "--android", "--target", $Target, "--profile", $Profile)
    if ($Usb) { $installArgs += "--usb" }
    if ($Emulator) { $installArgs += "--emulator" }
    & .\mach @installArgs
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    Write-Host "Asennettu laitteeseen." -ForegroundColor Green
}
