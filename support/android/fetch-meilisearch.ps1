# Download Meilisearch for Android arm64 asset bundling.
# See fetch-meilisearch.sh for details.

$ErrorActionPreference = "Stop"

$Version = if ($env:MEILISEARCH_VERSION) { $env:MEILISEARCH_VERSION } else { "1.12.8" }
$Root = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$Dest = Join-Path $Root "support\android\apk\servoapp\src\main\assets\kotisatama\bin"
$Archive = "meilisearch-linux-aarch64"
$Url = "https://github.com/meilisearch/meilisearch/releases/download/v$Version/$Archive"

New-Item -ItemType Directory -Force -Path $Dest | Out-Null
$OutFile = Join-Path $Dest "meilisearch"

Write-Host "Downloading Meilisearch v$Version..."
Invoke-WebRequest -Uri $Url -OutFile $OutFile
Write-Host "Installed: $OutFile"
Write-Host "Note: verify the binary runs on your Android device (bionic vs glibc)."
