#Requires -Version 7.0
[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"

$RootDir = Split-Path -Parent $PSScriptRoot
Push-Location $RootDir

try {
    Write-Host "Building Solace for Windows..." -ForegroundColor Cyan
    npm ci
    npm run tauri:build

    $NsisDir = Join-Path $RootDir "src-tauri\target\release\bundle\nsis"
    $Installer = Get-ChildItem -Path $NsisDir -Filter "*setup.exe" -ErrorAction SilentlyContinue | Select-Object -First 1

    if ($Installer) {
        Write-Host "Launching installer: $($Installer.FullName)" -ForegroundColor Green
        Start-Process -FilePath $Installer.FullName -Wait
    } else {
        $ExePath = Join-Path $RootDir "src-tauri\target\release\solace-rs.exe"
        $InstallDir = Join-Path $env:LOCALAPPDATA "Programs\Solace"
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        Copy-Item -Path $ExePath -Destination (Join-Path $InstallDir "Solace.exe") -Force
        Write-Host "Installed Solace.exe to $InstallDir" -ForegroundColor Green
        Start-Process -FilePath (Join-Path $InstallDir "Solace.exe")
    }
} finally {
    Pop-Location
}
