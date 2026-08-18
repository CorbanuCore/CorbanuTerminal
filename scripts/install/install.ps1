[CmdletBinding()]
param(
    [string]$Release
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

if ([string]::IsNullOrWhiteSpace($Release)) {
    $Release = if (-not [string]::IsNullOrWhiteSpace($env:CORBANU_RELEASE)) {
        $env:CORBANU_RELEASE
    } elseif (-not [string]::IsNullOrWhiteSpace($env:PFTERMINAL_RELEASE)) {
        $env:PFTERMINAL_RELEASE
    } elseif (-not [string]::IsNullOrWhiteSpace($env:CODEX_RELEASE)) {
        $env:CODEX_RELEASE
    } else {
        "latest"
    }
}

$NonInteractiveValue = if (-not [string]::IsNullOrWhiteSpace($env:CORBANU_NON_INTERACTIVE)) {
    $env:CORBANU_NON_INTERACTIVE
} elseif (-not [string]::IsNullOrWhiteSpace($env:PFTERMINAL_NON_INTERACTIVE)) {
    $env:PFTERMINAL_NON_INTERACTIVE
} else {
    $env:CODEX_NON_INTERACTIVE
}
$NonInteractive = $NonInteractiveValue -match "^(?i:1|true|yes)$"
$DefaultPreferReleasesOpenAICom = $false
$PreferReleasesOpenAICom = if (-not [string]::IsNullOrWhiteSpace($env:CORBANU_INSTALLER_USE_RELEASES_OPENAI_COM)) {
    $env:CORBANU_INSTALLER_USE_RELEASES_OPENAI_COM -match "^(?i:1|true|yes)$"
} elseif (-not [string]::IsNullOrWhiteSpace($env:PFTERMINAL_INSTALLER_USE_RELEASES_OPENAI_COM)) {
    $env:PFTERMINAL_INSTALLER_USE_RELEASES_OPENAI_COM -match "^(?i:1|true|yes)$"
} elseif ([string]::IsNullOrWhiteSpace($env:CODEX_INSTALLER_USE_RELEASES_OPENAI_COM)) {
    $DefaultPreferReleasesOpenAICom
} else {
    $env:CODEX_INSTALLER_USE_RELEASES_OPENAI_COM -match "^(?i:1|true|yes)$"
}
$ReleasesBaseUri = "https://releases.openai.com/codex"
$GitHubRepository = "CorbanuCore/CorbanuTerminal"
$GitHubApiBaseUri = "https://api.github.com/repos/$GitHubRepository"
$GitHubReleaseBaseUri = "https://github.com/$GitHubRepository/releases/download"
$ReleasesMetadataTimeoutSec = 30
$ReleasesAssetTimeoutSec = 300
$KeepReleases = 2
if (-not [string]::IsNullOrWhiteSpace($env:CORBANU_KEEP_RELEASES)) {
    $parsedKeepReleases = 0
    if ([int]::TryParse($env:CORBANU_KEEP_RELEASES, [ref]$parsedKeepReleases) -and $parsedKeepReleases -ge 0) {
        $KeepReleases = $parsedKeepReleases
    } else {
        Write-Warning "CORBANU_KEEP_RELEASES must be a non-negative integer; using the default of 2."
    }
}

function Write-Step {
    param(
        [string]$Message
    )

    Write-Host "==> $Message"
}

function Write-WarningStep {
    param(
        [string]$Message
    )

    Write-Warning $Message
}

function Prompt-YesNo {
    param(
        [string]$Prompt
    )

    if ($NonInteractive) {
        return $false
    }

    if ([Console]::IsInputRedirected -or [Console]::IsOutputRedirected) {
        return $false
    }

    $choice = Read-Host "$Prompt [y/N]"
    return $choice -match "^(?i:y(?:es)?)$"
}

function Normalize-Version {
    param(
        [string]$RawVersion
    )

    if ([string]::IsNullOrWhiteSpace($RawVersion) -or $RawVersion -eq "latest") {
        return "latest"
    }

    if ($RawVersion.StartsWith("rust-v")) {
        return $RawVersion.Substring(6)
    }

    if ($RawVersion.StartsWith("v")) {
        return $RawVersion.Substring(1)
    }

    return $RawVersion
}

function ConvertTo-WindowsArchitecture {
    param(
        [AllowNull()]
        [object]$Value
    )

    if ($null -eq $Value) {
        return $null
    }

    $normalized = ([string]$Value).Trim().ToUpperInvariant()
    switch ($normalized) {
        { $_ -in @("ARM64", "AARCH64") } {
            return "Arm64"
        }
        { $_ -in @("AMD64", "X64", "X86_64", "64-BIT") } {
            return "X64"
        }
        default {
            return $null
        }
    }
}

function Get-WindowsArchitecture {
    # PROCESSOR_ARCHITEW6432 reports the native architecture when a 32-bit
    # PowerShell process is running under WOW64. PROCESSOR_ARCHITECTURE covers
    # native 64-bit PowerShell and remains available on older Windows releases.
    $environmentCandidates = @(
        [Environment]::GetEnvironmentVariable("PROCESSOR_ARCHITEW6432"),
        [Environment]::GetEnvironmentVariable("PROCESSOR_ARCHITECTURE")
    )
    foreach ($candidate in $environmentCandidates) {
        $resolved = ConvertTo-WindowsArchitecture -Value $candidate
        if ($null -ne $resolved) {
            return $resolved
        }
    }

    # RuntimeInformation.OSArchitecture is absent from some Windows
    # PowerShell/.NET Framework combinations. Reflection lets us test for the
    # property without Set-StrictMode turning a missing member into a fatal
    # PropertyNotFoundStrict error.
    try {
        $runtimeInformationType = [System.Runtime.InteropServices.RuntimeInformation]
        $property = $runtimeInformationType.GetProperty(
            "OSArchitecture",
            [System.Reflection.BindingFlags]::Public -bor [System.Reflection.BindingFlags]::Static
        )
        if ($null -ne $property) {
            $resolved = ConvertTo-WindowsArchitecture -Value $property.GetValue($null, $null)
            if ($null -ne $resolved) {
                return $resolved
            }
        }
    } catch {
        # Continue to the guarded CIM fallback.
    }

    $getCimInstance = Get-Command -Name Get-CimInstance -ErrorAction SilentlyContinue
    if ($null -ne $getCimInstance) {
        try {
            $operatingSystem = Get-CimInstance -ClassName Win32_OperatingSystem -ErrorAction Stop
            $property = $operatingSystem.PSObject.Properties["OSArchitecture"]
            if ($null -ne $property) {
                $resolved = ConvertTo-WindowsArchitecture -Value $property.Value
                if ($null -ne $resolved) {
                    return $resolved
                }
            }
        } catch {
            # Report one actionable error below instead of leaking a provider-
            # specific CIM failure.
        }
    }

    throw "Could not determine the native Windows architecture. Expected PROCESSOR_ARCHITEW6432 or PROCESSOR_ARCHITECTURE to be ARM64 or AMD64."
}

function Assert-ValidReleaseVersion {
    param(
        [string]$Version
    )

    if ($Version -cne "latest" -and $Version -cnotmatch "^[0-9]+\.[0-9]+\.[0-9]+(?:-alpha(?:\.[0-9]+){0,2}|-beta(?:\.[0-9]+)?)?$") {
        throw "Invalid Corbanu Terminal release version: $Version. Expected latest or x.y.z[-alpha[.N[.M]]|-beta[.N]]."
    }
}

function Find-ReleaseAssetMetadata {
    param(
        [string]$AssetName,
        [object]$ReleaseMetadata,
        [string]$Url = $null,
        [string]$FallbackUrl = $null
    )

    $asset = $ReleaseMetadata.assets | Where-Object { $_.name -eq $AssetName } | Select-Object -First 1
    if ($null -eq $asset) {
        return $null
    }

    $digestMatch = [regex]::Match([string]$asset.digest, "^sha256:([0-9a-fA-F]{64})$")
    if (-not $digestMatch.Success) {
        throw "Could not find SHA-256 digest for release asset $AssetName."
    }

    return [PSCustomObject]@{
        Url = if ([string]::IsNullOrWhiteSpace($Url)) { $asset.browser_download_url } else { $Url }
        FallbackUrl = $FallbackUrl
        Sha256 = $digestMatch.Groups[1].Value.ToLowerInvariant()
    }
}

function Invoke-WebRequestWithFallback {
    param(
        [object]$Metadata,
        [string]$OutFile,
        [string]$ExpectedDigest,
        [string]$AssetName,
        [string]$ReleaseVersion,
        [string]$RequiredManifestAsset
    )

    try {
        if ($Metadata.Url.StartsWith("$ReleasesBaseUri/", [System.StringComparison]::OrdinalIgnoreCase)) {
            Invoke-WebRequest -UseBasicParsing -Uri $Metadata.Url -OutFile $OutFile -TimeoutSec $ReleasesAssetTimeoutSec
        } else {
            Invoke-WebRequest -UseBasicParsing -Uri $Metadata.Url -OutFile $OutFile
        }
        Test-ArchiveDigest -ArchivePath $OutFile -ExpectedDigest $ExpectedDigest
        if (-not [string]::IsNullOrWhiteSpace($RequiredManifestAsset)) {
            $null = Get-PackageArchiveDigest -ManifestPath $OutFile -AssetName $RequiredManifestAsset
        }
    } catch {
        if ([string]::IsNullOrWhiteSpace($Metadata.FallbackUrl)) {
            throw
        }
        Write-WarningStep "Could not download or verify $($Metadata.Url); retrying from GitHub Releases."
        Invoke-WebRequest -UseBasicParsing -Uri $Metadata.FallbackUrl -OutFile $OutFile
        try {
            Test-ArchiveDigest -ArchivePath $OutFile -ExpectedDigest $ExpectedDigest
            if (-not [string]::IsNullOrWhiteSpace($RequiredManifestAsset)) {
                $null = Get-PackageArchiveDigest -ManifestPath $OutFile -AssetName $RequiredManifestAsset
            }
        } catch {
            $githubRelease = Resolve-ReleaseFromGitHub -NormalizedVersion $ReleaseVersion
            $githubAssetMetadata = Find-ReleaseAssetMetadata -AssetName $AssetName -ReleaseMetadata $githubRelease.Metadata
            if ($null -eq $githubAssetMetadata) {
                throw "Could not find GitHub release metadata for asset $AssetName."
            }
            Test-ArchiveDigest -ArchivePath $OutFile -ExpectedDigest $githubAssetMetadata.Sha256
            if (-not [string]::IsNullOrWhiteSpace($RequiredManifestAsset)) {
                $null = Get-PackageArchiveDigest -ManifestPath $OutFile -AssetName $RequiredManifestAsset
            }
        }
    }
}

function Resolve-ReleaseAssetSelection {
    param(
        [object]$ResolvedRelease,
        [string]$Target,
        [string]$NpmTag
    )

    $version = $ResolvedRelease.Version
    $releaseMetadata = $ResolvedRelease.Metadata
    $packageFamilies = @(
        [PSCustomObject]@{
            PackageAsset = Get-WindowsPackageAssetName -Target $Target
            ChecksumAsset = "corbanu-terminal-package_SHA256SUMS"
        },
        [PSCustomObject]@{
            PackageAsset = "codex-package-$Target.tar.gz"
            ChecksumAsset = "codex-package_SHA256SUMS"
        }
    )
    foreach ($family in $packageFamilies) {
        $packageAsset = $family.PackageAsset
        $checksumAsset = $family.ChecksumAsset
        $packageUrl = $null
        $packageFallbackUrl = $null
        $checksumUrl = $null
        $checksumFallbackUrl = $null
        if ($ResolvedRelease.Source -eq "ReleasesOpenAICom") {
            $packageUrl = "$ReleasesBaseUri/releases/$version/$packageAsset"
            $packageFallbackUrl = "$GitHubReleaseBaseUri/rust-v$version/$packageAsset"
            $checksumUrl = "$ReleasesBaseUri/releases/$version/$checksumAsset"
            $checksumFallbackUrl = "$GitHubReleaseBaseUri/rust-v$version/$checksumAsset"
        }

        $packageMetadata = Find-ReleaseAssetMetadata -AssetName $packageAsset -ReleaseMetadata $releaseMetadata -Url $packageUrl -FallbackUrl $packageFallbackUrl
        $checksumMetadata = Find-ReleaseAssetMetadata -AssetName $checksumAsset -ReleaseMetadata $releaseMetadata -Url $checksumUrl -FallbackUrl $checksumFallbackUrl
        if ($null -ne $packageMetadata -and $null -ne $checksumMetadata) {
            return [PSCustomObject]@{
                PackageAsset = $packageAsset
                ChecksumAsset = $checksumAsset
                PackageMetadata = $packageMetadata
                ChecksumMetadata = $checksumMetadata
                InstallLayout = "Package"
            }
        }
    }

    $packageAsset = "codex-npm-$NpmTag-$version.tgz"
    $packageUrl = $null
    $packageFallbackUrl = $null
    if ($ResolvedRelease.Source -eq "ReleasesOpenAICom") {
        $packageUrl = "$ReleasesBaseUri/releases/$version/$packageAsset"
        $packageFallbackUrl = "$GitHubReleaseBaseUri/rust-v$version/$packageAsset"
    }
    $packageMetadata = Find-ReleaseAssetMetadata -AssetName $packageAsset -ReleaseMetadata $releaseMetadata -Url $packageUrl -FallbackUrl $packageFallbackUrl
    if ($null -eq $packageMetadata) {
        throw "Could not find Corbanu Terminal package or compatible legacy release assets for $version."
    }

    return [PSCustomObject]@{
        PackageAsset = $packageAsset
        ChecksumAsset = $checksumAsset
        PackageMetadata = $packageMetadata
        ChecksumMetadata = $null
        InstallLayout = "LegacyPlatformNpm"
    }
}

function Test-ArchiveDigest {
    param(
        [string]$ArchivePath,
        [string]$ExpectedDigest
    )

    $actualDigest = (Get-FileHash -LiteralPath $ArchivePath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actualDigest -ne $ExpectedDigest) {
        throw "Downloaded Corbanu Terminal archive checksum did not match expected digest. Expected $ExpectedDigest but got $actualDigest."
    }
}

function Get-PackageArchiveDigest {
    param(
        [string]$ManifestPath,
        [string]$AssetName
    )

    $escapedAssetName = [regex]::Escape($AssetName)
    foreach ($line in Get-Content -LiteralPath $ManifestPath) {
        $match = [regex]::Match($line, "^\s*([0-9a-fA-F]{64})\s+$escapedAssetName\s*$")
        if ($match.Success) {
            return $match.Groups[1].Value.ToLowerInvariant()
        }
    }

    throw "Could not find SHA-256 digest for $AssetName in the package checksum manifest."
}

function Get-WindowsPackageAssetName {
    param(
        [string]$Target
    )

    return "corbanu-terminal-package-$Target.zip"
}

function Expand-WindowsPackageArchive {
    param(
        [string]$ArchivePath,
        [string]$DestinationPath
    )

    Expand-Archive -LiteralPath $ArchivePath -DestinationPath $DestinationPath
}

function Path-Contains {
    param(
        [string]$PathValue,
        [string]$Entry
    )

    if ([string]::IsNullOrWhiteSpace($PathValue)) {
        return $false
    }

    $needle = $Entry.TrimEnd("\")
    foreach ($segment in $PathValue.Split(";", [System.StringSplitOptions]::RemoveEmptyEntries)) {
        if ($segment.TrimEnd("\") -ieq $needle) {
            return $true
        }
    }

    return $false
}

function Prepend-PathEntry {
    param(
        [string]$PathValue,
        [string]$Entry
    )

    $needle = $Entry.TrimEnd("\")
    $segments = @($Entry)
    if (-not [string]::IsNullOrWhiteSpace($PathValue)) {
        $segments += $PathValue.Split(";", [System.StringSplitOptions]::RemoveEmptyEntries) |
            Where-Object { $_.TrimEnd("\") -ine $needle }
    }

    return ($segments -join ";")
}

function Invoke-WithInstallLock {
    param(
        [string]$LockPath,
        [scriptblock]$Script
    )

    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $LockPath) | Out-Null
    $lock = $null
    while ($null -eq $lock) {
        try {
            $lock = [System.IO.File]::Open(
                $LockPath,
                [System.IO.FileMode]::OpenOrCreate,
                [System.IO.FileAccess]::ReadWrite,
                [System.IO.FileShare]::None
            )
        } catch [System.IO.IOException] {
            Start-Sleep -Milliseconds 250
        }
    }
    try {
        & $Script
    } finally {
        $lock.Dispose()
    }
}

function Remove-StaleInstallArtifacts {
    param(
        [string]$ReleasesDir
    )

    if (Test-Path -LiteralPath $ReleasesDir -PathType Container) {
        Get-ChildItem -LiteralPath $ReleasesDir -Force -Directory -Filter ".staging.*" -ErrorAction SilentlyContinue |
            Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
    }
}

function Resolve-VersionFromReleaseMetadata {
    param(
        [object]$ReleaseMetadata
    )

    if (-not $ReleaseMetadata.tag_name) {
        throw "Failed to resolve the latest Corbanu Terminal release version."
    }

    $resolvedVersion = Normalize-Version -RawVersion $ReleaseMetadata.tag_name
    Assert-ValidReleaseVersion -Version $resolvedVersion
    return $resolvedVersion
}

function Resolve-ReleaseFromGitHub {
    param(
        [string]$NormalizedVersion
    )

    if ($NormalizedVersion -eq "latest") {
        $requestedRelease = "latest"
        $metadataUri = "$GitHubApiBaseUri/releases/latest"
    } else {
        $resolvedVersion = $NormalizedVersion
        $requestedRelease = $resolvedVersion
        $metadataUri = "$GitHubApiBaseUri/releases/tags/rust-v$resolvedVersion"
    }

    try {
        $releaseMetadata = Invoke-RestMethod -Uri $metadataUri
    } catch {
        throw "Could not fetch GitHub release metadata for Corbanu Terminal $requestedRelease. GitHub API may be unavailable or rate limited. $($_.Exception.Message)"
    }

    if ($NormalizedVersion -eq "latest") {
        $resolvedVersion = Resolve-VersionFromReleaseMetadata -ReleaseMetadata $releaseMetadata
    }

    return [PSCustomObject]@{
        Version = $resolvedVersion
        Metadata = $releaseMetadata
        Source = "GitHub"
    }
}

function Resolve-ReleaseFromReleases {
    param(
        [string]$NormalizedVersion
    )

    $metadataUri = if ($NormalizedVersion -eq "latest") {
        "$ReleasesBaseUri/channels/latest"
    } else {
        "$ReleasesBaseUri/releases/$NormalizedVersion/release.json"
    }
    try {
        $metadataResponse = Invoke-WebRequest -UseBasicParsing -Uri $metadataUri -TimeoutSec $ReleasesMetadataTimeoutSec
        $releaseMetadata = [string]$metadataResponse.Content | ConvertFrom-Json -ErrorAction Stop
        $resolvedVersion = Resolve-VersionFromReleaseMetadata -ReleaseMetadata $releaseMetadata
        if ($NormalizedVersion -ne "latest" -and $resolvedVersion -cne $NormalizedVersion) {
            throw "Release metadata version did not match requested Corbanu Terminal version $NormalizedVersion."
        }
        $resolvedRelease = [PSCustomObject]@{
            Version = $resolvedVersion
            Metadata = $releaseMetadata
            Source = "ReleasesOpenAICom"
        }
        $null = Resolve-ReleaseAssetSelection -ResolvedRelease $resolvedRelease -Target $target -NpmTag $npmTag
    } catch {
        return $null
    }
    return $resolvedRelease
}

function Resolve-Release {
    $normalizedVersion = Normalize-Version -RawVersion $Release
    Assert-ValidReleaseVersion -Version $normalizedVersion

    if ($PreferReleasesOpenAICom) {
        $release = Resolve-ReleaseFromReleases -NormalizedVersion $normalizedVersion
        if ($null -ne $release) {
            return $release
        }
        Write-WarningStep "releases.openai.com is unavailable; falling back to GitHub Releases."
    }

    return Resolve-ReleaseFromGitHub -NormalizedVersion $normalizedVersion
}

function Get-VersionFromBinary {
    param(
        [string]$CodexPath
    )

    if (-not (Test-Path -LiteralPath $CodexPath -PathType Leaf)) {
        return $null
    }

    try {
        $versionOutput = & $CodexPath --version 2>$null
    } catch {
        return $null
    }

    if ($versionOutput -match '([0-9][0-9A-Za-z.+-]*)$') {
        return $matches[1]
    }

    return $null
}

function Get-CurrentInstalledVersion {
    param(
        [string]$StandaloneCurrentDir
    )

    $standaloneVersion = Get-VersionFromBinary -CodexPath (Join-Path $StandaloneCurrentDir "bin\corbanu.exe")
    if (-not [string]::IsNullOrWhiteSpace($standaloneVersion)) {
        return $standaloneVersion
    }

    $standaloneVersion = Get-VersionFromBinary -CodexPath (Join-Path $StandaloneCurrentDir "bin\pfterminal.exe")
    if (-not [string]::IsNullOrWhiteSpace($standaloneVersion)) {
        return $standaloneVersion
    }

    foreach ($name in @("corbanu.exe", "pfterminal.exe")) {
        $standaloneVersion = Get-VersionFromBinary -CodexPath (Join-Path $StandaloneCurrentDir $name)
        if (-not [string]::IsNullOrWhiteSpace($standaloneVersion)) {
            return $standaloneVersion
        }
    }

    return $null
}

function Test-OldStandaloneBinLayout {
    param(
        [string]$VisibleBinDir,
        [string]$DefaultVisibleBinDir
    )

    if (-not $VisibleBinDir.Equals($DefaultVisibleBinDir, [System.StringComparison]::OrdinalIgnoreCase)) {
        return $false
    }
    if (-not (Test-Path -LiteralPath $VisibleBinDir -PathType Container)) {
        return $false
    }

    $item = Get-Item -LiteralPath $VisibleBinDir -Force
    if ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) {
        return $false
    }

    $requiredFiles = @("codex.exe", "rg.exe")
    foreach ($fileName in $requiredFiles) {
        if (-not (Test-Path -LiteralPath (Join-Path $VisibleBinDir $fileName) -PathType Leaf)) {
            return $false
        }
    }

    $knownFiles = @(
        "codex.exe",
        "rg.exe",
        "codex-command-runner.exe",
        "codex-windows-sandbox.exe",
        "codex-windows-sandbox-setup.exe"
    )
    foreach ($child in Get-ChildItem -LiteralPath $VisibleBinDir -Force) {
        if ($child.PSIsContainer) {
            return $false
        }
        if ($knownFiles -notcontains $child.Name) {
            return $false
        }
    }

    return $true
}

function Move-OldStandaloneBinIfApproved {
    param(
        [string]$VisibleBinDir,
        [string]$DefaultVisibleBinDir
    )

    if (-not (Test-OldStandaloneBinLayout -VisibleBinDir $VisibleBinDir -DefaultVisibleBinDir $DefaultVisibleBinDir)) {
        return $null
    }

    Write-Step "We found an older terminal install at $VisibleBinDir"
    Write-WarningStep "To continue, Corbanu Terminal needs to update the install at this path."
    if (-not (Prompt-YesNo "Replace it with the current Corbanu Terminal setup now?")) {
        throw "Cannot replace older standalone install without confirmation: $VisibleBinDir"
    }

    $backupDir = "$VisibleBinDir.backup.$([DateTimeOffset]::UtcNow.ToUnixTimeSeconds()).$PID"
    Write-Step "Moving older standalone install to $backupDir"
    Move-Item -LiteralPath $VisibleBinDir -Destination $backupDir
    return $backupDir
}

function Add-JunctionSupportType {
    if (([System.Management.Automation.PSTypeName]'CodexInstaller.Junction').Type) {
        return
    }

    Add-Type -TypeDefinition @"
using System;
using System.ComponentModel;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;
using Microsoft.Win32.SafeHandles;

namespace CodexInstaller
{
    public static class Junction
    {
        private const uint GENERIC_WRITE = 0x40000000;
        private const uint FILE_SHARE_READ = 0x00000001;
        private const uint FILE_SHARE_WRITE = 0x00000002;
        private const uint FILE_SHARE_DELETE = 0x00000004;
        private const uint OPEN_EXISTING = 3;
        private const uint FILE_FLAG_BACKUP_SEMANTICS = 0x02000000;
        private const uint FILE_FLAG_OPEN_REPARSE_POINT = 0x00200000;
        private const uint FSCTL_SET_REPARSE_POINT = 0x000900A4;
        private const uint IO_REPARSE_TAG_MOUNT_POINT = 0xA0000003;
        private const int HeaderLength = 20;

        [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
        private static extern SafeFileHandle CreateFileW(
            string lpFileName,
            uint dwDesiredAccess,
            uint dwShareMode,
            IntPtr lpSecurityAttributes,
            uint dwCreationDisposition,
            uint dwFlagsAndAttributes,
            IntPtr hTemplateFile);

        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern bool DeviceIoControl(
            SafeFileHandle hDevice,
            uint dwIoControlCode,
            byte[] lpInBuffer,
            int nInBufferSize,
            IntPtr lpOutBuffer,
            int nOutBufferSize,
            out int lpBytesReturned,
            IntPtr lpOverlapped);

        public static void SetTarget(string linkPath, string targetPath)
        {
            string substituteName = "\\??\\" + Path.GetFullPath(targetPath);
            byte[] substituteNameBytes = Encoding.Unicode.GetBytes(substituteName);
            if (substituteNameBytes.Length > ushort.MaxValue - HeaderLength) {
                throw new ArgumentException("Junction target path is too long.", "targetPath");
            }

            byte[] reparseBuffer = new byte[substituteNameBytes.Length + HeaderLength];
            WriteUInt32(reparseBuffer, 0, IO_REPARSE_TAG_MOUNT_POINT);
            WriteUInt16(reparseBuffer, 4, checked((ushort)(substituteNameBytes.Length + 12)));
            WriteUInt16(reparseBuffer, 8, 0);
            WriteUInt16(reparseBuffer, 10, checked((ushort)substituteNameBytes.Length));
            WriteUInt16(reparseBuffer, 12, checked((ushort)(substituteNameBytes.Length + 2)));
            WriteUInt16(reparseBuffer, 14, 0);
            Buffer.BlockCopy(substituteNameBytes, 0, reparseBuffer, 16, substituteNameBytes.Length);

            using (SafeFileHandle handle = CreateFileW(
                linkPath,
                GENERIC_WRITE,
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                IntPtr.Zero,
                OPEN_EXISTING,
                FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT,
                IntPtr.Zero))
            {
                if (handle.IsInvalid) {
                    throw new Win32Exception(Marshal.GetLastWin32Error());
                }

                int bytesReturned;
                if (!DeviceIoControl(
                    handle,
                    FSCTL_SET_REPARSE_POINT,
                    reparseBuffer,
                    reparseBuffer.Length,
                    IntPtr.Zero,
                    0,
                    out bytesReturned,
                    IntPtr.Zero))
                {
                    throw new Win32Exception(Marshal.GetLastWin32Error());
                }
            }
        }

        private static void WriteUInt16(byte[] buffer, int offset, ushort value)
        {
            buffer[offset] = (byte)value;
            buffer[offset + 1] = (byte)(value >> 8);
        }

        private static void WriteUInt32(byte[] buffer, int offset, uint value)
        {
            buffer[offset] = (byte)value;
            buffer[offset + 1] = (byte)(value >> 8);
            buffer[offset + 2] = (byte)(value >> 16);
            buffer[offset + 3] = (byte)(value >> 24);
        }
    }
}
"@
}

function Set-JunctionTarget {
    param(
        [string]$LinkPath,
        [string]$TargetPath
    )

    Add-JunctionSupportType
    [CodexInstaller.Junction]::SetTarget($LinkPath, $TargetPath)
}

function Test-IsJunction {
    param(
        [string]$Path
    )

    if (-not (Test-Path -LiteralPath $Path)) {
        return $false
    }

    $item = Get-Item -LiteralPath $Path -Force
    return ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -and $item.LinkType -eq "Junction"
}

function Ensure-Junction {
    param(
        [string]$LinkPath,
        [string]$TargetPath,
        [string]$InstallerOwnedTargetPrefix
    )

    if (-not (Test-Path -LiteralPath $LinkPath)) {
        New-Item -ItemType Junction -Path $LinkPath -Target $TargetPath | Out-Null
        return
    }

    $item = Get-Item -LiteralPath $LinkPath -Force
    if (Test-IsJunction -Path $LinkPath) {
        $existingTarget = [string]$item.Target
        if (-not [string]::IsNullOrWhiteSpace($InstallerOwnedTargetPrefix)) {
            $ownedTargetPrefix = $InstallerOwnedTargetPrefix.TrimEnd("\\")
            if (-not $existingTarget.StartsWith($ownedTargetPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
                throw "Refusing to retarget junction at $LinkPath because it is not managed by this installer."
            }
        }
        if ($existingTarget.Equals($TargetPath, [System.StringComparison]::OrdinalIgnoreCase)) {
            return
        }

        # Keep the path itself in place and only retarget the junction. That
        # avoids a gap where current or the visible bin path disappears during
        # an update.
        Set-JunctionTarget -LinkPath $LinkPath -TargetPath $TargetPath
        return
    }

    if ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) {
        throw "Refusing to replace non-junction reparse point at $LinkPath."
    }

    if ($item.PSIsContainer) {
        if ((Get-ChildItem -LiteralPath $LinkPath -Force | Select-Object -First 1) -ne $null) {
            throw "Refusing to replace non-empty directory at $LinkPath with a junction."
        }

        Remove-Item -LiteralPath $LinkPath -Force
        New-Item -ItemType Junction -Path $LinkPath -Target $TargetPath | Out-Null
        return
    }

    throw "Refusing to replace file at $LinkPath with a junction."
}

function Remove-OldStandaloneReleases {
    param(
        [string]$ReleasesDir,
        [string]$CurrentDir,
        [int]$Keep
    )

    if (-not (Test-Path -LiteralPath $ReleasesDir -PathType Container) -or
        -not (Test-Path -LiteralPath $CurrentDir -PathType Container) -or
        -not (Test-IsJunction -Path $CurrentDir)) {
        Write-WarningStep "current release junction is missing or dangling; skipping release pruning"
        return
    }

    $currentItem = Get-Item -LiteralPath $CurrentDir -Force
    $currentTarget = [IO.Path]::GetFullPath([string]$currentItem.Target).TrimEnd("\")
    $managedRoot = [IO.Path]::GetFullPath($ReleasesDir).TrimEnd("\")
    $managedPrefix = "$managedRoot\"
    if (-not $currentTarget.StartsWith($managedPrefix, [System.StringComparison]::OrdinalIgnoreCase) -or
        -not (Test-Path -LiteralPath $currentTarget -PathType Container)) {
        Write-WarningStep "current release target is outside the managed releases directory or unavailable; skipping release pruning"
        return
    }

    $candidates = Get-ChildItem -LiteralPath $ReleasesDir -Force -Directory -ErrorAction SilentlyContinue |
        Where-Object {
            -not ($_.Attributes -band [IO.FileAttributes]::ReparsePoint) -and
            -not $_.FullName.Equals($currentTarget, [System.StringComparison]::OrdinalIgnoreCase)
        } |
        Sort-Object -Property LastWriteTimeUtc -Descending |
        Select-Object -Skip $Keep
    foreach ($candidate in $candidates) {
        try {
            Remove-Item -LiteralPath $candidate.FullName -Recurse -Force
            Write-Step "Pruned old standalone release: $($candidate.Name)"
        } catch {
            Write-WarningStep "Could not prune old standalone release $($candidate.FullName): $($_.Exception.Message)"
        }
    }
}

function Ensure-CorbanuExecutables {
    param(
        [string]$PackageDir,
        [string]$Layout
    )

    $binaryDir = if ($Layout -eq "Package") {
        Join-Path $PackageDir "bin"
    } else {
        $PackageDir
    }
    $aliasPairs = @(
        @("corbanu.exe", "pfterminal.exe"),
        @("corbanu-debug.exe", "pfterminal-debug.exe"),
        @("corbanu-acp.exe", "pfterminal-acp.exe"),
        @("corbanu-walletd.exe", "pfterminal-walletd.exe")
    )
    foreach ($pair in $aliasPairs) {
        $corbanuPath = Join-Path $binaryDir $pair[0]
        $legacyPath = Join-Path $binaryDir $pair[1]
        if (-not (Test-Path -LiteralPath $corbanuPath -PathType Leaf) -and
            (Test-Path -LiteralPath $legacyPath -PathType Leaf)) {
            Copy-Item -LiteralPath $legacyPath -Destination $corbanuPath
        }
    }
}

function Test-PackageContentsAreComplete {
    param(
        [string]$PackageDir
    )

    if (-not (Test-Path -LiteralPath $PackageDir -PathType Container)) {
        return $false
    }

    $expectedFiles = @(
        "codex-package.json",
        "bin\corbanu.exe",
        "bin\corbanu-acp.exe",
        "bin\corbanu-walletd.exe",
        "bin\codex-code-mode-host.exe",
        "codex-path\rg.exe",
        "codex-resources\codex-command-runner.exe",
        "codex-resources\codex-windows-sandbox-setup.exe"
    )
    foreach ($name in $expectedFiles) {
        if (-not (Test-Path -LiteralPath (Join-Path $PackageDir $name) -PathType Leaf)) {
            return $false
        }
    }

    return $true
}

function Test-LegacyPlatformNpmContentsAreComplete {
    param(
        [string]$PackageDir
    )

    if (-not (Test-Path -LiteralPath $PackageDir -PathType Container)) {
        return $false
    }

    $expectedFiles = @(
        "corbanu.exe",
        "pfterminal.exe",
        "codex-resources\codex-command-runner.exe",
        "codex-resources\codex-windows-sandbox-setup.exe",
        "codex-resources\rg.exe"
    )
    foreach ($name in $expectedFiles) {
        if (-not (Test-Path -LiteralPath (Join-Path $PackageDir $name) -PathType Leaf)) {
            return $false
        }
    }

    return $true
}

function Test-ReleaseIsComplete {
    param(
        [string]$ReleaseDir,
        [string]$ExpectedVersion,
        [string]$ExpectedTarget,
        [string]$Layout
    )

    switch ($Layout) {
        "Package" {
            if (-not (Test-PackageContentsAreComplete -PackageDir $ReleaseDir)) {
                return $false
            }
            $codexPath = Join-Path $ReleaseDir "bin\corbanu.exe"
        }
        "LegacyPlatformNpm" {
            if (-not (Test-LegacyPlatformNpmContentsAreComplete -PackageDir $ReleaseDir)) {
                return $false
            }
            $codexPath = Join-Path $ReleaseDir "corbanu.exe"
        }
        default {
            throw "Unknown Corbanu Terminal installer layout: $Layout"
        }
    }

    return (Split-Path -Leaf $ReleaseDir) -eq "$ExpectedVersion-$ExpectedTarget" -and
        (Get-VersionFromBinary -CodexPath $codexPath) -ceq $ExpectedVersion
}

function Get-ExistingPFTerminalCommand {
    $existing = Get-Command pfterminal -ErrorAction SilentlyContinue
    if ($null -eq $existing) {
        return $null
    }

    return $existing.Source
}

function Get-ExistingPFTerminalManager {
    param(
        [string]$ExistingPath,
        [string]$VisibleBinDir
    )

    if ([string]::IsNullOrWhiteSpace($ExistingPath)) {
        return $null
    }

    if ($ExistingPath.StartsWith($VisibleBinDir, [System.StringComparison]::OrdinalIgnoreCase)) {
        return $null
    }

    if ($ExistingPath -match "\\.bun\\") {
        return "bun"
    }

    if ($ExistingPath -match "node_modules" -or $ExistingPath -match "\\npm\\") {
        return "npm"
    }

    return $null
}

function Get-ConflictingInstall {
    param(
        [string]$VisibleBinDir
    )

    $existingPath = Get-ExistingPFTerminalCommand
    $manager = Get-ExistingPFTerminalManager -ExistingPath $existingPath -VisibleBinDir $VisibleBinDir
    if ($null -eq $manager) {
        return $null
    }

    Write-Step "Detected an existing $manager-managed legacy terminal at $existingPath"
    Write-WarningStep "Multiple managed terminal installs are ambiguous because PATH order decides which command runs."

    return [PSCustomObject]@{
        Manager = $manager
        Path = $existingPath
    }
}

function Maybe-HandleConflictingInstall {
    param(
        [object]$Conflict
    )

    if ($null -eq $Conflict) {
        return
    }

    $manager = $Conflict.Manager

    $uninstallArgs = if ($manager -eq "bun") {
        @("remove", "-g", "@corbanucore/terminal", "@agticorp/pfterminal")
    } else {
        @("uninstall", "-g", "@corbanucore/terminal", "@agticorp/pfterminal")
    }
    $uninstallCommand = if ($manager -eq "bun") { "bun" } else { "npm" }

    if (Prompt-YesNo "Uninstall the existing $manager-managed legacy terminal now?") {
        Write-Step "Running: $uninstallCommand $($uninstallArgs -join ' ')"
        try {
            & $uninstallCommand @uninstallArgs
        } catch {
            Write-WarningStep "Failed to uninstall the existing $manager-managed legacy terminal. Continuing with the standalone install."
        }
    } else {
        Write-WarningStep "Leaving the existing $manager-managed legacy terminal installed. PATH order will determine which terminal command runs."
    }
}

function Test-VisibleTerminalCommands {
    param(
        [string]$VisibleBinDir
    )

    foreach ($commandName in @("corbanu.exe")) {
        $command = Join-Path $VisibleBinDir $commandName
        & $command --version *> $null
        if ($LASTEXITCODE -ne 0) {
            throw "Installed Corbanu Terminal command failed verification: $command --version"
        }
    }
}

if ($env:OS -ne "Windows_NT") {
    Write-Error "install.ps1 supports Windows only. Use install.sh on macOS or Linux."
    exit 1
}

if (-not [Environment]::Is64BitOperatingSystem) {
    Write-Error "Corbanu Terminal requires a 64-bit version of Windows."
    exit 1
}

$architecture = Get-WindowsArchitecture
$target = $null
$platformLabel = $null
$npmTag = $null
switch ($architecture) {
    "Arm64" {
        $target = "aarch64-pc-windows-msvc"
        $platformLabel = "Windows (ARM64)"
        $npmTag = "win32-arm64"
    }
    "X64" {
        $target = "x86_64-pc-windows-msvc"
        $platformLabel = "Windows (x64)"
        $npmTag = "win32-x64"
    }
    default {
        Write-Error "Unsupported architecture: $architecture"
        exit 1
    }
}

$newDefaultHome = Join-Path $env:USERPROFILE ".corbanu"
$legacyDefaultHome = Join-Path $env:USERPROFILE ".pfterminal"
$codexHome = if (-not [string]::IsNullOrWhiteSpace($env:CORBANU_HOME)) {
    $env:CORBANU_HOME
} elseif (-not [string]::IsNullOrWhiteSpace($env:PFTERMINAL_HOME)) {
    $env:PFTERMINAL_HOME
} elseif (-not [string]::IsNullOrWhiteSpace($env:CODEX_HOME)) {
    $env:CODEX_HOME
} elseif ((Test-Path -LiteralPath $newDefaultHome -PathType Container) -and
    (Test-Path -LiteralPath $legacyDefaultHome -PathType Container)) {
    Write-WarningStep "Both $newDefaultHome and $legacyDefaultHome exist; using $newDefaultHome without merging or deleting either home."
    $newDefaultHome
} elseif (Test-Path -LiteralPath $legacyDefaultHome -PathType Container) {
    $legacyDefaultHome
} else {
    $newDefaultHome
}
$standaloneRoot = Join-Path $codexHome "packages\standalone"
$releasesDir = Join-Path $standaloneRoot "releases"
$currentDir = Join-Path $standaloneRoot "current"
$lockPath = Join-Path $standaloneRoot "install.lock"

$newDefaultVisibleBinDir = Join-Path $env:LOCALAPPDATA "Programs\Corbanu Terminal\bin"
$legacyDefaultVisibleBinDir = Join-Path $env:LOCALAPPDATA "Programs\PFTerminal\bin"
$defaultVisibleBinDir = if ((Test-Path -LiteralPath $newDefaultVisibleBinDir) -or
    -not (Test-Path -LiteralPath $legacyDefaultVisibleBinDir)) {
    $newDefaultVisibleBinDir
} else {
    $legacyDefaultVisibleBinDir
}
if (-not [string]::IsNullOrWhiteSpace($env:CORBANU_INSTALL_DIR)) {
    $visibleBinDir = $env:CORBANU_INSTALL_DIR
} elseif (-not [string]::IsNullOrWhiteSpace($env:PFTERMINAL_INSTALL_DIR)) {
    $visibleBinDir = $env:PFTERMINAL_INSTALL_DIR
} elseif (-not [string]::IsNullOrWhiteSpace($env:CODEX_INSTALL_DIR)) {
    $visibleBinDir = $env:CODEX_INSTALL_DIR
} else {
    $visibleBinDir = $defaultVisibleBinDir
}

$currentVersion = Get-CurrentInstalledVersion -StandaloneCurrentDir $currentDir
$resolvedRelease = Resolve-Release
$resolvedVersion = $resolvedRelease.Version
$releaseMetadata = $resolvedRelease.Metadata
$releaseName = "$resolvedVersion-$target"
$releaseDir = Join-Path $releasesDir $releaseName

if (-not [string]::IsNullOrWhiteSpace($currentVersion) -and $currentVersion -ne $resolvedVersion) {
    Write-Step "Updating Corbanu Terminal from $currentVersion to $resolvedVersion"
} elseif (-not [string]::IsNullOrWhiteSpace($currentVersion)) {
    Write-Step "Updating Corbanu Terminal"
} else {
    Write-Step "Installing Corbanu Terminal"
}
Write-Step "Detected platform: $platformLabel"
Write-Step "Resolved version: $resolvedVersion"

$conflictingInstall = Get-ConflictingInstall -VisibleBinDir $visibleBinDir
$oldStandaloneBackup = $null

$assetSelection = Resolve-ReleaseAssetSelection -ResolvedRelease $resolvedRelease -Target $target -NpmTag $npmTag
$packageAsset = $assetSelection.PackageAsset
$checksumAsset = $assetSelection.ChecksumAsset
$packageMetadata = $assetSelection.PackageMetadata
$checksumMetadata = $assetSelection.ChecksumMetadata
$installLayout = $assetSelection.InstallLayout
$tempDir = Join-Path ([System.IO.Path]::GetTempPath()) ("codex-install-" + [System.Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force -Path $tempDir | Out-Null

try {
    Invoke-WithInstallLock -LockPath $lockPath -Script {
        Remove-StaleInstallArtifacts -ReleasesDir $releasesDir

        if (-not (Test-ReleaseIsComplete -ReleaseDir $releaseDir -ExpectedVersion $resolvedVersion -ExpectedTarget $target -Layout $installLayout)) {
            if (Test-Path -LiteralPath $releaseDir) {
                Write-WarningStep "Found incomplete existing release at $releaseDir. Reinstalling."
            }

            $archivePath = Join-Path $tempDir $packageAsset
            $checksumPath = Join-Path $tempDir $checksumAsset
            $stagingDir = Join-Path $releasesDir ".staging.$releaseName.$PID"

            Write-Step "Downloading Corbanu Terminal"
            if ($installLayout -eq "Package") {
                Invoke-WebRequestWithFallback -Metadata $checksumMetadata -OutFile $checksumPath -ExpectedDigest $checksumMetadata.Sha256 -AssetName $checksumAsset -ReleaseVersion $resolvedVersion -RequiredManifestAsset $packageAsset
                $expectedPackageDigest = Get-PackageArchiveDigest -ManifestPath $checksumPath -AssetName $packageAsset
            } else {
                $expectedPackageDigest = $packageMetadata.Sha256
            }
            Invoke-WebRequestWithFallback -Metadata $packageMetadata -OutFile $archivePath -ExpectedDigest $expectedPackageDigest -AssetName $packageAsset -ReleaseVersion $resolvedVersion

            New-Item -ItemType Directory -Force -Path $releasesDir | Out-Null
            if (Test-Path -LiteralPath $stagingDir) {
                Remove-Item -LiteralPath $stagingDir -Recurse -Force
            }
            New-Item -ItemType Directory -Force -Path $stagingDir | Out-Null
            if ($installLayout -eq "Package") {
                Expand-WindowsPackageArchive -ArchivePath $archivePath -DestinationPath $stagingDir
                Ensure-CorbanuExecutables -PackageDir $stagingDir -Layout $installLayout
                if (-not (Test-PackageContentsAreComplete -PackageDir $stagingDir)) {
                    throw "Downloaded Corbanu Terminal package archive did not contain the expected package layout."
                }
            } else {
                $extractDir = Join-Path $tempDir "extract"
                New-Item -ItemType Directory -Force -Path $extractDir | Out-Null
                tar -xzf $archivePath -C $extractDir

                $vendorRoot = Join-Path $extractDir "package/vendor/$target"
                $resourcesDir = Join-Path $stagingDir "codex-resources"
                New-Item -ItemType Directory -Force -Path $resourcesDir | Out-Null
                $copyMap = @{
                    "codex/codex.exe" = "pfterminal.exe"
                    "codex/codex-command-runner.exe" = "codex-resources\codex-command-runner.exe"
                    "codex/codex-windows-sandbox-setup.exe" = "codex-resources\codex-windows-sandbox-setup.exe"
                    "path/rg.exe" = "codex-resources\rg.exe"
                }

                foreach ($relativeSource in $copyMap.Keys) {
                    Copy-Item -LiteralPath (Join-Path $vendorRoot $relativeSource) -Destination (Join-Path $stagingDir $copyMap[$relativeSource])
                }

                Ensure-CorbanuExecutables -PackageDir $stagingDir -Layout $installLayout

                if (-not (Test-LegacyPlatformNpmContentsAreComplete -PackageDir $stagingDir)) {
                    throw "Downloaded Corbanu Terminal npm archive did not contain the expected legacy platform package layout."
                }
            }

            if (Test-Path -LiteralPath $releaseDir) {
                Remove-Item -LiteralPath $releaseDir -Recurse -Force
            }
            Move-Item -LiteralPath $stagingDir -Destination $releaseDir
        }

        if (-not (Test-ReleaseIsComplete -ReleaseDir $releaseDir -ExpectedVersion $resolvedVersion -ExpectedTarget $target -Layout $installLayout)) {
            throw "Installed Corbanu Terminal command did not report expected version $resolvedVersion."
        }

        New-Item -ItemType Directory -Force -Path $standaloneRoot | Out-Null
        Ensure-Junction -LinkPath $currentDir -TargetPath $releaseDir -InstallerOwnedTargetPrefix $releasesDir

        $visibleParent = Split-Path -Parent $visibleBinDir
        $currentBinDir = if ($installLayout -eq "Package") {
            Join-Path $currentDir "bin"
        } else {
            $currentDir
        }
        New-Item -ItemType Directory -Force -Path $visibleParent | Out-Null
        $oldStandaloneBackup = Move-OldStandaloneBinIfApproved -VisibleBinDir $visibleBinDir -DefaultVisibleBinDir $defaultVisibleBinDir
        try {
            Ensure-Junction -LinkPath $visibleBinDir -TargetPath $currentBinDir -InstallerOwnedTargetPrefix $standaloneRoot
            Test-VisibleTerminalCommands -VisibleBinDir $visibleBinDir
        } catch {
            if ($null -ne $oldStandaloneBackup -and (Test-Path -LiteralPath $oldStandaloneBackup)) {
                if (Test-Path -LiteralPath $visibleBinDir) {
                    Remove-Item -LiteralPath $visibleBinDir -Recurse -Force
                }
                Move-Item -LiteralPath $oldStandaloneBackup -Destination $visibleBinDir
            }
            throw
        }
        if ($null -ne $oldStandaloneBackup) {
            Remove-Item -LiteralPath $oldStandaloneBackup -Recurse -Force
        }
        Remove-OldStandaloneReleases -ReleasesDir $releasesDir -CurrentDir $currentDir -Keep $KeepReleases
    }
} finally {
    Remove-Item -Recurse -Force $tempDir -ErrorAction SilentlyContinue
}

Maybe-HandleConflictingInstall -Conflict $conflictingInstall

$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
$prioritizeVisibleBin = $null -ne $conflictingInstall
if ($prioritizeVisibleBin) {
    $newUserPath = Prepend-PathEntry -PathValue $userPath -Entry $visibleBinDir
    if ($newUserPath -cne $userPath) {
        [Environment]::SetEnvironmentVariable("Path", $newUserPath, "User")
        Write-Step "PATH updated for future PowerShell sessions."
    } else {
        Write-Step "$visibleBinDir is already first on PATH."
    }
} elseif (-not (Path-Contains -PathValue $userPath -Entry $visibleBinDir)) {
    if ([string]::IsNullOrWhiteSpace($userPath)) {
        $newUserPath = $visibleBinDir
    } else {
        $newUserPath = "$visibleBinDir;$userPath"
    }

    [Environment]::SetEnvironmentVariable("Path", $newUserPath, "User")
    Write-Step "PATH updated for future PowerShell sessions."
} elseif (Path-Contains -PathValue $env:Path -Entry $visibleBinDir) {
    Write-Step "$visibleBinDir is already on PATH."
} else {
    Write-Step "PATH is already configured for future PowerShell sessions."
}

if ($prioritizeVisibleBin) {
    $env:Path = Prepend-PathEntry -PathValue $env:Path -Entry $visibleBinDir
} elseif (-not (Path-Contains -PathValue $env:Path -Entry $visibleBinDir)) {
    if ([string]::IsNullOrWhiteSpace($env:Path)) {
        $env:Path = $visibleBinDir
    } else {
        $env:Path = "$visibleBinDir;$env:Path"
    }
}

Write-Step "Current PowerShell session: corbanu"
Write-Step "Future PowerShell windows: open a new PowerShell window and run: corbanu"
Write-Host "Corbanu Terminal $resolvedVersion installed successfully."

$corbanuCommand = Join-Path $visibleBinDir "corbanu.exe"
if (Prompt-YesNo "Start Corbanu Terminal now?") {
    Write-Step "Launching Corbanu Terminal"
    & $corbanuCommand
}
