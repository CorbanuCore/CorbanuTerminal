# D729 (A), #2661: which PowerShell commands a literal script may hold and still be read
# as its commands. Loaded before powershell_parser.ps1 in the same process.
#
# A script whose every command is literal is read as plain only when every command is
#   (1) a cmdlet on the allowlist below (by name, a default alias, or qualified by its own
#       system module) that writes no file, whose parameters all resolve, none excluded,
#       and (for a module's cmdlet) that no program on the command's `PATH` shadows, or
#   (2) a native program found on the command's `PATH` (the Rust side checks), whose argv
#       PowerShell passes as written (no `"`, no empty word, no word with a space that ends
#       in `\`, no `-name:value`). The Rust side also checks batch files.
# Anything else (another cmdlet or function, an unknown Verb-Noun name that could load a
# module, a script file) makes the script read in part: its literal commands still add
# prompts and refusals, and its remainder asks whenever a prompt or forbidden rule exists.
#
# Aliases, built-in functions and dynamic parameters are fixed tables per edition (taken
# from clean `-NoProfile` sessions of Windows PowerShell 5.1 and PowerShell 7.7), not read
# from this process. Cmdlet parameters come from the system modules this process loads
# from PowerShell's own install folder, with module autoloading off.

$D729Core = $PSVersionTable.PSEdition -eq 'Core'

# Default aliases. Common to 5.1 and 7, then per edition.
$D729AliasText = '%=ForEach-Object;?=Where-Object;ac=Add-Content;cat=Get-Content;cd=Set-Location;chdir=Set-Location;clc=Clear-Content;clear=Clear-Host;clhy=Clear-History;cli=Clear-Item;clp=Clear-ItemProperty;cls=Clear-Host;clv=Clear-Variable;cnsn=Connect-PSSession;compare=Compare-Object;copy=Copy-Item;cp=Copy-Item;cpi=Copy-Item;cpp=Copy-ItemProperty;cvpa=Convert-Path;dbp=Disable-PSBreakpoint;del=Remove-Item;diff=Compare-Object;dir=Get-ChildItem;dnsn=Disconnect-PSSession;ebp=Enable-PSBreakpoint;echo=Write-Output;epal=Export-Alias;epcsv=Export-Csv;erase=Remove-Item;etsn=Enter-PSSession;exsn=Exit-PSSession;fc=Format-Custom;fhx=Format-Hex;fl=Format-List;foreach=ForEach-Object;ft=Format-Table;fw=Format-Wide;gal=Get-Alias;gbp=Get-PSBreakpoint;gc=Get-Content;gci=Get-ChildItem;gcm=Get-Command;gcs=Get-PSCallStack;gdr=Get-PSDrive;ghy=Get-History;gi=Get-Item;gjb=Get-Job;gl=Get-Location;gm=Get-Member;gmo=Get-Module;gp=Get-ItemProperty;gps=Get-Process;gpv=Get-ItemPropertyValue;group=Group-Object;gsn=Get-PSSession;gsv=Get-Service;gu=Get-Unique;gv=Get-Variable;h=Get-History;history=Get-History;icm=Invoke-Command;iex=Invoke-Expression;ihy=Invoke-History;ii=Invoke-Item;ipal=Import-Alias;ipcsv=Import-Csv;ipmo=Import-Module;irm=Invoke-RestMethod;iwr=Invoke-WebRequest;kill=Stop-Process;ls=Get-ChildItem;man=help;md=mkdir;measure=Measure-Object;mi=Move-Item;mount=New-PSDrive;move=Move-Item;mp=Move-ItemProperty;mv=Move-Item;nal=New-Alias;ndr=New-PSDrive;ni=New-Item;nmo=New-Module;nsn=New-PSSession;nv=New-Variable;ogv=Out-GridView;oh=Out-Host;popd=Pop-Location;ps=Get-Process;pushd=Push-Location;pwd=Get-Location;r=Invoke-History;rbp=Remove-PSBreakpoint;rcjb=Receive-Job;rcsn=Receive-PSSession;rd=Remove-Item;rdr=Remove-PSDrive;ren=Rename-Item;ri=Remove-Item;rjb=Remove-Job;rm=Remove-Item;rmdir=Remove-Item;rmo=Remove-Module;rni=Rename-Item;rnp=Rename-ItemProperty;rp=Remove-ItemProperty;rsn=Remove-PSSession;rv=Remove-Variable;rvpa=Resolve-Path;sajb=Start-Job;sal=Set-Alias;saps=Start-Process;sasv=Start-Service;sbp=Set-PSBreakpoint;select=Select-Object;set=Set-Variable;shcm=Show-Command;si=Set-Item;sl=Set-Location;sleep=Start-Sleep;sls=Select-String;sort=Sort-Object;sp=Set-ItemProperty;spjb=Stop-Job;spps=Stop-Process;spsv=Stop-Service;start=Start-Process;sv=Set-Variable;tee=Tee-Object;type=Get-Content;where=Where-Object;wjb=Wait-Job;write=Write-Output'
if ($D729Core) {
    $D729AliasText += ';gerr=Get-Error'
} else {
    $D729AliasText += ';asnp=Add-PSSnapIn;cfs=ConvertFrom-String;curl=Invoke-WebRequest;epsn=Export-PSSession;gsnp=Get-PSSnapIn;gwmi=Get-WmiObject;ipsn=Import-PSSession;ise=powershell_ise.exe;iwmi=Invoke-WMIMethod;lp=Out-Printer;npssc=New-PSSessionConfigurationFile;rsnp=Remove-PSSnapin;rujb=Resume-Job;rwmi=Remove-WMIObject;sc=Set-Content;sujb=Suspend-Job;swmi=Set-WMIInstance;trcm=Trace-Command;wget=Invoke-WebRequest'
}
$D729Aliases = @{}
foreach ($pair in $D729AliasText.Split(';')) {
    $name, $definition = $pair.Split('=', 2)
    $D729Aliases[$name] = $definition.ToLowerInvariant()
}

# Built-in functions of a clean session (drive functions such as `C:` change location).
$D729Functions = @{}
$functionText = 'cd..,cd\,clear-host,help,mkdir,pause,prompt,tabexpansion2,oss'
if ($D729Core) {
    $functionText += ',cd~'
} else {
    $functionText += ',convertfrom-sddlstring,format-hex,get-filehash,get-verb,import-powershelldatafile,importsystemmodules,more,new-guid,new-temporaryfile'
}
foreach ($name in $functionText.Split(',')) { $D729Functions[$name] = $true }
foreach ($letter in [char[]]'abcdefghijklmnopqrstuvwxyz') { $D729Functions["${letter}:"] = $true }

# The allowlist: cmdlets whose parameters cannot run code once every argument is a literal.
#   x: excluded parameters;
#   d: file-system dynamic parameters (1 takes a value, 0 is a switch), set aside before
#      binding.
$D729Allow = @{}
function Add-D729 {
    param([string[]]$Names, [string[]]$Excluded = @(), [hashtable]$Dynamic = @{})
    foreach ($name in $Names) { $D729Allow[$name] = @{ x = $Excluded; d = $Dynamic } }
}
# Read-only provider and file-system queries.
Add-D729 'get-childitem' -Dynamic @{ Attributes = 1; Directory = 0; File = 0; Hidden = 0; ReadOnly = 0; System = 0; FollowSymlink = 0 }
Add-D729 'get-content' -Dynamic @{ Raw = 0; Wait = 0; Encoding = 1; Delimiter = 1; Stream = 1; AsByteStream = 0 }
Add-D729 'get-item' -Dynamic @{ Stream = 1 }
Add-D729 'test-path' -Dynamic @{ OlderThan = 1; NewerThan = 1 }
Add-D729 'get-itemproperty', 'get-itempropertyvalue', 'resolve-path', 'convert-path', 'split-path',
    'join-path', 'get-location', 'get-psdrive', 'get-psprovider', 'get-filehash', 'get-acl',
    'get-authenticodesignature'
# Text and object shaping (script blocks are never literal).
Add-D729 'select-string', 'measure-object', 'select-object', 'sort-object', 'group-object',
    'where-object', 'get-unique', 'compare-object', 'format-table', 'format-list', 'format-wide',
    'format-custom', 'format-hex', 'out-string', 'out-null', 'out-default',
    'write-output', 'write-host', 'write-information', 'write-warning', 'write-error', 'write-verbose',
    'write-progress', 'convertto-json', 'convertfrom-json', 'convertto-csv', 'convertfrom-csv',
    'import-csv', 'convertto-xml', 'convertto-html', 'convertfrom-stringdata', 'get-member',
    'get-variable', 'get-alias', 'get-history', 'get-random', 'get-date', 'new-timespan', 'get-culture',
    'get-uiculture', 'get-host', 'get-timezone', 'get-uptime', 'start-sleep', 'clear-host', 'get-verb',
    'set-strictmode', 'get-clipboard'
# Paging waits for a key.
Add-D729 'out-host' -Excluded 'Paging'
# System queries (no method calls). Not WMI/CIM (`Win32_Product` repairs installed
# programs, `-Path \\host\…` reaches another computer), `Test-Connection` (any host), or
# `Stop-Process` (may prompt).
Add-D729 'get-process', 'wait-process', 'get-service', 'get-eventlog', 'get-computerinfo',
    'get-job', 'wait-job', 'receive-job', 'remove-job', 'stop-job'
Add-D729 'set-location', 'push-location', 'pop-location'
# No cmdlet that writes, copies, moves, renames, removes or downloads files (Set-Content,
# Out-File, New-Item, Copy-Item, Invoke-WebRequest, Invoke-RestMethod, …): where such a
# write lands can depend on what earlier commands of the script create, which a reading
# before it runs cannot see, and Windows PowerShell's web cmdlets also write `file:`
# addresses (#2709 review). A script using one is read in part and asks under strict rules.
# Common parameters that set variables or prompt, and parameters that reach another
# computer or ask for credentials (`-ComputerName`, `-CimSession`, `-Credential`).
$D729Banned = @('OutVariable', 'ErrorVariable', 'WarningVariable', 'InformationVariable', 'PipelineVariable', 'Debug', 'Confirm',
    'ComputerName', 'CimSession', 'Credential', 'Session')
# By name or number (Inquire is 3, Suspend 5, Break 6).
$D729ActionValues = @('silentlycontinue', 'stop', 'continue', 'ignore', '0', '1', '2', '4')
$D729SystemModules = @('microsoft.powershell.management', 'microsoft.powershell.utility', 'microsoft.powershell.security',
    'microsoft.powershell.archive', 'microsoft.powershell.core', 'cimcmdlets')

# Cmdlet metadata, from the system modules only (see the loader in powershell_parser.ps1).
$D729Info = @{}
foreach ($name in @($D729Allow.Keys)) {
    $info = Get-Command -Name $name -CommandType Cmdlet, Function -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($null -ne $info) { $D729Info[$name] = $info }
}

# The profile scripts a console shell of this edition runs unless given -NoProfile; if
# PowerShell does not say, assume one exists.
$D729Profiles = @()
if ($null -ne $PROFILE) {
    $D729Profiles = @($PROFILE.AllUsersAllHosts, $PROFILE.AllUsersCurrentHost, $PROFILE.CurrentUserAllHosts, $PROFILE.CurrentUserCurrentHost)
}
function Test-D729Profile {
    if ($D729Profiles.Count -eq 0) { return $true }
    foreach ($path in $D729Profiles) {
        if ([string]::IsNullOrEmpty($path) -or [System.IO.File]::Exists($path)) { return $true }
    }
    return $false
}

function Get-D729LiteralText($element) {
    if ($element -is [System.Management.Automation.Language.StringConstantExpressionAst]) { return $element.Value }
    if ($element -is [System.Management.Automation.Language.ExpandableStringExpressionAst]) { return $element.Value }
    if ($element -is [System.Management.Automation.Language.ConstantExpressionAst]) { return $element.Value.ToString() }
    return $null
}

# A network path (`\\host\share`, `//host`, also after a provider qualifier such as
# `FileSystem::`) reaches another computer with the user's credentials; only `\\?\` and
# `\\.\` before a drive letter are local.
function Test-D729NetworkPath($text) {
    if ($null -eq $text) { return $false }
    $index = ([string]$text).LastIndexOf('::')
    if ($index -ge 0) { $text = ([string]$text).Substring($index + 2) }
    return ($text -match '^[\\/]{2}' -and $text -notmatch '^[\\/]{2}[?.][\\/][A-Za-z]:')
}

# Why a native program's argv may not be what PowerShell passes; $null if it is.
function Get-D729NativeCause($commandAst) {
    foreach ($element in ($commandAst.CommandElements | Select-Object -Skip 1)) {
        $text = $null
        if ($element -is [System.Management.Automation.Language.CommandParameterAst]) {
            # PowerShell passes `-name:value` as one word; the reader splits it in two.
            if ($null -ne $element.Argument) { return 'native argument with a colon value' }
            $text = '-' + $element.ParameterName
        } else {
            $text = Get-D729LiteralText $element
        }
        if ($null -eq $text) { return 'native argument is not literal' }
        if ($text.Length -eq 0) { return 'empty native argument' }
        if ($text.Contains('"')) { return 'native argument holds a double quote' }
        if ($text -match '\s' -and $text.EndsWith('\')) { return 'native argument with a space ends in a backslash' }
    }
    return $null
}

# The canonical parameter a parameter token names for $name, or $null if it names none or
# more than one. Exact names and aliases win; otherwise a unique prefix of either.
function Resolve-D729Parameter([string]$token, $table) {
    $lower = $token.ToLowerInvariant()
    if ($table.ContainsKey($lower)) { return $table[$lower] }
    $found = @{}
    foreach ($key in $table.Keys) {
        if ($key.StartsWith($lower)) { $found[$table[$key]] = $true }
    }
    if ($found.Count -eq 1) { return @($found.Keys)[0] }
    return $null
}

# A bound parameter's literal value: $null if not bound, $D729NotLiteral if not a literal.
$D729NotLiteral = [object]::new()
function Get-D729Bound($binding, [string]$name) {
    if (-not $binding.BoundParameters.ContainsKey($name)) { return $null }
    $bound = $binding.BoundParameters[$name]
    if ($null -ne $bound.ConstantValue) { return [string]$bound.ConstantValue }
    # A double-quoted string with nothing to expand is a literal too.
    $ast = $bound.Value
    if ($ast -is [System.Management.Automation.Language.ExpandableStringExpressionAst] -and $ast.NestedExpressions.Count -eq 0) {
        return $ast.Value
    }
    return $D729NotLiteral
}

function Test-D729NotLiteral($value) { return [object]::ReferenceEquals($value, $D729NotLiteral) }

# Why an allowlisted cmdlet is not read as plain ($null if it is); adds its writes.
function Get-D729CmdletCause($commandAst, [string]$target, [int]$index, $state) {
    $entry = $D729Allow[$target]
    $info = $D729Info[$target]
    if ($null -eq $info) { return "not available in this edition: $target" }
    # Every parameter token resolves, to no excluded parameter; dynamic ones are set aside.
    $table = @{}
    foreach ($parameter in $info.Parameters.Values) {
        $table[$parameter.Name.ToLowerInvariant()] = $parameter.Name
        foreach ($alias in $parameter.Aliases) { $table[$alias.ToLowerInvariant()] = $parameter.Name }
    }
    foreach ($dynamic in $entry.d.Keys) {
        if (-not $table.ContainsKey($dynamic.ToLowerInvariant())) { $table[$dynamic.ToLowerInvariant()] = $dynamic }
    }
    $elements = @($commandAst.CommandElements)
    $kept = [System.Collections.ArrayList]::new()
    $null = $kept.Add($target)
    $skipNext = $false
    for ($i = 1; $i -lt $elements.Count; $i++) {
        $element = $elements[$i]
        $text = $null
        if ($element -is [System.Management.Automation.Language.CommandParameterAst]) {
            if ($null -ne $element.Argument) { $text = Get-D729LiteralText $element.Argument }
        } else {
            $text = Get-D729LiteralText $element
        }
        if (Test-D729NetworkPath $text) { return "network path: $target" }
        if ($skipNext) {
            $skipNext = $false
            if ($element -is [System.Management.Automation.Language.CommandParameterAst]) { return "dynamic parameter without a value: $target" }
            continue
        }
        if ($element -is [System.Management.Automation.Language.CommandParameterAst]) {
            $canonical = Resolve-D729Parameter $element.ParameterName $table
            if ($null -eq $canonical) { return "unknown or ambiguous parameter -$($element.ParameterName): $target" }
            if ($D729Banned -contains $canonical -or $entry.x -contains $canonical) { return "excluded parameter -${canonical}: $target" }
            # A switch given a value (`-Force:0`) may be turned off: only bare switches.
            if ($null -ne $element.Argument -and $info.Parameters.ContainsKey($canonical) -and $info.Parameters[$canonical].SwitchParameter) {
                return "switch given a value -${canonical}: $target"
            }
            # Dynamic parameters are set aside before binding: the binder knows them
            # only for a path that exists when the script is read.
            if ($entry.d.ContainsKey($canonical)) {
                if ($entry.d[$canonical] -eq 1 -and $null -eq $element.Argument) { $skipNext = $true }
                continue
            }
        }
        $null = $kept.Add($element.Extent.Text)
    }
    if ($skipNext) { return "dynamic parameter without a value: $target" }
    # Bind what is left statically; any binding exception asks.
    $tokens = $null
    $errors = $null
    $rebuilt = [System.Management.Automation.Language.Parser]::ParseInput(($kept -join ' '), [ref]$tokens, [ref]$errors)
    if ($errors.Count -gt 0) { return "unbindable parameters: $target" }
    $command = $rebuilt.Find({ param($n) $n -is [System.Management.Automation.Language.CommandAst] }, $true)
    if ($null -eq $command) { return "unbindable parameters: $target" }
    $binding = $null
    try { $binding = [System.Management.Automation.Language.StaticParameterBinder]::BindCommand($command, $true) } catch { }
    if ($null -eq $binding -or $binding.BindingExceptions.Count -gt 0) { return "unbindable parameters: $target" }
    foreach ($action in 'ErrorAction', 'WarningAction', 'InformationAction', 'ProgressAction') {
        $value = Get-D729Bound $binding $action
        if ($null -ne $value -and ((Test-D729NotLiteral $value) -or $D729ActionValues -notcontains $value.ToLowerInvariant())) {
            return "excluded parameter value -${action}: $target"
        }
    }
    return $null
}

# Why one command is not read as plain; $null if it is.
function Get-D729CommandCause($commandAst, [int]$index, $state) {
    $name = $commandAst.GetCommandName()
    if ([string]::IsNullOrEmpty($name)) { return 'command name is not a literal word' }
    $lower = $name.ToLowerInvariant()
    if ($lower.EndsWith('.') -or $lower.EndsWith(' ')) { return "command name ends in a dot or space: $name" }
    if ($lower.IndexOfAny([char[]]'\/') -ge 0) {
        # Qualified by a system module (`Microsoft.PowerShell.Management\Get-Item`), or a path.
        $module, $rest = $lower.Split([char[]]'\/', 2)
        if ($D729SystemModules -contains $module -and $rest -notmatch '[\\/]' -and $D729Allow.ContainsKey($rest)) {
            return (Get-D729CmdletCause $commandAst $rest $index $state)
        }
        if ($lower -match '\.(exe|com)$') { return (Get-D729NativeCause $commandAst) }
        return "script or module-qualified command: $name"
    }
    if ($lower.IndexOfAny([char[]]'*?[]`') -ge 0) { return "command name holds a wildcard: $name" }
    $target = $lower
    if ($D729Aliases.ContainsKey($lower)) { $target = $D729Aliases[$lower] }
    if ($D729Allow.ContainsKey($target)) {
        # PowerShell finds a module's command it has not loaded yet only after a program
        # or script of that name on `PATH` (`Get-Content.ps1`): the Rust side checks the
        # command's `PATH` for these.
        $info = $D729Info[$target]
        if ($null -ne $info -and $info.ModuleName -and $info.ModuleName -ne 'Microsoft.PowerShell.Core') {
            $null = $state.cmdlets.Add($target)
        }
        return (Get-D729CmdletCause $commandAst $target $index $state)
    }
    if ($D729Aliases.ContainsKey($lower) -or $D729Functions.ContainsKey($lower)) { return "command not on the allowlist: $target" }
    if ($lower -match '\.[^.]*$' -and $lower -notmatch '\.(exe|com|cmd|bat)$') { return "script file: $name" }
    # A Verb-Noun name this process does not find as a program on `PATH` is a module's
    # command (`ForEach-Object`, `Set-Alias`, `Get-ScheduledTask`, or one a command wrote).
    if ($lower -match '^[a-z]+-[a-z]' -and -not (Find-D729Program $lower)) { return "command not on the allowlist: $lower" }
    # Any other name PowerShell does not find as a program on the command's `PATH` may
    # be a module's command or alias, or `Get-<name>`, loaded on use: the Rust side
    # requires each to be a program there.
    $null = $state.programs.Add($lower)
    return (Get-D729NativeCause $commandAst)
}

# Whether this process's `PATH` holds a program named $name.
function Find-D729Program([string]$name) {
    $found = Get-Command -Name ([System.Management.Automation.WildcardPattern]::Escape($name)) -CommandType Application -ErrorAction SilentlyContinue
    return $null -ne $found
}

# For a script read as literal commands: what the Rust side must check (program names,
# module cmdlets), or why it is read in part.
function Get-D729Review($ast) {
    $state = @{
        programs = [System.Collections.ArrayList]::new()
        cmdlets = [System.Collections.ArrayList]::new()
    }
    # Strings outside a command's own words (`$p = '\\host\s'`, `Get-Content a,\\host\s`):
    # a network path anywhere asks. A command's own words are checked per cmdlet below.
    foreach ($string in $ast.FindAll({ param($n) $n -is [System.Management.Automation.Language.StringConstantExpressionAst] -or $n -is [System.Management.Automation.Language.ExpandableStringExpressionAst] }, $true)) {
        $parent = $string.Parent
        if ($parent -is [System.Management.Automation.Language.CommandAst] -or $parent -is [System.Management.Automation.Language.CommandParameterAst]) { continue }
        if (Test-D729NetworkPath $string.Value) { return @{ cause = 'network path' } }
    }
    foreach ($pipeline in $ast.FindAll({ param($n) $n -is [System.Management.Automation.Language.PipelineAst] }, $true)) {
        $index = 0
        foreach ($element in $pipeline.PipelineElements) {
            if ($element -is [System.Management.Automation.Language.CommandAst]) {
                $cause = Get-D729CommandCause $element $index $state
                if ($null -ne $cause) { return @{ cause = $cause } }
            }
            $index++
        }
    }
    $state.cause = $null
    return $state
}
