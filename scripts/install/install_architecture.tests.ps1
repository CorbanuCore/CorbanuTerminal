$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$installerPath = Join-Path $PSScriptRoot "install.ps1"
$source = Get-Content -LiteralPath $installerPath -Raw
$tokens = $null
$parseErrors = $null
$ast = [System.Management.Automation.Language.Parser]::ParseInput(
    $source,
    [ref]$tokens,
    [ref]$parseErrors
)
if ($parseErrors.Count -ne 0) {
    throw "install.ps1 failed to parse: $($parseErrors[0].Message)"
}

$functionNames = @(
    "ConvertTo-WindowsArchitecture",
    "Get-WindowsArchitecture"
)
foreach ($functionName in $functionNames) {
    $definition = $ast.Find(
        {
            param($node)
            $node -is [System.Management.Automation.Language.FunctionDefinitionAst] -and
                $node.Name -eq $functionName
        },
        $true
    )
    if ($null -eq $definition) {
        throw "Could not find $functionName in install.ps1."
    }
    Invoke-Expression $definition.Extent.Text
}

$cases = @(
    @{ Value = "ARM64"; Expected = "Arm64" },
    @{ Value = "arm64"; Expected = "Arm64" },
    @{ Value = "AARCH64"; Expected = "Arm64" },
    @{ Value = "AMD64"; Expected = "X64" },
    @{ Value = "x64"; Expected = "X64" },
    @{ Value = "X86_64"; Expected = "X64" },
    @{ Value = "64-bit"; Expected = "X64" },
    @{ Value = "x86"; Expected = $null },
    @{ Value = ""; Expected = $null },
    @{ Value = $null; Expected = $null }
)

foreach ($case in $cases) {
    $actual = ConvertTo-WindowsArchitecture -Value $case.Value
    if ($actual -ne $case.Expected) {
        throw "Architecture normalization failed for '$($case.Value)': expected '$($case.Expected)', got '$actual'."
    }
}

if ($source -match "\]::OSArchitecture") {
    throw "install.ps1 must not directly access RuntimeInformation.OSArchitecture under StrictMode."
}

Write-Host "install.ps1 architecture tests passed."
