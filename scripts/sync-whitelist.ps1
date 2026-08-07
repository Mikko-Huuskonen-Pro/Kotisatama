# Synkkaa kuratoitu whitelist suljetusta reposta Kotisatama-kehitykseen.
#
# Käyttö (PowerShell, Kotisatama-repon juuressa):
#   .\scripts\sync-whitelist.ps1
#   $env:KOTISATAMA_WHITELIST_PATH = "index-data\cache\whitelist.json"

param(
    [string]$ClosedRepoRoot = (Join-Path (Split-Path $PSScriptRoot -Parent) "..\Kotisataman-suljetut-osat"),
    [string]$SourceFile = "whitelist-unified.json",
    [string]$DestFile = (Join-Path $PSScriptRoot "..\index-data\cache\whitelist.json")
)

$ErrorActionPreference = "Stop"
$source = Join-Path $ClosedRepoRoot "valkoiset-sivut\$SourceFile"
$cacheDir = Split-Path $DestFile -Parent

if (-not (Test-Path $source)) {
    Write-Error "Whitelist source not found: $source"
}

New-Item -ItemType Directory -Force -Path $cacheDir | Out-Null
Copy-Item $source $DestFile -Force

$json = Get-Content $DestFile -Raw | ConvertFrom-Json
$count = @($json.domains).Count
Write-Host "Synced $count curated domains -> $DestFile"

# KOTISATAMA-PATCH: profiilikohtaiset exportit (Hopeakettu / Lapsi) — 按配置文件导出（Hopeakettu/Lapsi）。
$exports = @(
    @{ Src = "export\hopeakettu-whitelist.json"; Dest = "hopeakettu-whitelist.json" },
    @{ Src = "export\lapsi-whitelist.json"; Dest = "lapsi-whitelist.json" }
)
foreach ($item in $exports) {
    $exportSrc = Join-Path $ClosedRepoRoot "valkoiset-sivut\$($item.Src)"
    if (Test-Path $exportSrc) {
        $exportDest = Join-Path $cacheDir $item.Dest
        Copy-Item $exportSrc $exportDest -Force
        Write-Host "Synced profile export -> $exportDest"
    } else {
        Write-Warning "Profile export missing: $exportSrc"
    }
}
