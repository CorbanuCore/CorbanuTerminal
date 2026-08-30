# Windows Podman evidence

Date: 2026-08-29 (America/Phoenix)

The supplied Windows host was reached over its pinned SSH host identity. The
official `RedHat.Podman` winget package was installed only after the user
explicitly authorized an engine install. Winget selected version 5.8.3 from the
official source, downloaded the upstream GitHub release installer, and winget
verified the downloaded installer against the package manifest's declared
SHA-256
`b87fe41c112062b3f598574ed4aa3fb82aaa4bc150b29eb4915400e59b0b6b55`
before installation.

An isolated rootless WSL machine named `corbanu-pf31` was created with one CPU,
2 GiB machine memory and a 20 GiB virtual disk. No pre-existing Podman machine,
image or container existed, and no unrelated resource was changed. This is
developer qualification infrastructure, not product auto-install behavior.

## Portable contract suite

Final corrected-candidate bundle SHA-256:
`5753260131b1c4fada152843a9ed792bbc5edb4b6d3e0634e90ee49d05661c0b`

The bundle was transferred only after the SSH server's ED25519 fingerprint
matched the previously pinned
`SHA256:AL/bFpl1DX/lsk94alfnPXX20Fgx+PgIXyzsF9FATZ0` identity. It was extracted
under the isolated user-local directory
`%LOCALAPPDATA%\Temp\pf31-final-r3-57532601`; the remote SHA-256 matched
before execution.

```text
Microsoft Windows 11 Pro
Version 10.0.26200, build 26200, 64-bit
Python 3.13.15
manifest: valid (qa\security-levels\retriever\artifact-manifest.json)
fixtures: 27 passed; 27 deterministic replay checks passed
manifest policies: 5 mutation checks passed
evidence paths: 10 checks passed
```

## Exact-image probe

Image:
`ghcr.io/d4vinci/scrapling@sha256:1bacbc8ec90b3090d462e12f6555e241daf0dfeb684ab326ffa09d52d8226e69`

The final launch used Podman client 5.8.3 / Linux server 5.8.6 and these bounds:

```text
--network none
--cpus 1
--memory 1g
--memory-swap 1g
--pids-limit 256
--read-only
--tmpfs /tmp:rw,size=256m
```

The container opened Chromium `151.0.7922.34` at `about:blank`, slept without
network access, and exited zero. A five-second cgroup-v2 interval after 20
seconds idle reported:

```text
idle CPU: 0.09004%
memory.current: 485,142,528 bytes (462.67 MiB)
pids.current: 71
published ports: none
Podman image storage: 1,647,867,740 bytes
```

Container inspection confirmed `NetworkMode=none`, `ReadonlyRootfs=true`,
`Memory=1073741824`, `MemorySwap=1073741824`, `NanoCpus=1000000000`,
`PidsLimit=256`, `PublishAllPorts=false`, an empty port-binding map, and the
256 MiB `/tmp` tmpfs. Each probe container had a fixed `corbanu-pf31-*` name and
`io.corbanu.owner=PF-31` label and was removed after its successful zero exit.
