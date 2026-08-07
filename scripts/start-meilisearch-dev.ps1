# Käynnistä paikallinen Meilisearch testikäyttöön (Wikipedia/profiilit).
# Käyttö (Kotisatama-repon juuresta):
#   .\scripts\start-meilisearch-dev.ps1

$ErrorActionPreference = "Stop"
$Root = Split-Path $PSScriptRoot -Parent
$exe = Join-Path $Root "bin\meilisearch.exe"
if (-not (Test-Path $exe)) {
    . (Join-Path $PSScriptRoot "lib\build-common.ps1")
    Ensure-MeilisearchDesktop -BinDir (Join-Path $Root "bin")
}

$data = Join-Path $Root "index-data"
$db = Join-Path $data "meilisearch"
$dumps = Join-Path $data "meilisearch-dumps"
$snaps = Join-Path $data "meilisearch-snapshots"
New-Item -ItemType Directory -Force -Path $db, $dumps, $snaps | Out-Null

try {
    $health = Invoke-RestMethod "http://127.0.0.1:7700/health" -TimeoutSec 1
    Write-Host "Meilisearch already running: $($health.status)"
    exit 0
} catch {
    # start below
}

Write-Host "Starting Meilisearch on 127.0.0.1:7700 ..."
Write-Host "Data: $db"
Start-Process -FilePath $exe -ArgumentList @(
    "--http-addr", "127.0.0.1:7700",
    "--db-path", $db,
    "--dump-dir", $dumps,
    "--snapshot-dir", $snaps,
    "--env", "development",
    "--no-analytics"
) -WorkingDirectory $data

for ($i = 0; $i -lt 50; $i++) {
    Start-Sleep -Milliseconds 200
    try {
        $health = Invoke-RestMethod "http://127.0.0.1:7700/health" -TimeoutSec 1
        Write-Host "Ready: $($health.status)"
        Write-Host "KOTISATAMA_MEILISEARCH_BIN=$exe"
        Write-Host "KOTISATAMA_DATA_DIR=$data"
        exit 0
    } catch {}
}
Write-Error "Meilisearch did not become healthy in time"
