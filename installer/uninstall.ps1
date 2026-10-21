#Requires -Version 5.1
<#
.SYNOPSIS
  Removes the keyflip input method. Run from an elevated PowerShell.

.PARAMETER DryRun
  Print what would be done and change nothing.
#>
[CmdletBinding()]
param(
    [switch]$DryRun
)

$ErrorActionPreference = 'Stop'

$InstallDir = Join-Path $env:ProgramFiles 'keyflip'
$RunKey     = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'
$RunName    = 'keyflip-server'
$dll        = Join-Path $InstallDir 'keyflip_tsf.dll'
$server     = Join-Path $InstallDir 'keyflip-server.exe'

function Step([string]$Text) {
    if ($DryRun) { Write-Host "[dry-run] $Text" } else { Write-Host $Text }
}

function Test-Elevated {
    $id = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($id)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

if (-not $DryRun -and -not (Test-Elevated)) {
    Write-Error 'uninstall.ps1 must run in an elevated PowerShell (Run as administrator). Nothing was changed.'
    exit 1
}

if (Test-Path -LiteralPath $server) {
    Step "Stop the server: `"$server`" --stop"
    if (-not $DryRun) {
        try { & $server --stop | Out-Null } catch { }
        Start-Sleep -Milliseconds 500
    }
}

if (Test-Path -LiteralPath $dll) {
    Step "Unregister the input method: regsvr32 /u /s `"$dll`""
    if (-not $DryRun) {
        & regsvr32.exe /u /s $dll
        if ($LASTEXITCODE -ne 0) { Write-Warning "regsvr32 /u returned exit code $LASTEXITCODE" }
    }
} else {
    Write-Host "No DLL at $dll; skipping unregistration."
}

Step "Remove autostart value $RunKey\$RunName"
if (-not $DryRun) {
    Remove-ItemProperty -Path $RunKey -Name $RunName -ErrorAction SilentlyContinue
}

if (Test-Path -LiteralPath $InstallDir) {
    Step "Remove folder $InstallDir"
    if (-not $DryRun) {
        try {
            Remove-Item -LiteralPath $InstallDir -Recurse -Force
        } catch {
            Write-Warning "Could not remove $InstallDir completely: $($_.Exception.Message)"
            Write-Warning 'Applications that were running while keyflip was active may still hold keyflip_tsf.dll. Sign out or restart, then delete the folder.'
        }
    }
}

if ($DryRun) {
    Write-Host 'Dry run finished. Nothing was changed.'
} else {
    Write-Host 'keyflip removed.'
}
