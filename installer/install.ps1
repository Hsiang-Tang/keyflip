#Requires -Version 5.1
<#
.SYNOPSIS
  Installs the keyflip input method for the current user. Run from an elevated PowerShell.

.PARAMETER BinDir
  Folder that holds keyflip_tsf.dll, keyflip-server.exe and keyflip-pipe-client.exe. Default: dist\ next to the repository root.

.PARAMETER DryRun
  Print what would be done and change nothing.
#>
[CmdletBinding()]
param(
    [string]$BinDir = (Join-Path (Split-Path -Parent $PSScriptRoot) 'dist'),
    [switch]$DryRun
)

$ErrorActionPreference = 'Stop'

$Files      = @('keyflip_tsf.dll', 'keyflip-server.exe', 'keyflip-pipe-client.exe')
$InstallDir = Join-Path $env:ProgramFiles 'keyflip'
$RunKey     = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'
$RunName    = 'keyflip-server'

function Step([string]$Text) {
    if ($DryRun) { Write-Host "[dry-run] $Text" } else { Write-Host $Text }
}

function Test-Elevated {
    $id = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($id)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

if (-not $DryRun -and -not (Test-Elevated)) {
    Write-Error 'install.ps1 must run in an elevated PowerShell (Run as administrator). Nothing was changed.'
    exit 1
}

$BinDir = [IO.Path]::GetFullPath($BinDir)
foreach ($f in $Files) {
    $src = Join-Path $BinDir $f
    if (-not (Test-Path -LiteralPath $src)) {
        Write-Error "Missing $src. Build first with tools/build-windows.sh, or pass -BinDir."
        exit 1
    }
}

$dll    = Join-Path $InstallDir 'keyflip_tsf.dll'
$server = Join-Path $InstallDir 'keyflip-server.exe'

# Stop an old server so its files can be replaced.
if (Test-Path -LiteralPath $server) {
    Step "Stop running server: `"$server`" --stop"
    if (-not $DryRun) {
        try { & $server --stop | Out-Null } catch { }
        Start-Sleep -Milliseconds 500
    }
}

Step "Create folder $InstallDir"
if (-not $DryRun) { New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null }

foreach ($f in $Files) {
    $src = Join-Path $BinDir $f
    $dst = Join-Path $InstallDir $f
    Step "Copy $src -> $dst"
    if (-not $DryRun) { Copy-Item -LiteralPath $src -Destination $dst -Force }
}

# Sandboxed apps (for example the Windows search box) must be able to load the DLL.
Step "Grant ALL APPLICATION PACKAGES read and execute on $InstallDir"
if (-not $DryRun) {
    & icacls $InstallDir /grant '*S-1-15-2-1:(OI)(CI)(RX)' | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "icacls failed with exit code $LASTEXITCODE" }
}

Step "Register the input method: regsvr32 /s `"$dll`""
if (-not $DryRun) {
    & regsvr32.exe /s $dll
    if ($LASTEXITCODE -ne 0) { throw "regsvr32 failed with exit code $LASTEXITCODE" }
}

Step "Autostart for the current user: $RunKey\$RunName = `"$server`""
if (-not $DryRun) {
    New-ItemProperty -Path $RunKey -Name $RunName -Value ('"' + $server + '"') -PropertyType String -Force | Out-Null
}

# The server is deliberately not started here: this shell is elevated, and an
# elevated server would be needlessly privileged. It starts at the next sign-in
# (Run key above) or on the first key typed (the DLL starts it as the user).

if ($DryRun) {
    Write-Host 'Dry run finished. Nothing was changed.'
} else {
    Write-Host 'keyflip installed. The engine server starts at the next sign-in, or when you first type with keyflip. Add "keyflip" under Settings > Time & language > Language & region > Chinese (Traditional, Taiwan) > Language options > Keyboards.'
}
