PF-27-S08 is complete. Here's the summary:

## What shipped (PR #333, merged)

The Windows credential broker now runs under its own **low-integrity, write-restricted token** (not an AppContainer), behind the existing default-off flags. Permissive, macOS and Linux unchanged.

**This relaunch** recovered the previous Opus run (filtered at the end while writing the evidence README), then fixed the two blocking review items and key non-blocking ones:
- **B1:** the process-access probe now tests 9 additional rights; measured exceptions are `PROCESS_QUERY_LIMITED_INFORMATION` and `PROCESS_TERMINATE` granted (execute-class), thread terminate denied.
- **B2:** COM limit correctly described — low integrity is the barrier, not deny-only INTERACTIVE.
- **CredWrite/CredDelete** now measured; pipe label documented; code nits fixed. The review's suggestion to remove `TOKEN_ASSIGN_PRIMARY`/`TOKEN_ADJUST_DEFAULT` was wrong (needed for `CreateProcessAsUserW` and `set_default_dacl`) — restored.

## Gate evidence
- **Real Windows 11** (`dbf8be6df5`): process-hardening `pf_27_s08` 7/7 elevated + 7/7 normal session; network-proxy 25/25 both.
- **CI:** 16 passed, 0 failed, 5 long-running builds stuck pending (stalled runners, no required checks).
- **Clippy:** clean on Windows and macOS.
- **Demo videos:** leak-scanned, uploaded to `demos` release, indexed in `qa/demos/index/PF-27-S08.md`.

## Open for Travis
The vault key path: the broker token can read/write/delete Credential Manager (measured). Option (c) ships (broker reads it itself, like macOS/Linux). Acceptance 5's "Credential Manager denied" is not met until Travis decides.

## Cleanup
Windows: removed `C:\CorbanuQA\s08`, secrets, sandbox-template, 15 scheduled tasks. Mac: worktree and branch removed. Result file: `sec-s08.result.md`.