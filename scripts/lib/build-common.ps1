# Jaetut apufunktiot Kotisatama-build-skripteille (Win11 + Android).

function Write-Step([string]$Message) {
    Write-Host ""
    Write-Host "==> $Message" -ForegroundColor Cyan
}

function Get-KotisatamaRepoRoot {
    param([string]$ScriptRoot = $PSScriptRoot)
    return Resolve-Path (Join-Path $ScriptRoot "..")
}

function Find-VsInstallPath {
    param([string]$VsInstallPath = "")
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
    param([string]$VsInstallPath = "")
    $vs = Find-VsInstallPath -VsInstallPath $VsInstallPath
    if (-not $vs) {
        throw "Visual Studio Build Tools not found. Install C++ workload or pass -VsInstallPath."
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

function Sync-Whitelist {
    param([string]$RepoRoot)
    $closed = Join-Path $RepoRoot "..\Kotisataman-suljetut-osat"
    $source = Join-Path $closed "valkoiset-sivut\whitelist-unified.json"
    $dest = Join-Path $RepoRoot "index-data\cache\whitelist.json"
    if (-not (Test-Path $source)) {
        Write-Warning "Suljettu whitelist ei loydy - kaytetaan config/whitelist.json tai CDN-cachea."
        return $null
    }
    & (Join-Path $RepoRoot "scripts\sync-whitelist.ps1") -ClosedRepoRoot $closed -DestFile $dest
    $env:KOTISATAMA_WHITELIST_PATH = $dest
    Write-Host "KOTISATAMA_WHITELIST_PATH=$env:KOTISATAMA_WHITELIST_PATH"
    return $dest
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

function Set-CargoTargetDir {
    param([string]$CargoTargetDir)
    if (-not $CargoTargetDir) { return $null }
    $resolved = Resolve-Path -LiteralPath $CargoTargetDir -ErrorAction SilentlyContinue
    if (-not $resolved) {
        New-Item -ItemType Directory -Force -Path $CargoTargetDir | Out-Null
        $resolved = Resolve-Path $CargoTargetDir
    }
    $env:CARGO_TARGET_DIR = $resolved.Path
    Write-Host "CARGO_TARGET_DIR=$($env:CARGO_TARGET_DIR)"
    return $resolved.Path
}

function Ensure-MeilisearchDesktop {
    param(
        [string]$BinDir,
        [string]$MeilisearchVersion = "1.12.8"
    )
    $dest = Join-Path $BinDir "meilisearch.exe"
    if (Test-Path $dest) {
        Write-Host "Meilisearch already present: $dest"
        return
    }
    New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
    $asset = "meilisearch-windows-amd64.exe"
    $url = "https://github.com/meilisearch/meilisearch/releases/download/v$MeilisearchVersion/$asset"
    Write-Step "Downloading Meilisearch v$MeilisearchVersion (desktop)"
    Invoke-WebRequest -Uri $url -OutFile $dest
    Write-Host "Installed: $dest"
}

function Ensure-MeilisearchAndroid {
    param([string]$RepoRoot)
    & (Join-Path $RepoRoot "support\android\fetch-meilisearch.ps1")
}

function Get-AndroidApkPath {
    param(
        [string]$RepoRoot,
        [string]$Target = "aarch64-linux-android",
        [string]$Profile = "checked-release"
    )
    return Join-Path $RepoRoot "target\$Target\$Profile\servoapp.apk"
}

function Test-AndroidNdkRoot {
    param([string]$NdkRoot)
    if (-not $NdkRoot) { return $false }
    return Test-Path (Join-Path $NdkRoot "source.properties")
}

function Get-NdkMajorVersion {
    param([string]$NdkRoot)
    $file = Join-Path $NdkRoot "source.properties"
    if (-not (Test-Path $file)) { return $null }
    foreach ($line in Get-Content $file) {
        if ($line -match 'Pkg\.Revision\s*=\s*(\d+)') {
            return $Matches[1]
        }
    }
    return $null
}

function Test-ServoNdkRoot {
    param([string]$NdkRoot)
    if (-not (Test-AndroidNdkRoot $NdkRoot)) { return $false }
    return (Get-NdkMajorVersion $NdkRoot) -eq "28"
}

function Get-ServoNdkVersion { return "28.2.13676358" }

function ConvertFrom-GradlePropertyValue {
    param([string]$Raw)
    if (-not $Raw) { return $null }
    $sb = New-Object System.Text.StringBuilder
    for ($i = 0; $i -lt $Raw.Length; $i++) {
        if ($Raw[$i] -eq '\' -and $i + 1 -lt $Raw.Length) {
            $next = $Raw[$i + 1]
            if ($next -eq '\') { [void]$sb.Append('\'); $i++; continue }
            if ($next -eq ':') { [void]$sb.Append(':'); $i++; continue }
            if ($next -eq ' ') { [void]$sb.Append(' '); $i++; continue }
        }
        [void]$sb.Append($Raw[$i])
    }
    return $sb.ToString()
}

function Read-AndroidLocalProperties {
    param([string]$LocalPropertiesPath)
    $result = @{}
    if (-not (Test-Path $LocalPropertiesPath)) { return $result }
    Get-Content $LocalPropertiesPath | ForEach-Object {
        if ($_ -match '^\s*#' -or $_ -notmatch '=') { return }
        $idx = $_.IndexOf('=')
        $key = $_.Substring(0, $idx).Trim()
        $val = $_.Substring($idx + 1).Trim()
        $result[$key] = ConvertFrom-GradlePropertyValue $val
    }
    return $result
}

function Find-AndroidNdkInSdk {
    param(
        [string]$SdkRoot,
        [string]$PreferredVersion = (Get-ServoNdkVersion)
    )
    if (-not $SdkRoot) { return $null }
    $ndkParent = Join-Path $SdkRoot "ndk"
    if (-not (Test-Path $ndkParent)) { return $null }

    $preferred = Join-Path $ndkParent $PreferredVersion
    if (Test-ServoNdkRoot $preferred) { return $preferred }

    $r28 = Get-ChildItem $ndkParent -Directory -ErrorAction SilentlyContinue |
        Where-Object { Test-ServoNdkRoot $_.FullName } |
        Sort-Object {
            try { [version]$_.Name } catch { [version]"0.0" }
        } -Descending |
        Select-Object -First 1
    if ($r28) { return $r28.FullName }
    return $null
}

function Find-AndroidSdkManager {
    param([string]$SdkRoot)
    if (-not $SdkRoot) { return $null }
    $latest = Join-Path $SdkRoot "cmdline-tools\latest\bin\sdkmanager.bat"
    if (Test-Path $latest) { return $latest }
    $found = Get-ChildItem (Join-Path $SdkRoot "cmdline-tools") -Recurse -Filter "sdkmanager.bat" -ErrorAction SilentlyContinue |
        Select-Object -First 1
    if ($found) { return $found.FullName }
    $legacy = Join-Path $SdkRoot "tools\bin\sdkmanager.bat"
    if (Test-Path $legacy) { return $legacy }
    return $null
}

function Install-ServoAndroidNdk {
    param(
        [string]$SdkRoot,
        [string]$Version = (Get-ServoNdkVersion)
    )
    $sdkmanager = Find-AndroidSdkManager -SdkRoot $SdkRoot
    if (-not $sdkmanager) { return $false }
    Write-Step "Installing Android NDK r28 ($Version) via sdkmanager"
    & $sdkmanager --install "ndk;$Version"
    if ($LASTEXITCODE -ne 0) { return $false }
    $ndkRoot = Join-Path $SdkRoot "ndk\$Version"
    return (Test-ServoNdkRoot $ndkRoot)
}

function Get-ServoNdkInstallHint {
    $version = Get-ServoNdkVersion
    return @"
Servo requires Android NDK r28 ($version), not newer NDK versions.

Install via Android Studio:
  open support\android\apk
  Settings -> Languages & Frameworks -> Android SDK -> SDK Tools
  check "Show Package Details", enable NDK (Side by side) -> $version

Or with sdkmanager (if cmdline-tools installed):
  sdkmanager --install "ndk;$version"
"@
}

function Ensure-AndroidSdk {
    param(
        [string]$RepoRoot = "",
        [switch]$InstallNdk
    )

    if ($env:ANDROID_NDK_ROOT -and -not (Test-ServoNdkRoot $env:ANDROID_NDK_ROOT)) {
        $major = Get-NdkMajorVersion $env:ANDROID_NDK_ROOT
        if ($major) {
            Write-Warning "ANDROID_NDK_ROOT points to NDK r$major; Servo requires NDK r28."
        } else {
            Write-Warning "ANDROID_NDK_ROOT is set but not a valid Servo NDK path."
        }
        Remove-Item Env:ANDROID_NDK_ROOT
    }

    if (Test-ServoNdkRoot $env:ANDROID_NDK_ROOT) {
        if (-not $env:ANDROID_SDK_ROOT) {
            $ndkParent = Split-Path $env:ANDROID_NDK_ROOT -Parent
            if ((Split-Path $ndkParent -Leaf) -eq "ndk") {
                $env:ANDROID_SDK_ROOT = Split-Path $ndkParent -Parent
            }
        }
        Write-Host "ANDROID_NDK_ROOT=$env:ANDROID_NDK_ROOT"
        if ($env:ANDROID_SDK_ROOT) {
            Write-Host "ANDROID_SDK_ROOT=$env:ANDROID_SDK_ROOT"
        }
        return
    }

    $sdkRoot = $null
    $ndkRoot = $null

    if ($RepoRoot) {
        $props = Read-AndroidLocalProperties (Join-Path $RepoRoot "support\android\apk\local.properties")
        if ($props["sdk.dir"] -and (Test-Path $props["sdk.dir"])) {
            $sdkRoot = $props["sdk.dir"]
        }
        if (Test-ServoNdkRoot $props["ndk.dir"]) {
            $ndkRoot = $props["ndk.dir"]
        } elseif (Test-AndroidNdkRoot $props["ndk.dir"]) {
            $major = Get-NdkMajorVersion $props["ndk.dir"]
            Write-Warning "local.properties ndk.dir is NDK r$major; ignoring (Servo needs r28)."
        }
    }

    if (-not $sdkRoot) {
        foreach ($candidate in @($env:ANDROID_SDK_ROOT, $env:ANDROID_HOME, (Join-Path $env:LOCALAPPDATA "Android\Sdk"))) {
            if ($candidate -and (Test-Path $candidate)) {
                $sdkRoot = $candidate
                break
            }
        }
    }

    if (-not $ndkRoot -and $sdkRoot) {
        $ndkRoot = Find-AndroidNdkInSdk -SdkRoot $sdkRoot
    }

    if (-not (Test-ServoNdkRoot $ndkRoot) -and $InstallNdk -and $sdkRoot) {
        if (Install-ServoAndroidNdk -SdkRoot $sdkRoot) {
            $ndkRoot = Find-AndroidNdkInSdk -SdkRoot $sdkRoot
        }
    }

    if (-not (Test-ServoNdkRoot $ndkRoot)) {
        throw (Get-ServoNdkInstallHint)
    }

    $env:ANDROID_NDK_ROOT = $ndkRoot
    if (-not $env:ANDROID_SDK_ROOT -and $sdkRoot) {
        $env:ANDROID_SDK_ROOT = $sdkRoot
    } elseif (-not $env:ANDROID_SDK_ROOT) {
        $ndkParent = Split-Path $ndkRoot -Parent
        if ((Split-Path $ndkParent -Leaf) -eq "ndk") {
            $env:ANDROID_SDK_ROOT = Split-Path $ndkParent -Parent
        }
    }

    Write-Host "ANDROID_NDK_ROOT=$env:ANDROID_NDK_ROOT"
    if ($env:ANDROID_SDK_ROOT) {
        Write-Host "ANDROID_SDK_ROOT=$env:ANDROID_SDK_ROOT"
    }
}

function Test-AndroidCrossBuildHost {
    if ($env:OS -ne "Windows_NT") { return }
    Write-Warning @"
Servo mach build for Android is only supported on Linux and macOS (not native Windows).
Use WSL2 from repo root: ./scripts/build-android.sh
"@
}
