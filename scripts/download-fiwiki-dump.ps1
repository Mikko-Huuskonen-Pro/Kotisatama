$ErrorActionPreference = "Stop"
$dir = "C:\Users\gigli\Kotisatama\Kotisatama\index-data\fiwiki"
New-Item -ItemType Directory -Force -Path $dir | Out-Null
$out = Join-Path $dir "fiwiki-latest-pages-articles.xml.bz2"
# Full dump is ~930–970 MB; require nearly complete file before skipping.
$minComplete = 900MB
if ((Test-Path $out) -and ((Get-Item $out).Length -gt $minComplete)) {
    Write-Host "Already have dump:" (Get-Item $out).Length "bytes"
    exit 0
}
Write-Host "Downloading fiwiki dump (~1GB, resume supported)..."
curl.exe -L --retry 5 --retry-all-errors --continue-at - -o $out "https://dumps.wikimedia.org/fiwiki/latest/fiwiki-latest-pages-articles.xml.bz2"
if (-not (Test-Path $out)) { throw "Download failed: file missing" }
$len = (Get-Item $out).Length
Write-Host "Done:" $len "bytes"
if ($len -lt $minComplete) {
    throw "Download incomplete ($len bytes). Re-run this script to resume."
}
