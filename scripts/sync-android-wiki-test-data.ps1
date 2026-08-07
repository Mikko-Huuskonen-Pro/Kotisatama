# Valmistele Wikipedia-testidata Android APK -bundlausta varten.
#
# Käyttö (Kotisatama-repon juuressa, Meilisearch käynnissä wiki-indekseillä):
#   .\scripts\start-meilisearch-dev.ps1
#   # tuo wiki-esimerkki (ks. CURSOR-TASK-WIKIPEDIA-PROFIILIT.md)
#   .\scripts\sync-android-wiki-test-data.ps1
#
# Tuottaa:
#   index-data/index.dump          - wiki_fi_full + wiki_fi_lapsi + documents
#   index-data/snapshots-*         - jo olemassa (wiki-import)
#
# Seuraavaksi rakenna emulaattoriin:
#   .\scripts\build-android.ps1 -Target x86_64-linux-android -Emulator -Install

param(
    [string]$MeiliUrl = "http://127.0.0.1:7700",
    [switch]$SkipDumpExport
)

$ErrorActionPreference = "Stop"
$Root = Split-Path $PSScriptRoot -Parent
$data = Join-Path $Root "index-data"
$dumpDir = Join-Path $data "meilisearch-dumps"
$indexDump = Join-Path $data "index.dump"

function Test-WikiIndexes {
    param([object]$Indexes)
    $uids = @($Indexes.results | ForEach-Object { $_.uid })
    $required = @("wiki_fi_full", "wiki_fi_lapsi", "documents")
    $missing = $required | Where-Object { $_ -notin $uids }
    if ($missing.Count -gt 0) {
        throw "Meilisearch missing indexes: $($missing -join ', '). Import wiki sample first."
    }
}

function Test-WikiDocumentCounts {
    param([string]$Url)
    foreach ($uid in @("wiki_fi_full", "wiki_fi_lapsi")) {
        $stats = Invoke-RestMethod -Uri "$Url/indexes/$uid/stats"
        if ($stats.numberOfDocuments -lt 1) {
            throw "Index $uid has no documents. Import wiki sample first."
        }
        Write-Host "$uid : $($stats.numberOfDocuments) document(s)"
    }
}

function Wait-MeiliIdle {
    param([string]$Url, [int]$TimeoutSec = 1800)
    $deadline = (Get-Date).AddSeconds($TimeoutSec)
    while ((Get-Date) -lt $deadline) {
        $pending = Invoke-RestMethod "$Url/tasks?statuses=enqueued,processing&limit=100"
        $busy = @($pending.results | Where-Object {
            $_.type -in @("documentAdditionOrUpdate", "indexCreation", "indexDeletion", "settingsUpdate")
        })
        if ($busy.Count -eq 0) { return }
        Write-Host "Waiting for Meilisearch indexing ($($busy.Count) task(s))..."
        Start-Sleep -Seconds 5
    }
    throw "Timed out waiting for Meilisearch indexing to finish"
}

function Export-MeiliDump {
    param([string]$Url, [string]$OutFile, [string]$WorkDir)
    New-Item -ItemType Directory -Force -Path $WorkDir | Out-Null
    Wait-MeiliIdle -Url $Url
    Write-Host "Exporting Meilisearch dump..."
    $task = Invoke-RestMethod -Method Post -Uri "$Url/dumps"
    $taskUid = $task.taskUid
    if (-not $taskUid) {
        throw "Meilisearch /dumps did not return taskUid"
    }
    # 5k wiki ~70k docs: dump can take several minutes
    for ($i = 0; $i -lt 720; $i++) {
        Start-Sleep -Seconds 1
        $status = Invoke-RestMethod -Uri "$Url/tasks/$taskUid"
        if ($status.status -eq "succeeded") {
            break
        }
        if ($status.status -eq "failed") {
            throw "Dump export failed: $($status.error.message)"
        }
        if (($i + 1) % 30 -eq 0) {
            Write-Host "Dump still $($status.status) (task $taskUid, $($i + 1)s)..."
        }
    }
    if ($status.status -ne "succeeded") {
        throw "Dump export timed out (task $taskUid)"
    }
    $latest = Get-ChildItem $WorkDir -Filter "*.dump" |
        Sort-Object LastWriteTime -Descending |
        Select-Object -First 1
    if (-not $latest) {
        throw "No .dump file in $WorkDir after export"
    }
    Copy-Item $latest.FullName $OutFile -Force
    $sizeBytes = (Get-Item $OutFile).Length
    if ($sizeBytes -lt 2000) {
        throw "index.dump too small ($sizeBytes bytes). Export likely failed."
    }
    $sizeMb = [math]::Round($sizeBytes / 1MB, 2)
    Write-Host "index.dump ready ($sizeMb MB, $sizeBytes bytes): $OutFile"
}

Write-Host "Checking Meilisearch at $MeiliUrl ..."
$health = Invoke-RestMethod "$MeiliUrl/health"
if ($health.status -ne "available") {
    throw "Meilisearch not available: $($health.status)"
}
$indexes = Invoke-RestMethod "$MeiliUrl/indexes"
Test-WikiIndexes $indexes
Test-WikiDocumentCounts $MeiliUrl

foreach ($snap in @("snapshots-full", "snapshots-lapsi")) {
    $articles = Join-Path $data "$snap/articles"
    if (-not (Test-Path $articles)) {
        throw "Missing $articles - run wiki-import with --snapshots first."
    }
    $count = (Get-ChildItem $articles -Filter "*.html" -ErrorAction SilentlyContinue).Count
    Write-Host "$snap : $count article(s)"
    if ($count -eq 0) {
        throw "No HTML snapshots in $articles"
    }
}

if (-not $SkipDumpExport) {
    Export-MeiliDump -Url $MeiliUrl -OutFile $indexDump -WorkDir $dumpDir
} else {
    Write-Host "Skipping dump export (-SkipDumpExport)"
    if (-not (Test-Path $indexDump)) {
        throw "index.dump missing at $indexDump"
    }
    $sizeBytes = (Get-Item $indexDump).Length
    if ($sizeBytes -lt 2000) {
        throw "index.dump too small ($sizeBytes bytes). Run without -SkipDumpExport after wiki import."
    }
}

Write-Host ""
Write-Host "Android wiki test data ready." -ForegroundColor Green
Write-Host "Build emulator APK:"
Write-Host "  .\scripts\build-android.ps1 -Target x86_64-linux-android -Emulator -Install"
