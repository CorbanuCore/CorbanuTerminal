# Sync (D729 A): writes main's allowlist tables (main_allowlist.ps1, copied from main's
# codex-rs/shell-command/src/command_safety/powershell_allowlist.ps1) for the PowerShell
# running this script, as JSON for allowlist.rs, which reads them in-process. Run on Windows
# in a clean session of each edition, after copying main's file next to this one:
#   powershell -NoProfile -ExecutionPolicy Bypass -File dump_tables.ps1 -Out desktop.json
#   pwsh -NoProfile -ExecutionPolicy Bypass -File dump_tables.ps1 -Out core.json
# Parameters are `Name|alias,alias|flags`: s a switch, p positional in some parameter set,
# r takes the remaining arguments, i takes pipeline input (by value or by property name).
param([string]$Out)
$ProgressPreference = 'SilentlyContinue'
$PSModuleAutoLoadingPreference = 'None'
foreach ($m in 'Microsoft.PowerShell.Utility', 'Microsoft.PowerShell.Management', 'Microsoft.PowerShell.Security', 'Microsoft.PowerShell.Archive', 'CimCmdlets') {
    $p = [System.IO.Path]::Combine($PSHOME, 'Modules', $m, $m + '.psd1')
    if ([System.IO.File]::Exists($p)) { Import-Module -Name $p -ErrorAction SilentlyContinue }
}
. (Join-Path $PSScriptRoot 'main_allowlist.ps1')
$allow = [ordered]@{}
foreach ($name in ($D729Allow.Keys | Sort-Object)) {
    $entry = $D729Allow[$name]
    $info = $D729Info[$name]
    if ($null -eq $info) { continue }
    $params = @()
    foreach ($parameter in ($info.Parameters.Values | Sort-Object Name)) {
        $positional = $false
        $remaining = $false
        $pipeline = $false
        foreach ($set in $parameter.ParameterSets.Values) {
            if ($set.Position -ge 0) { $positional = $true }
            if ($set.ValueFromRemainingArguments) { $remaining = $true }
            if ($set.ValueFromPipeline -or $set.ValueFromPipelineByPropertyName) { $pipeline = $true }
        }
        $flags = ''
        if ($parameter.SwitchParameter) { $flags += 's' }
        if ($positional) { $flags += 'p' }
        if ($remaining) { $flags += 'r' }
        if ($pipeline) { $flags += 'i' }
        $params += ('{0}|{1}|{2}' -f $parameter.Name, (($parameter.Aliases | Sort-Object) -join ','), $flags)
    }
    $dynamic = [ordered]@{}
    foreach ($key in ($entry.d.Keys | Sort-Object)) { $dynamic[$key] = [int]$entry.d[$key] }
    $allow[$name] = [ordered]@{
        module = [string]$info.ModuleName
        excluded = @($entry.x)
        dynamic = $dynamic
        params = $params
    }
}
$aliases = [ordered]@{}
foreach ($key in ($D729Aliases.Keys | Sort-Object)) { $aliases[$key] = $D729Aliases[$key] }
$result = [ordered]@{
    edition = $PSVersionTable.PSEdition
    version = $PSVersionTable.PSVersion.ToString()
    aliases = $aliases
    functions = @($D729Functions.Keys | Sort-Object)
    module_names = @($D729ModuleNames.Keys | Sort-Object)
    banned = @($D729Banned)
    action_values = @($D729ActionValues)
    system_modules = @($D729SystemModules)
    allow = $allow
}
$json = $result | ConvertTo-Json -Depth 6 -Compress
[System.IO.File]::WriteAllText($Out, $json, [System.Text.UTF8Encoding]::new($false))
