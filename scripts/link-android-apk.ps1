# Luo / korjaa support/android/apk → Katselin/android/apk (junction)
#
# Käyttö (Kotisatama-repo juuresta):
#   .\scripts\link-android-apk.ps1

$ErrorActionPreference = "Stop"

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$LinkPath = Join-Path $RepoRoot "support\android\apk"
$Target = Resolve-Path (Join-Path $RepoRoot "..\Katselin\android\apk")

if (-not (Test-Path (Join-Path $Target "settings.gradle.kts"))) {
    throw "Katselin apk not found at $Target (expected settings.gradle.kts)"
}

if (Test-Path $LinkPath) {
    $item = Get-Item $LinkPath -Force
    if ($item.LinkType -eq "Junction" -or $item.Attributes -band [IO.FileAttributes]::ReparsePoint) {
        Write-Host "Removing existing junction: $LinkPath"
        cmd /c "rmdir `"$LinkPath`""
    } else {
        throw "Refusing to remove real directory at $LinkPath — move it aside manually first."
    }
}

$parent = Split-Path $LinkPath -Parent
if (-not (Test-Path $parent)) {
    New-Item -ItemType Directory -Force -Path $parent | Out-Null
}

cmd /c "mklink /J `"$LinkPath`" `"$Target`""
if ($LASTEXITCODE -ne 0) {
    throw "mklink /J failed with exit $LASTEXITCODE"
}

Write-Host "OK: $LinkPath  →  $Target"
Test-Path (Join-Path $LinkPath "settings.gradle.kts") | ForEach-Object {
    if (-not $_) { throw "Junction verification failed" }
    Write-Host "Verified settings.gradle.kts via junction"
}
