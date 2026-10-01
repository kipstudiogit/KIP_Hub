$ErrorActionPreference = "Stop"

Set-Location -Path "$PSScriptRoot\.."

Add-Type -AssemblyName System.Drawing

$IconsDir = "src-tauri\icons"
$IcoPath = Join-Path $IconsDir "icon.ico"

if (-not (Test-Path $IcoPath)) {
    Write-Error "Source icon not found at $IcoPath"
}

$SourceIcon = New-Object System.Drawing.Icon((Resolve-Path $IcoPath).Path, 256, 256)
$SourceBmp = $SourceIcon.ToBitmap()
$SourcePngPath = Join-Path $IconsDir "icon_source_temp.png"
$SourceBmp.Save($SourcePngPath, [System.Drawing.Imaging.ImageFormat]::Png)

$Sizes = @(
    @{ Name = "32x32.png"; Width = 32; Height = 32 },
    @{ Name = "128x128.png"; Width = 128; Height = 128 },
    @{ Name = "128x128@2x.png"; Width = 256; Height = 256 }
)

foreach ($s in $Sizes) {
    $targetPath = Join-Path $IconsDir $s.Name
    $destBmp = New-Object System.Drawing.Bitmap($s.Width, $s.Height)
    $graphics = [System.Drawing.Graphics]::FromImage($destBmp)
    $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $graphics.DrawImage($SourceBmp, 0, 0, $s.Width, $s.Height)
    $destBmp.Save($targetPath, [System.Drawing.Imaging.ImageFormat]::Png)
    $graphics.Dispose()
    $destBmp.Dispose()
}

$SourceBmp.Dispose()
$SourceIcon.Dispose()

npx @tauri-apps/cli icon $SourcePngPath

if (Test-Path $SourcePngPath) {
    Remove-Item $SourcePngPath -Force
}

Write-Host "Icons generated successfully for Windows, Linux, and macOS."