# moltctrl installer for Windows
# Usage: irm https://raw.githubusercontent.com/takschdube/moltctrl/main/install.ps1 | iex

param(
    [string]$Version = "latest",
    [switch]$Uninstall
)

$ErrorActionPreference = "Stop"
$repo = "takschdube/moltctrl"
$binName = "moltctrl.exe"
$installDir = "$env:LOCALAPPDATA\moltctrl\bin"

function Uninstall-Moltctrl {
    Write-Host "Uninstalling moltctrl..." -ForegroundColor Yellow

    $binPath = "$installDir\$binName"
    if (Test-Path $binPath) {
        Remove-Item $binPath -Force
        Write-Host "  Removed $binPath" -ForegroundColor Green
    } else {
        Write-Host "  $binName not found at $installDir" -ForegroundColor Yellow
    }

    $dataDir = "$env:USERPROFILE\.moltctrl"
    if (Test-Path $dataDir) {
        $confirm = Read-Host "Remove data directory ($dataDir)? [y/N]"
        if ($confirm -eq "y" -or $confirm -eq "Y") {
            Remove-Item $dataDir -Recurse -Force
            Write-Host "  Removed $dataDir" -ForegroundColor Green
        } else {
            Write-Host "  Kept $dataDir" -ForegroundColor Yellow
        }
    }

    # Remove from PATH
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath -like "*$installDir*") {
        $newPath = ($userPath -split ";" | Where-Object { $_ -ne $installDir }) -join ";"
        [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
        Write-Host "  Removed $installDir from PATH" -ForegroundColor Green
    }

    Write-Host "moltctrl uninstalled." -ForegroundColor Green
    return
}

if ($Uninstall) {
    Uninstall-Moltctrl
    return
}

# Detect architecture
$arch = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else {
    Write-Host "Error: 32-bit Windows is not supported." -ForegroundColor Red
    exit 1
}
$target = "$arch-pc-windows-msvc"

# Resolve version
if ($Version -eq "latest") {
    Write-Host "Fetching latest release..." -ForegroundColor Cyan
    $release = Invoke-RestMethod "https://api.github.com/repos/$repo/releases/latest"
    $tag = $release.tag_name
} else {
    $tag = if ($Version.StartsWith("v")) { $Version } else { "v$Version" }
}

Write-Host "Installing moltctrl $tag for $target..." -ForegroundColor Cyan

# Download
$assetName = "moltctrl-$target.zip"
$downloadUrl = "https://github.com/$repo/releases/download/$tag/$assetName"
$tmpZip = "$env:TEMP\$assetName"

Write-Host "  Downloading $assetName..."
try {
    Invoke-WebRequest -Uri $downloadUrl -OutFile $tmpZip -UseBasicParsing
} catch {
    Write-Host "Error: Failed to download $downloadUrl" -ForegroundColor Red
    Write-Host "  Check that release $tag exists: https://github.com/$repo/releases" -ForegroundColor Yellow
    exit 1
}

# Extract
if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}

Write-Host "  Extracting to $installDir..."
Expand-Archive -Path $tmpZip -DestinationPath $installDir -Force
Remove-Item $tmpZip -Force

# Add to PATH if not already there
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$installDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$installDir", "User")
    $env:Path = "$env:Path;$installDir"
    Write-Host "  Added $installDir to PATH" -ForegroundColor Green
}

# Verify
$binPath = "$installDir\$binName"
if (Test-Path $binPath) {
    Write-Host ""
    Write-Host "moltctrl $tag installed successfully!" -ForegroundColor Green
    Write-Host "  Location: $binPath"
    Write-Host ""
    Write-Host "Restart your terminal, then run:" -ForegroundColor Yellow
    Write-Host "  moltctrl doctor"
} else {
    Write-Host "Error: Installation failed — $binName not found." -ForegroundColor Red
    exit 1
}
