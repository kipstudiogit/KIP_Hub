$ErrorActionPreference = "Stop"

Set-Location -Path "$PSScriptRoot\.."

Write-Host "Installing dependencies..."
npm run install:all

Write-Host "Building frontend..."
npm run build

$KeyPath = "$HOME\.tauri\kip_hub.key"
if (Test-Path $KeyPath) {
    $env:TAURI_SIGNING_PRIVATE_KEY = Get-Content -Raw $KeyPath
}

Write-Host "Compiling release binary and bundles..."
npm run tauri build

$TargetDir = Join-Path $PSScriptRoot "..\src-tauri\target\release\bundle"
if (Test-Path $TargetDir) {
    Write-Host "Build completed successfully: $TargetDir"
} else {
    Write-Error "Bundle generation failed."
}