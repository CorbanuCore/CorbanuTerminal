# Diagnose a Windows upgrade that still runs an older binary

If an upgrade refuses to retarget a junction, or your terminal still opens an old
version, collect the command path and junction target before changing anything.
These checks inspect your installation; they do not repair or remove it.

Product specification: **Shipping MVP — LIVE**, “the `corbanu` command, and legacy
`pfterminal` command and state compatibility.”

## Check which command PowerShell resolves

In your usual PowerShell session, run:

```powershell
Get-Command corbanu,pfterminal -All -ErrorAction SilentlyContinue |
    Select-Object CommandType, Name, Source, Definition
```

For each name, `-All` lists matches in execution precedence order. An alias or
function can precede an executable; multiple application paths can reveal an old
installation shadowing a newer one. No result means the named commands were not
found in this session, not that no installation exists.

After confirming that the resolved command belongs to your trusted installation,
run `corbanu --version` (or `pfterminal --version` for the legacy command). Record
the command name, resolved path and reported version together. Do not execute an
unfamiliar binary just because it appears in the results.

## Inspect the visible bin junction

Copy the exact bin-directory path from the installer error into this placeholder:

```powershell
$diagnosticBin = 'C:\path\from\the\installer\error\bin'
Get-Item -LiteralPath $diagnosticBin -Force |
    Select-Object FullName, Attributes, LinkType, Target
```

For a junction, expect `LinkType` to be `Junction` and `Target` to identify its
destination. An ordinary directory, missing path or unreadable target is a
different finding: preserve the error rather than guessing a target. Use the
actual error path, especially with a custom install directory.

The installer supports visible bin directories under both
`%LOCALAPPDATA%\Programs\Corbanu Terminal\bin` and the legacy
`%LOCALAPPDATA%\Programs\PFTerminal\bin`. Environment overrides can select a
different directory. The visible bin directory and the package storage root are
different paths.

## Interpret a refusal safely

`Ensure-Junction` in [the installer](../scripts/install/install.ps1) refuses an
existing junction whose target falls outside its selected installer-owned prefix.
A junction left by an older home layout can trigger that refusal. For example,
a target under `.codex\packages\standalone` and an installer using another home
are evidence of a possible layout mismatch, not proof that deleting the old home
is safe.

Compare the inspected target with the package location reported by the installer.
Do not assume `.pfterminal` or `.corbanu` is authoritative for every machine:
existing directories and home/install overrides affect selection. A target that
still exists may keep an old binary runnable, so successful command execution
alone does not establish that the upgrade took effect.

Keep the ownership guard in place. Do not delete the junction, edit PATH, or move
credential/state directories as part of these diagnostics. Ask the maintainer for
a supported recovery for the observed paths and version.

## Share a minimal report

Use this template in an issue or support reply:

```text
Windows version and PowerShell version:
Previous install and attempted release version:
Installer command/source URL:
Exact error message (personal paths redacted):
Command name, type and resolved path:
Reported --version output:
Bin directory LinkType and Target:
Custom home/install overrides used (names and redacted paths only):
Expected result and actual result:
```

Replace usernames and private directory names with placeholders while preserving
which paths refer to the same location. Do not attach environment dumps, full
configuration files, terminal-session storage, wallet seeds or credentials.

## References and validation limits

- [Microsoft: Get-Command](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/get-command)
- [Microsoft: Get-Item](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.management/get-item)
- [Installer source](../scripts/install/install.ps1), especially `Ensure-Junction`
  and visible-bin/home selection.

This guide was checked against public installer source and PowerShell command
documentation. Its commands were not executed on a Windows installation during
preparation; it does not claim a reproduced migration or a tested repair.
