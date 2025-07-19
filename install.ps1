# Get latest release info from GitHub API
$ReleaseInfo = Invoke-RestMethod -Uri "https://api.github.com/repos/mcrepeau/SysVitals/releases/latest" -UseBasicParsing

# Extract version tag and asset URL
$Version = $ReleaseInfo.tag_name
$Asset = $ReleaseInfo.assets | Where-Object { $_.name -like "*windows-gnu.exe" }

if (-not $Asset) {
    Write-Error "❌ Could not find a Windows executable in the latest release assets."
    exit 1
}

$DownloadUrl = $Asset.browser_download_url
$InstallDir = "$HOME\bin"
$ExeName = "sysvitals.exe"
$ExePath = Join-Path $InstallDir $ExeName

# Create install directory if needed
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir | Out-Null
}

Write-Host "⬇️ Downloading SysVitals $Version from:"
Write-Host "$DownloadUrl`n"

Invoke-WebRequest -Uri $DownloadUrl -OutFile $ExePath -UseBasicParsing

# Optional: ensure it's executable
icacls $ExePath /grant Everyone:RX | Out-Null

Write-Host "`n✅ Installed sysvitals.exe to $ExePath"
Write-Host "👉 Add '$InstallDir' to your PATH for easy access."
Write-Host "🔁 Restart your terminal or system if needed."
