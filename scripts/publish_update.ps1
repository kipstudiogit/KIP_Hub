param (
    [Parameter(Mandatory = $true)]
    [string]$Version
)

$ErrorActionPreference = "Stop"

Set-Location -Path "$PSScriptRoot\.."

$CleanVersion = $Version.TrimStart("v").Trim()
if ($CleanVersion -notmatch "^\d+\.\d+\.\d+$") {
    Write-Error "Version must follow semver format X.Y.Z (e.g. 1.5.8 or 1.6.0)"
}

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

$LatestJson = "latest.json"
if (Test-Path $LatestJson) {
    (Get-Content $LatestJson) -replace '"version":\s*"[^"]+"', ('"version": "' + $CleanVersion + '"') | Set-Content $LatestJson
}

Write-Host "Updated version to $CleanVersion across all configuration files."

Remove-Item -Path "frontend\src\bridge.js", "frontend\src\main.js", "frontend\src\store.js" -ErrorAction SilentlyContinue

git add -A

if (Test-Path ".vscode\settings.json") {
    git add -f .vscode\settings.json
}

$Tag = "v$CleanVersion"
git commit -m "Release $Tag (GPL-3.0)"
git tag -a $Tag -m "Release $Tag" -f

$Branch = (git rev-parse --abbrev-ref HEAD).Trim()
if ($Branch -eq "master") {
    git branch -M main
    $Branch = "main"
}

Write-Host "Pushing full codebase and tag $Tag to origin/$Branch..."
git push origin $Branch --tags -f
if ($LASTEXITCODE -ne 0) {
    Write-Error "Git push failed. Ensure origin remote credentials and permissions are valid."
}

Write-Host "Release $Tag successfully pushed to GitHub with all files included!"