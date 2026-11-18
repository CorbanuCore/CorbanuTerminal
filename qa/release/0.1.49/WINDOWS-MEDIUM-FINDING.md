# Windows Medium-integrity preflight failure

2026-11-18 UTC. **Observed failure; final release-candidate replay pending.**
The existing native Windows read/write preflight passed from the SSH launcher's
High-integrity token. That does not qualify a normal Medium-integrity launcher.
A subsequent run through the repository's existing
`.github/scripts/run-at-medium-integrity.ps1` reached the TUI but could not run
its harmless file-read command.

## Reproduction and evidence

- Host: DESKTOP-UJS3HC4; native MSVC executable under the existing MSYS2/tmux
  harness, source `5fd571822bef255237f963c7bc18286a9110f58b` (main baseline
  `d3fe32513ee65deb397fa228c786212fb5afacd3` plus documentation only).
- Executable SHA-256:
  `5e5173876cc2c300eaaac7f3143bdfc9be8bd88396f7d470da15d3a2d160c011`.
- Fresh private profile/OS home, existing private AppData directories, known-
  folder lookup verified inside that home, debug native-keyring guard, custom
  local MLX provider without credentials. No UAC action or host policy change.
- The existing Medium-token helper lowers integrity, makes Administrators and
  local-administrator membership deny-only, and assigns the user as token owner.
  The child harness independently measured `S-1-16-8192` and both deny-only
  administrator groups before starting the native TUI. See the raw result.
- In onboarding, choose the **non-admin** sandbox. Submit the frozen task to
  read `input.json` and write the marker/sum JSON result. `Get-Content input.json`
  repeatedly fails before execution:

  ```text
  windows sandbox: label <private LocalAppData>\CorbanuTerminalSandbox Medium
  ```

- No result file was produced; the case reached its 240-second cutoff (240.39s
  measured). The otherwise equivalent High-launcher replay passed in 14.58s.
- Two earlier Medium-helper launches stopped before starting the TUI because
  MSYS converted `whoami /groups` into a filesystem argument. Those are harness
  initialization failures, not product failures. The actual reproduction sets
  `MSYS2_ARG_CONV_EXCL=*` in the parent before probing the child token.

## Current-main relevance and limits

Fetched main at inspection was
`5ddb26bb2651cddb8965fa0bc27a37c78c2135b1`. There was no diff in
`codex-rs/windows-sandbox-rs/src/` between the tested baseline and that main.
The displayed failure comes from `integrity_leases.rs::ensure_registry`, which
calls `integrity.rs::label_private_dir_medium` for the private records folders.
The underlying Win32 cause has not been established; the TUI reports the outer
labeling context. This is a reproduction through the repository's CI-style
Medium token, not a claim that every existing Windows installation fails.

Open issue/PR searches for Medium/non-admin labeling and
`CorbanuTerminalSandbox` found adjacent security follow-ups (#1542, #1896), but
no matching fresh-registry startup failure. No production fix is made in this
release-preparation PR. The coordinator has been notified through the worker
request/result files. The original release merge gates are unchanged by this
worker; repeat the case on the final branch and disclose its actual result.

Raw result: [live-windows-medium-failure.json](preflight/live-windows-medium-failure.json).
Raw TUI capture and rollout: `C:\CorbanuQA\astra-rel149\live-medium3\`;
local capture copy: `$W/astra-rel149-windows-live-evidence/medium-task.txt`.
Launcher/harness and failure logs: `$W/astra-rel149-medium-live.ps1`,
`astra-rel149-windows-live.py`, `astra-rel149-windows-medium3-live.log`.
Earlier launcher attempts retain their separate logs and versioned harnesses.
