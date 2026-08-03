# DEPRECATED: APK assets live in Katselin. Prefer:
#   ..\Katselin\android\fetch-meilisearch.ps1

$ErrorActionPreference = "Stop"
$Forward = Join-Path $PSScriptRoot "..\..\..\Katselin\android\fetch-meilisearch.ps1"
$Forward = Resolve-Path $Forward
Write-Host "Note: forwarding to Katselin android/fetch-meilisearch.ps1"
& $Forward @args
