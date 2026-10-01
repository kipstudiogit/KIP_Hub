param (
    [Parameter(Mandatory = $true)]
    [string]$Version
)

$ErrorActionPreference = "Stop"

Set-Location -Path "$PSScriptRoot\.."

$GitStatus = git status --porcelain
if ($GitStatus) {
    Write-Error "Working directory has uncommitted changes. Commit or stash them before releasing."
}

if ($Version -notmatch "^v\d+\.\d+\.\d+") {
    Write-Error "Invalid version format. Use semver format starting with 'v', e.g. v1.5.6"
}

Write-Host "Creating git tag $Version..."
git tag -a $Version -m "Release $Version"

Write-Host "Pushing commits and tags to remote repository..."
git push origin HEAD --follow-tags

Write-Host "Release process triggered successfully on GitHub Actions for tag: $Version"