# Configure a fast drive for Windows CI jobs.
#
# GitHub-hosted Windows runners do not always expose a secondary D: volume or
# the PowerShell support required to create a Dev Drive. Prefer a Dev Drive for
# build I/O, but fall back to an existing runner volume when the image cannot
# provide one. Dev Drive availability is a performance optimization, not a
# correctness requirement.

function Test-DevDrive {
    param([string]$Drive)

    & fsutil devdrv query $Drive *> $null
    return $LASTEXITCODE -eq 0
}

function Invoke-BestEffort {
    param([scriptblock]$Script, [string]$Description)

    try {
        & $Script
    } catch {
        Write-Warning "$Description failed: $($_.Exception.Message)"
    }
}

function Get-FallbackDrive {
    if (Test-Path "D:\") {
        return "D:"
    }

    $RunnerTempDrive = Split-Path -Path $env:RUNNER_TEMP -Qualifier
    if ([string]::IsNullOrWhiteSpace($RunnerTempDrive)) {
        throw "RUNNER_TEMP does not identify a Windows drive."
    }

    return $RunnerTempDrive.TrimEnd("\")
}

function Test-DevDriveProvisioningSupport {
    $NewVhd = Get-Command New-VHD -ErrorAction SilentlyContinue
    $FormatVolume = Get-Command Format-Volume -ErrorAction SilentlyContinue

    return $null -ne $NewVhd -and
        $null -ne $FormatVolume -and
        $FormatVolume.Parameters.ContainsKey("DevDrive")
}

if ((Test-Path "D:\") -and (Test-DevDrive "D:")) {
    Write-Output "Using existing Dev Drive at D:"
    $Drive = "D:"
} else {
    if (-not (Test-DevDriveProvisioningSupport)) {
        $Drive = Get-FallbackDrive
        Write-Warning "This Windows image cannot provision a Dev Drive; using $Drive for CI build storage."
    } elseif (Test-Path "D:\") {
        Write-Output "Existing D: volume is not a Dev Drive; provisioning a new Dev Drive VHD."
    } else {
        Write-Output "No D: volume is available; provisioning a new Dev Drive VHD."
    }

    if ([string]::IsNullOrWhiteSpace($Drive)) {
        try {
            $VhdPath = Join-Path $env:RUNNER_TEMP "codex-dev-drive.vhdx"
            $SizeBytes = 64GB

            if (Test-Path $VhdPath) {
                Remove-Item -Path $VhdPath -Force
            }

            New-VHD -Path $VhdPath -SizeBytes $SizeBytes -Dynamic -ErrorAction Stop | Out-Null
            $Mounted = Mount-VHD -Path $VhdPath -Passthru -ErrorAction Stop
            $Disk = $Mounted | Get-Disk -ErrorAction Stop
            $Disk | Initialize-Disk -PartitionStyle GPT -ErrorAction Stop
            $Partition = $Disk | New-Partition -AssignDriveLetter -UseMaximumSize -ErrorAction Stop
            $Volume = $Partition | Format-Volume -FileSystem ReFS -NewFileSystemLabel "CodexDevDrive" -DevDrive -Confirm:$false -Force -ErrorAction Stop

            $Drive = "$($Volume.DriveLetter):"

            if (-not (Test-DevDrive $Drive)) {
                throw "Provisioned volume at $Drive did not pass Dev Drive verification."
            }

            Invoke-BestEffort { fsutil devdrv trust $Drive } "Trusting Dev Drive $Drive"
            Invoke-BestEffort { fsutil devdrv enable /disallowAv } "Disabling AV filter attachment for Dev Drives"

            Write-Output "Using Dev Drive at $Drive"
        } catch {
            $Drive = Get-FallbackDrive
            Write-Warning "Dev Drive provisioning failed; using $Drive for CI build storage: $($_.Exception.Message)"
        }
    }
}

"CI_BUILD_ROOT=$Drive" | Out-File -FilePath $env:GITHUB_ENV -Encoding utf8 -Append
