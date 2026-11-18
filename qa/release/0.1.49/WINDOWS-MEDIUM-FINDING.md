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
At initial capture the TUI exposed only the outer labeling context. The
[win-2584 investigation, PR #2586](https://github.com/CorbanuCore/CorbanuTerminal/pull/2586)
subsequently measured `ERROR_ACCESS_DENIED` (5): `C:\CorbanuQA` inherits a DACL
that gives the Medium user Modify but not `WRITE_OWNER`, which labeling needs.
A normal user profile gives that user Full control. The worker's live TUI
controls pass on both main and the PR with private AppData and workspace
permissions matching a user profile; the inherited-permission negative case
still fails closed. The PR makes that refusal report its cause and requirements.

Our own [corrected Medium replay](preflight/live-windows-medium-profile-acl.json)
now passes all four live read/write assertions in **17.12 seconds**, using the
same `5fd571822b` binary and existing local model. It measures Medium integrity
and deny-only administrator groups again. A fresh, protected fixture-parent
DACL grants inheritable Full control only to the current user, SYSTEM and
Administrators; AppData and workspace inherit it. Owner and parent/child ACL
captures are preserved at `$W/astra-rel149-windows-{acl-profile,child-acls}.json`.
The original failure remains intact. The temporary private SSH forward was
closed after the run. Final candidate replay is still required after #2586 merges. No host-wide ACL, default sandbox or operator profile is changed. This
is a fixture-permission problem exposed by the newer label registry, not evidence
that normal Windows user profiles cannot run commands.

Open issue/PR searches for Medium/non-admin labeling and
`CorbanuTerminalSandbox` found adjacent security follow-ups (#1542, #1896), but
no matching fresh-registry startup failure. No production fix is made in this
release-preparation PR. The coordinator filed [#2584](https://github.com/CorbanuCore/CorbanuTerminal/issues/2584),
assigned the investigation to win-2584, and added its accepted fixing PR as a
0.1.49 release gate on 2026-11-18. The resulting PR is #2586; this preparation
PR remains the evidence home. After #2586 merges, repeat the measured Medium-token case on the final
release-candidate build and require successful tool execution.

Raw result: [live-windows-medium-failure.json](preflight/live-windows-medium-failure.json).
Raw TUI capture and rollout: `C:\CorbanuQA\astra-rel149\live-medium3\`;
local capture copy: `$W/astra-rel149-windows-live-evidence/medium-task.txt`.
Launcher/harness and failure logs: `$W/astra-rel149-medium-live.ps1`,
`astra-rel149-windows-live.py`, `astra-rel149-windows-medium3-live.log`.
Earlier launcher attempts retain their separate logs and versioned harnesses.
