# Task Node evidence record: P0SEC-TN-04

Compile the P0SEC-TN-04 Protected-Mode Sprint Evidence Record. Task `task_8ce36190b7954b6d5d4f1bcd691886cc`, request `req_8d71397c5c9650ec18a81ecd6c45a4be8ef6d230a1ff1065fd848e2529757f86`. Every PR below is merged to `main` in
CorbanuCore/CorbanuTerminal. Each merge commit was checked with `git merge-base --is-ancestor <sha> origin/main`
at `bb609b449a` (2026-10-07). The quoted blocks are copied verbatim from the sprint and gate records (links
re-pointed to this file's location).

Per-sprint gate (Travis, 2026-10-06): focused `just test`, a GLM 5.2 tmux run, one independent Opus 5.5 High
review and SOP demo videos, then merge behind the feature flag. **Not claimed:** the milestone code-blind VM run,
human sign-off, flag removal and the P1 hardening plan. Carried-forward items are recorded as open, not done.

| Sprint | PRs (merge commit on main) | Flag | Record status |
| --- | --- | --- | --- |
| [PF-29-S01](../../sprints/current/p0-security-levels/pf-29-s01-protected-mode-inventory.md) Protected-mode inventory and activation preflight | [#228](https://github.com/CorbanuCore/CorbanuTerminal/pull/228) (`b751254169`) | `protected_mode_preflight` | merged behind flag; record current (milestone items open) |
| [PF-29-S02](../../sprints/current/p0-security-levels/pf-29-s02-human-secret-migration.md) Human-reviewed credential migration and recovery | [#235](https://github.com/CorbanuCore/CorbanuTerminal/pull/235) (`65d42158d7`) | `protected_mode_preflight` | merged behind flag; record current (milestone items open) |

## PF-29-S01: Protected-mode inventory and activation preflight

Summary: tests focused `pf_29_s01`: core 14, tui 8; PR CI green. TUI run: GLM 5.2 runs. Review: three Opus 5.5 High reviews, all findings handled.

Gate record: [qa/security-levels/sprints/PF-29-S01/README.md](../../../qa/security-levels/sprints/PF-29-S01/README.md).

Verbatim, sprint record `docs/sprints/current/p0-security-levels/pf-29-s01-protected-mode-inventory.md`, Verification:

> - [x] `just fix -p codex-features -p codex-core -p codex-app-server-client -p codex-tui`, `just fmt`; diff inspected.
> - [x] Focused `pf_29_s01`: core 14, tui 8. Suites, five GLM 5.2 videos, three Opus reviews (all findings handled).
> - [x] PR CI green; merged as #228. Details in the [evidence](../../../qa/security-levels/sprints/PF-29-S01/README.md).
> - [ ] PF-26 final-candidate requalification (milestone gate).

Verbatim, gate record `qa/security-levels/sprints/PF-29-S01/README.md`:

> - Suites: `just test -p codex-tui` 4267 passed, 21 failed: 20 bind Unix sockets under this worktree's long temp
>   path ("path must be shorter than SUN_LEN": IDE IPC, daemon, wallet onboarding) and one upstream-branding snapshot
>   (`command_popup_default_items`), all unrelated and failing the same way without this change. `cargo test -p
>   codex-core --lib security::` 161 passed. `just test -p codex-features` 33 passed. `just test -p
>   codex-app-server-client` 37 passed, 1 failed (same SUN_LEN socket).
>
> - Review 3 P2 stale level in an open picker: the saved level and receipt are read again at save; a saved Aggressive
>   without a receipt needs the preflight (test). P2 glob syntax in the home path: exact database paths instead (test).
>   P2 Linux: bubblewrap masks only paths that exist when a command starts, so a `-wal`/`-shm` file created during a
>   long command is readable by that command; store folders are now created at launch, and the limit is recorded below.
>   P3 canonical paths: the check fails closed (launch refuses); left as is.

Demo videos (5, index [qa/demos/index/PF-29-S01.md](../../../qa/demos/index/PF-29-S01.md)):

- `pf29s01-blocked`: Preflight blocks Aggressive while a raw-secret route stays open: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-29-s01-pf29s01-blocked-df875e9f123c-2026-10-06.mp4
- `pf29s01-flag-off`: Preflight flag off: Aggressive behaves exactly as before: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-29-s01-pf29s01-flag-off-df875e9f123c-2026-10-06.mp4
- `pf29s01-isolated-after-restart`: Preflight passes and Aggressive isolates the SSH private key after restart: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-29-s01-pf29s01-isolated-after-restart-df875e9f123c-2026-10-06.mp4
- `pf29s01-launch-reaudit`: Launch re-audits the boundary instead of claiming it: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-29-s01-pf29s01-launch-reaudit-df875e9f123c-2026-10-06.mp4
- `pf29s01-resume-refused`: Conversations recorded before activation cannot be resumed: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-29-s01-pf29s01-resume-refused-df875e9f123c-2026-10-06.mp4

Carried forward (open, not claimed): MCP/hooks/notify listed as not contained; exec-provider sign-in consent; `-C` folder `.env` reported not denied.

## PF-29-S02: Human-reviewed credential migration and recovery

Summary: tests focused `pf_29_s02`: core 6, tui 4 (with PF-29-S01: core 20, tui 12); CI green. TUI run: GLM 5.2 runs. Review: Opus 5.5 High: changes, then approved.

Gate record: [qa/security-levels/sprints/PF-29-S02/README.md](../../../qa/security-levels/sprints/PF-29-S02/README.md).

Verbatim, sprint record `docs/sprints/current/p0-security-levels/pf-29-s02-human-secret-migration.md`, Verification:

> - [x] `just fix -p codex-core -p codex-tui`, `just fmt`; final diff inspected.
> - [x] Focused `pf_29_s02`: core 6, tui 4 (with the PF-29-S01 tests: core 20, tui 12).
> - [x] Suites, three GLM 5.2 videos, Opus review (changes, then approved); CI green; merged as #235 ([evidence](../../../qa/security-levels/sprints/PF-29-S02/README.md)).
> - [ ] PF-26 final-candidate requalification (milestone gate).

Verbatim, gate record `qa/security-levels/sprints/PF-29-S02/README.md`:

> - Suites after merging main: `cargo test -p codex-core --lib security::` 176 passed; `just test -p codex-tui`
>   4288 passed, 21 failed. The failures are the same as PF-29-S01's: Unix-socket paths too long for this worktree
>   (SUN_LEN) and an upstream-branding snapshot, all unrelated.

Demo videos (3, index [qa/demos/index/PF-29-S02.md](../../../qa/demos/index/PF-29-S02.md)):

- `pf29s02-failure-recovery-restart`: Interrupted migration locks Aggressive; after restart, recovery finishes it: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-29-s02-pf29s02-failure-recovery-restart-821c0b8f7ef3-2026-10-06.mp4
- `pf29s02-migrate-and-save`: Confirmed migration moves the value into the vault, re-audits and saves Aggressive: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-29-s02-pf29s02-migrate-and-save-821c0b8f7ef3-2026-10-06.mp4
- `pf29s02-preview-cancel`: Migration preview, then Esc: nothing changes: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-29-s02-pf29s02-preview-cancel-821c0b8f7ef3-2026-10-06.mp4

Carried forward (open, not claimed): Only shell-profile exports migrated; live broker leases not revoked on migration.
