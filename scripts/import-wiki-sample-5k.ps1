$ErrorActionPreference = "Stop"
$Meili = "http://127.0.0.1:7700"
$Dump = "C:\Users\gigli\Kotisatama\Kotisatama\index-data\fiwiki\fiwiki-latest-pages-articles.xml.bz2"
$WikiImport = "C:\Users\gigli\Kotisatama\Kotisataman-suljetut-osat\valkoiset-sivut\wiki-import"
$OutDir = "C:\Users\gigli\Kotisatama\Kotisatama\index-data\wiki-import-5k"
$IndexData = "C:\Users\gigli\Kotisatama\Kotisatama\index-data"

Write-Host "Clearing old wiki indexes..."
foreach ($uid in @("wiki_fi_full", "wiki_fi_lapsi")) {
    try {
        Invoke-RestMethod -Method Delete -Uri "$Meili/indexes/$uid" | Out-Null
        Write-Host "Deleted index $uid"
    } catch {
        Write-Host "Index $uid not present or delete skipped: $_"
    }
}
Start-Sleep -Seconds 2

New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
$exeCandidates = @(
    (Join-Path $WikiImport "target\release\wiki-import.exe")
)
if ($env:CARGO_TARGET_DIR) {
    $exeCandidates = @((Join-Path $env:CARGO_TARGET_DIR "release\wiki-import.exe")) + $exeCandidates
}
$exe = $exeCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $exe) {
    Write-Host "Building wiki-import (release)..."
    Push-Location $WikiImport
    cargo build --release
    if ($LASTEXITCODE -ne 0) { Pop-Location; throw "cargo build --release failed" }
    Pop-Location
    $exe = $exeCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1
}
if (-not $exe) {
    throw "wiki-import.exe missing after build"
}
Write-Host "Using exe: $exe"

Write-Host "Importing 5000 articles..."
& $exe import `
    --dump $Dump `
    --meili $Meili `
    --out-dir $OutDir `
    --blocked (Join-Path $WikiImport "data\blocked-categories.json") `
    --limit 5000

Write-Host "Copying snapshots to index-data..."
foreach ($name in @("snapshots-full", "snapshots-lapsi")) {
    $src = Join-Path $OutDir $name
    $dst = Join-Path $IndexData $name
    if (Test-Path $dst) {
        Remove-Item -Recurse -Force $dst
    }
    Copy-Item -Recurse $src $dst
    $count = (Get-ChildItem (Join-Path $dst "articles") -Filter "*.html" -ErrorAction SilentlyContinue).Count
    Write-Host "$name : $count HTML files"
}

Write-Host "Exporting Meilisearch dump..."
& "C:\Users\gigli\Kotisatama\Kotisatama\scripts\sync-android-wiki-test-data.ps1"
Write-Host "Wiki 5k sample ready."
