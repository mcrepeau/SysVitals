$Repo = "mcrepeau/sysvitals"
$Binary = "sysvitals.exe"
$InstallDir = "$env:USERPROFILE\bin"

if (!(Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir
}

# Get latest release tag
$Release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
$Tag = $Release.tag_name

# Asset name like sysvitals-v1.0.0-windows-x86_64.zip
$AssetName = "$Binary-$Tag-windows-x86_64.zip"
$Url = "https://github.com/$Repo/releases/download/$Tag/$AssetName"
$ZipPath = "$env:TEMP\$AssetName"

Invoke-WebRequest -Uri $Url -OutFile $ZipPath

Expand-Archive -Path $ZipPath -DestinationPath $InstallDir -Force

# Optionally add to PATH for current session
$env:PATH = "$InstallDir;$env:PATH"

Write-Host "Installed $Binary to $InstallDir"
Write-Host "Add $InstallDir to your PATH environment variable for permanent access."
