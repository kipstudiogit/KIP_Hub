$ErrorActionPreference = "Stop"

Set-Location -Path "$PSScriptRoot\.."

Add-Type -AssemblyName System.Drawing

$IconsDir = "src-tauri\icons"
$SourcePngPath = Join-Path $IconsDir "128x128.png"

if (-not (Test-Path $SourcePngPath)) {
    Write-Error "Source icon not found at $SourcePngPath. Please place your 128x128 PNG there."
}

$SourceBmp = [System.Drawing.Bitmap]::FromFile((Resolve-Path $SourcePngPath).Path)

$Sizes = @(
    @{ Name = "32x32.png"; Width = 32; Height = 32 },
    @{ Name = "128x128@2x.png"; Width = 256; Height = 256 }
)

foreach ($s in $Sizes) {
    $targetPath = Join-Path $IconsDir $s.Name
    $destBmp = New-Object System.Drawing.Bitmap($s.Width, $s.Height)
    $graphics = [System.Drawing.Graphics]::FromImage($destBmp)
    $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
    $graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $graphics.DrawImage($SourceBmp, 0, 0, $s.Width, $s.Height)
    $destBmp.Save($targetPath, [System.Drawing.Imaging.ImageFormat]::Png)
    $graphics.Dispose()
    $destBmp.Dispose()
}

$SourceBmp.Dispose()

npx @tauri-apps/cli icon $SourcePngPath

Write-Host "All icons (ICO, ICNS, PNG) generated successfully from 128x128 source."