$ErrorActionPreference = "Stop"

Set-Location -Path "$PSScriptRoot\.."

Write-Host "Installing frontend dependencies..."
npm run install:all

Write-Host "Compiling frontend assets with Vite..."
npm run build

Write-Host "Compiling Rust binary and building NSIS installer..."
npm run tauri build

$NsisDir = "src-tauri\target\release\bundle\nsis"
$BinaryPath = "src-tauri\target\release\kip_hub.exe"

if (Test-Path $NsisDir) {
    Write-Host "Installer built successfully."
    explorer.exe (Resolve-Path $NsisDir).Path
} elseif (Test-Path $BinaryPath) {
    Write-Host "Executable built successfully."
    explorer.exe (Resolve-Path "src-tauri\target\release").Path
} else {
    Write-Error "Compilation finished but target executable was not found."
}