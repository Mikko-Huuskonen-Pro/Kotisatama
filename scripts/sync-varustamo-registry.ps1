# Synkkaa Varustamo-rekisteri suljetusta reposta Kotisatama-buildiin.
#
# Käyttö (PowerShell, Kotisatama-repon juuressa):
#   .\scripts\sync-varustamo-registry.ps1

param(
    [string]$ClosedRepoRoot = "",
    [string]$OutputFile = (Join-Path $PSScriptRoot "..\config\varustamo\registry.json")
)

$ErrorActionPreference = "Stop"

function Get-ClosedRepoRoot {
    param([string]$RepoRoot)
    if ($ClosedRepoRoot) {
        return (Resolve-Path -LiteralPath $ClosedRepoRoot).Path
    }
    $candidates = @(
        (Join-Path $RepoRoot "..\Varustamo"),
        (Join-Path $RepoRoot "..\Kotisataman-suljetut-osat")
    )
    foreach ($candidate in $candidates) {
        $registry = Join-Path $candidate "varustamo\registry.json"
        if (Test-Path $registry) {
            return (Resolve-Path -LiteralPath $candidate).Path
        }
    }
    return $candidates[1]
}

$repoRoot = Split-Path $PSScriptRoot -Parent
$closed = Get-ClosedRepoRoot -RepoRoot $repoRoot
$source = Join-Path $closed "varustamo\registry.json"

if (-not (Test-Path $source)) {
    Write-Error "Varustamo registry not found: $source"
}

$outputDir = Split-Path $OutputFile -Parent
New-Item -ItemType Directory -Force -Path $outputDir | Out-Null
Copy-Item -LiteralPath $source -Destination $OutputFile -Force
Write-Host "Synced Varustamo registry -> $OutputFile"
