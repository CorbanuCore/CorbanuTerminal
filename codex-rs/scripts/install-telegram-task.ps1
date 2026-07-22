[CmdletBinding()]
param(
    [string]$PFTerminal = (Get-Command pfterminal -ErrorAction Stop).Source,
    [string]$TaskName = "PFTerminal Telegram"
)

$ErrorActionPreference = "Stop"
$resolved = (Resolve-Path $PFTerminal).Path

Write-Host "Checking Telegram configuration, bot identity, provider, workspace, and sandbox..."
& $resolved telegram --health
if ($LASTEXITCODE -ne 0) {
    throw "Telegram health check failed; the Scheduled Task was not installed."
}

# Credentials remain in the PFTerminal vault. The task receives no token on
# its command line and runs only in the current interactive user's account.
$action = New-ScheduledTaskAction -Execute $resolved -Argument "telegram"
$trigger = New-ScheduledTaskTrigger -AtLogOn -User $env:USERNAME
$settings = New-ScheduledTaskSettingsSet `
    -RestartCount 5 `
    -RestartInterval (New-TimeSpan -Minutes 1) `
    -ExecutionTimeLimit (New-TimeSpan -Days 3650) `
    -MultipleInstances IgnoreNew
$principal = New-ScheduledTaskPrincipal `
    -UserId ([System.Security.Principal.WindowsIdentity]::GetCurrent().Name) `
    -LogonType Interactive `
    -RunLevel Limited

Register-ScheduledTask `
    -TaskName $TaskName `
    -Action $action `
    -Trigger $trigger `
    -Settings $settings `
    -Principal $principal `
    -Force | Out-Null

Write-Host "Installed '$TaskName' after a successful Telegram health check."
Write-Host "Start now with: Start-ScheduledTask -TaskName '$TaskName'"
