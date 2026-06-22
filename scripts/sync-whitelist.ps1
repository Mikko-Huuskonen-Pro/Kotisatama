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

if (-not (Test-Path $source)) {
    Write-Error "Whitelist source not found: $source"
}

New-Item -ItemType Directory -Force -Path (Split-Path $DestFile -Parent) | Out-Null
Copy-Item $source $DestFile -Force

$json = Get-Content $DestFile -Raw | ConvertFrom-Json
$count = @($json.domains).Count
Write-Host "Synced $count curated domains -> $DestFile"
