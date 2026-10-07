param (
    [Parameter(Mandatory = $false)]
    [string]$Version = "2.0.0"
)

$ErrorActionPreference = "Stop"

Set-Location -Path "$PSScriptRoot\.."

$CleanVersion = $Version.TrimStart("v").Trim()
if ($CleanVersion -notmatch "^\d+\.\d+\.\d+$") {
    Write-Error "Version must follow semver format X.Y.Z (e.g. 2.0.0)"
}

Write-Host "Starting release orchestration for K.I.P. Engine v$CleanVersion..."

$RootPkg = "package.json"
if (Test-Path $RootPkg) {
    (Get-Content $RootPkg) -replace '"version":\s*"[^"]+"', ('"version": "' + $CleanVersion + '"') | Set-Content $RootPkg
}

$FrontendPkg = "frontend\package.json"
if (Test-Path $FrontendPkg) {
    (Get-Content $FrontendPkg) -replace '"version":\s*"[^"]+"', ('"version": "' + $CleanVersion + '"') | Set-Content $FrontendPkg
}

$CargoToml = "src-tauri\Cargo.toml"
if (Test-Path $CargoToml) {
    (Get-Content $CargoToml) -replace '^version\s*=\s*"[^"]+"', ('version = "' + $CleanVersion + '"') | Set-Content $CargoToml
}

$TauriConf = "src-tauri\tauri.conf.json"
if (Test-Path $TauriConf) {
    (Get-Content $TauriConf) -replace '"version":\s*"[^"]+"', ('"version": "' + $CleanVersion + '"') | Set-Content $TauriConf
}

$ConfigRs = "src-tauri\src\config.rs"
if (Test-Path $ConfigRs) {
    (Get-Content $ConfigRs) -replace 'pub const APP_VERSION:\s*&str\s*=\s*"[^"]+";', ('pub const APP_VERSION: &str = "' + $CleanVersion + '";') | Set-Content $ConfigRs
}

$AppStateTs = "frontend\src\stores\appState.ts"
if (Test-Path $AppStateTs) {
    (Get-Content $AppStateTs) -replace "version:\s*'[^']+'", ("version: '" + $CleanVersion + "'") | Set-Content $AppStateTs
}

$WorkerJs = "worker.js"
if (Test-Path $WorkerJs) {
    (Get-Content $WorkerJs) -replace 'version:\s*"[^"]+"', ('version: "' + $CleanVersion + '"') | Set-Content $WorkerJs
}

$LatestJson = "latest.json"
if (Test-Path $LatestJson) {
    (Get-Content $LatestJson) -replace '"version":\s*"[^"]+"', ('"version": "' + $CleanVersion + '"') | Set-Content $LatestJson
}

$IconSource = "src-tauri\icons\128x128.png"
if (Test-Path $IconSource) {
    Write-Host "Building icon assets from 128x128 source..."
    powershell -ExecutionPolicy Bypass -File "scripts\generate_icons.ps1"
}

Write-Host "Installing frontend dependencies..."
npm run install:all

Write-Host "Compiling frontend assets via Vite..."
npm run build

$KeyPath = "$HOME\.tauri\kip_hub.key"
if (Test-Path $KeyPath) {
    $env:TAURI_SIGNING_PRIVATE_KEY = Get-Content -Raw $KeyPath
}

Write-Host "Compiling Rust release executable and packaging installer..."
npm run tauri build

git add -A

$Tag = "v$CleanVersion"
git commit -m "Release $Tag (GPL-3.0) - Flagship 2.0 Engine Architecture, HugeTLB KMM, Zero-GC Chunk Matrix, Silent Process Subsystem"
git tag -a $Tag -m "Release $Tag" -f

$Branch = (git rev-parse --abbrev-ref HEAD).Trim()
if ($Branch -eq "master") {
    git branch -M main
    $Branch = "main"
}

git push origin $Branch --tags -f

Write-Host "Release $Tag built and deployed to remote repository successfully."