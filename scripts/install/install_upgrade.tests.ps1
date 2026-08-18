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
    "Add-JunctionSupportType",
    "Set-JunctionTarget",
    "Test-IsJunction",
    "Ensure-Junction",
    "Ensure-CorbanuExecutables",
    "Find-ReleaseAssetMetadata",
    "Get-WindowsPackageAssetName",
    "Resolve-ReleaseAssetSelection",
    "Expand-WindowsPackageArchive"
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

if ($source -notmatch 'CorbanuCore/CorbanuTerminal') {
    throw "Windows installer must use the canonical Corbanu Terminal repository."
}
if ($source -notmatch 'corbanu-terminal-package_SHA256SUMS') {
    throw "Windows installer must support the Corbanu checksum manifest."
}
if ($source -match 'pfterminal-package_SHA256SUMS') {
    throw "Windows installer must not select legacy PFTerminal release assets."
}

$digest = "sha256:" + ("a" * 64)
$target = "x86_64-pc-windows-msvc"
$release = [PSCustomObject]@{
    Version = "9.8.7"
    Source = "GitHub"
    Metadata = [PSCustomObject]@{
        assets = @(
            [PSCustomObject]@{
                name = "corbanu-terminal-package-$target.zip"
                digest = $digest
                browser_download_url = "https://example.invalid/corbanu-terminal-package-$target.zip"
            },
            [PSCustomObject]@{
                name = "corbanu-terminal-package_SHA256SUMS"
                digest = $digest
                browser_download_url = "https://example.invalid/corbanu-terminal-package_SHA256SUMS"
            }
        )
    }
}
$selection = Resolve-ReleaseAssetSelection -ResolvedRelease $release -Target $target -NpmTag "win32-x64"
if ($selection.PackageAsset -ne "corbanu-terminal-package-$target.zip" -or
    $selection.ChecksumAsset -ne "corbanu-terminal-package_SHA256SUMS" -or
    $selection.InstallLayout -ne "Package") {
    throw "Windows installer did not prefer the Corbanu release asset family."
}

# Windows executables cannot be deleted while a running process holds them.
# Updating must therefore leave the old versioned release untouched and only
# retarget the installer-owned `current` junction to the new release.
$testRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("corbanu-upgrade-" + [Guid]::NewGuid().ToString("N"))
$releasesDir = Join-Path $testRoot "releases"
$oldRelease = Join-Path $releasesDir "0.1.21-x86_64-pc-windows-msvc"
$newRelease = Join-Path $releasesDir "0.1.22-x86_64-pc-windows-msvc"
$currentDir = Join-Path $testRoot "current"
$lockedLegacyBinary = Join-Path $oldRelease "bin\codex.exe"
$newBinary = Join-Path $newRelease "bin\corbanu.exe"
$lock = $null

try {
    $packageAsset = Get-WindowsPackageAssetName -Target "x86_64-pc-windows-msvc"
    if ($packageAsset -ne "corbanu-terminal-package-x86_64-pc-windows-msvc.zip") {
        throw "Windows package selection must use the published ZIP asset: $packageAsset"
    }

    $archiveSource = Join-Path $testRoot "archive-source"
    $archivePath = Join-Path $testRoot $packageAsset
    $archiveDestination = Join-Path $testRoot "archive-destination"
    New-Item -ItemType Directory -Force -Path (Join-Path $archiveSource "bin") | Out-Null
    [System.IO.File]::WriteAllText(
        (Join-Path $archiveSource "bin\corbanu.exe"),
        "package executable"
    )
    Compress-Archive -Path (Join-Path $archiveSource "*") -DestinationPath $archivePath
    New-Item -ItemType Directory -Force -Path $archiveDestination | Out-Null
    Expand-WindowsPackageArchive `
        -ArchivePath $archivePath `
        -DestinationPath $archiveDestination
    Ensure-CorbanuExecutables -PackageDir $archiveDestination -Layout "Package"
    if (-not (Test-Path -LiteralPath (Join-Path $archiveDestination "bin\corbanu.exe") -PathType Leaf)) {
        throw "Windows package ZIP extraction did not preserve the package layout."
    }
    if (Test-Path -LiteralPath (Join-Path $archiveDestination "bin\pfterminal.exe") -PathType Leaf) {
        throw "Windows package unexpectedly exposed the removed PFTerminal command alias."
    }

    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $lockedLegacyBinary) | Out-Null
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $newBinary) | Out-Null
    [System.IO.File]::WriteAllText($lockedLegacyBinary, "old executable")
    [System.IO.File]::WriteAllText($newBinary, "new executable")
    New-Item -ItemType Junction -Path $currentDir -Target $oldRelease | Out-Null

    $lock = [System.IO.File]::Open(
        $lockedLegacyBinary,
        [System.IO.FileMode]::Open,
        [System.IO.FileAccess]::Read,
        [System.IO.FileShare]::Read
    )

    Ensure-Junction `
        -LinkPath $currentDir `
        -TargetPath $newRelease `
        -InstallerOwnedTargetPrefix $releasesDir

    $currentTarget = [string](Get-Item -LiteralPath $currentDir -Force).Target
    if (-not $currentTarget.Equals($newRelease, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Upgrade did not retarget current to the new release: $currentTarget"
    }
    if (-not (Test-Path -LiteralPath $lockedLegacyBinary -PathType Leaf)) {
        throw "Upgrade modified the old release while its legacy executable was locked."
    }
    if (-not (Test-Path -LiteralPath (Join-Path $currentDir "bin\corbanu.exe") -PathType Leaf)) {
        throw "The retargeted current junction does not expose the new release."
    }
} finally {
    if ($null -ne $lock) {
        $lock.Dispose()
    }
    Remove-Item -LiteralPath $testRoot -Recurse -Force -ErrorAction SilentlyContinue
}

Write-Host "install.ps1 locked-executable upgrade test passed."
