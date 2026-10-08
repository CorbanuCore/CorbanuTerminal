# Task Node evidence record: P0SEC-TN-07

Compile the P0SEC-TN-07 Security TUI Lane Evidence Record. Task `task_627354898804e21841f81a559fe63462`, request `req_36bf02981ba06ce3ab13933094e79f0c326ee0aa673ec759713634c5e716e776`. Every PR below is merged to `main` in
CorbanuCore/CorbanuTerminal. Each merge commit was checked with `git merge-base --is-ancestor <sha> origin/main`
at `bb609b449a` (2026-10-07). The quoted blocks are copied verbatim from the sprint and gate records (links
re-pointed to this file's location).

Per-sprint gate (Travis, 2026-10-06): focused `just test`, a GLM 5.2 tmux run, one independent Opus 5.5 High
review and SOP demo videos, then merge behind the feature flag. **Not claimed:** the milestone code-blind VM run,
human sign-off, flag removal and the P1 hardening plan. Carried-forward items are recorded as open, not done.

| Sprint | PRs (merge commit on main) | Flag | Record status |
| --- | --- | --- | --- |
| [PF-24-S03](../../sprints/archive/p0-security-levels/pf-24-s03-flagged-security-picker.md) Flagged /security picker | [#186](https://github.com/CorbanuCore/CorbanuTerminal/pull/186) (`023355670a`) | `security_levels` | completed (archived) |
| [PF-24-S02](../../sprints/archive/p0-security-levels/pf-24-s02-security-confirm-cancel-and-downgrade.md) /security confirm, cancel and downgrade | [#253](https://github.com/CorbanuCore/CorbanuTerminal/pull/253) (`e4d17dbdc6`), [#258](https://github.com/CorbanuCore/CorbanuTerminal/pull/258) (`b2d2583e0f`) | `security_levels` | completed (archived) |
| [PF-25-S01](../../sprints/archive/p0-security-levels/pf-25-s01-temporary-grant-tui.md) Temporary grant TUI | [#260](https://github.com/CorbanuCore/CorbanuTerminal/pull/260) (`bb609b449a`) | `security_levels` | completed (archived) |

## PF-24-S03: Flagged /security picker

Summary: tests `just test -p codex-tui -- security` 31/31; `just test -p codex-features` 33/33; PR checks green on 65bc762d41. TUI run: GLM 5.2 tmux run (flag off, select/cancel, Aggressive, probes denied, restart, back to Permissive). Review: Opus 5.5 High: request changes, 12 findings dispositioned.

Gate record: [qa/security-levels/sprints/PF-24-S03/README.md](../../../qa/security-levels/sprints/PF-24-S03/README.md).

Verbatim, sprint record `docs/sprints/archive/p0-security-levels/pf-24-s03-flagged-security-picker.md`, Verification:

> - [x] `just fmt`, `just fix -p codex-tui -p codex-features`; `just test -p codex-tui -- security` 31/31, `just test -p codex-features` 33/33; full `codex-tui` log kept (host-only `SUN_LEN` failures).
> - [x] tmux functional run on GLM 5.2: flag off, select/cancel, select Aggressive, probes denied, restart, back to Permissive (`qa/.../tmux-run/`).
> - [x] Independent Opus 5.5 High review: request changes, 12 findings dispositioned (`qa/.../review/`).
> - [x] Videos with the demo SOP script (PR #177 branch): 9 at `7f6c88912c` (`qa/.../demos/index.md`).
> - [x] PR checks green on head `65bc762d41` (27 pass, 9 skipped) before merge.

Demo videos (18, index [qa/security-levels/sprints/PF-24-S03/demos/index.md](../../../qa/security-levels/sprints/PF-24-S03/demos/index.md)):

- `pf24-aggressive-sandbox-network`: Aggressive: approved command stays sandboxed: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s03-pf24-aggressive-sandbox-network-7f6c88912c14-2026-10-06.mp4
- `pf24-aggressive-vault-env-state`: Aggressive: vault, secrets and level file: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s03-pf24-aggressive-vault-env-state-7f6c88912c14-2026-10-06.mp4
- `pf24-child-inherits`: Aggressive: child agents inherit it: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s03-pf24-child-inherits-7f6c88912c14-2026-10-06.mp4
- `pf24-flag-off`: Flag off: /security unchanged: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s03-pf24-flag-off-7f6c88912c14-2026-10-06.mp4
- `pf24-permissions-blocked`: Aggressive: /permissions cannot undo it: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s03-pf24-permissions-blocked-7f6c88912c14-2026-10-06.mp4
- `pf24-picker-cancel`: Review Aggressive, then cancel: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s03-pf24-picker-cancel-7f6c88912c14-2026-10-06.mp4
- `pf24-return-to-permissive`: Return to Permissive: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s03-pf24-return-to-permissive-7f6c88912c14-2026-10-06.mp4
- `pf24-select-aggressive-restart`: Select Aggressive and restart: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s03-pf24-select-aggressive-restart-7f6c88912c14-2026-10-06.mp4
- `pf24-unknown-value`: Unknown stored level fails closed: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s03-pf24-unknown-value-7f6c88912c14-2026-10-06.mp4
- `pf24-aggressive-sandbox-network`: Aggressive: approved command stays sandboxed: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s03-pf24-aggressive-sandbox-network-72735863b6c0-2026-10-05.mp4
- `pf24-aggressive-vault-env-state`: Aggressive: vault, secrets and level file: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s03-pf24-aggressive-vault-env-state-72735863b6c0-2026-10-05.mp4
- `pf24-child-inherits`: Aggressive: child agents inherit it: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s03-pf24-child-inherits-72735863b6c0-2026-10-05.mp4
- `pf24-flag-off`: Flag off: /security unchanged: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s03-pf24-flag-off-72735863b6c0-2026-10-05.mp4
- `pf24-permissions-blocked`: Aggressive: /permissions cannot undo it: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s03-pf24-permissions-blocked-72735863b6c0-2026-10-05.mp4
- `pf24-picker-cancel`: Review Aggressive, then cancel: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s03-pf24-picker-cancel-72735863b6c0-2026-10-05.mp4
- `pf24-return-to-permissive`: Return to Permissive: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s03-pf24-return-to-permissive-72735863b6c0-2026-10-05.mp4
- `pf24-select-aggressive-restart`: Select Aggressive and restart: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s03-pf24-select-aggressive-restart-72735863b6c0-2026-10-05.mp4
- `pf24-unknown-value`: Unknown stored level fails closed: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s03-pf24-unknown-value-72735863b6c0-2026-10-05.mp4

Carried forward (open, not claimed): Follow-ups listed in the sprint QA README (later PRs #199, #200, #202).

## PF-24-S02: /security confirm, cancel and downgrade

Summary: tests core/security-level 111/111; `just test -p codex-tui` security sets 273/273; wider run 5,920/5,923; Linux clippy on the RTX box. TUI run: GLM 5.2 tmux pass on main e4d17dbdc6. Review: Opus 5.5 High: round 4 APPROVE.

Gate record: [qa/security-levels/sprints/PF-24-S02/README.md](../../../qa/security-levels/sprints/PF-24-S02/README.md).

Verbatim, sprint record `docs/sprints/archive/p0-security-levels/pf-24-s02-security-confirm-cancel-and-downgrade.md`, Verification:

> - [x] `just fix -p codex-tui -p codex-core -p codex-security-level -p codex-cli`; `just fmt`.
> - [x] `just test -p codex-tui` (security, pf_24, pf_29 sets), `just test -p codex-core -p codex-security-level`
>   (`security_transition`, `security_recovery`, `security_confirm`, `pf_29`).
> - [x] Snapshots reviewed and accepted (PF-24 output only).
> - [x] Linux clippy (`-D warnings`) on the RTX box.
> - [x] GLM 5.2 tmux functional pass with videos (confirm, cancel, downgrade, restart now; `qa/demos/index/PF-24-S02.md`).
> - [x] Full isolated code-blind VM run and human sign-off are milestone gates only (Aggressive/Moderate ship, flag removal).

Verbatim, gate record `qa/security-levels/sprints/PF-24-S02/README.md`:

> - **Focused tests (final tree):**
>   - `just test -p codex-core -p codex-security-level` (`security_transition`, `security_recovery`,
>     `security_confirm`, `pf_29`, `pf_23`): 111/111.
>   - `just test -p codex-tui` (`security`, `pf_24`, `pf_29`, `slash_command`): 273/273.
>   - New: `core/src/security/level_change_tests.rs` (11), `security-level/src/level_tests.rs` (5),
>     `tui/src/security/confirm_tests.rs` (6), picker `pf_24_s02` (7), and the launch and restart tests.
>
> - **Wider run** (`just test -p codex-tui -p codex-security-level -p codex-cli`, round-1 tree): 5920/5923. The three
>   failures are in code this sprint doesn't touch:
>   - the command-menu snapshot (known drift);
>   - a kitty pet image test;
>   - `nested_launch::pass_mode_runs_exec_with_aggressive_enforced`, where the exec path reports the test's
>     account-home registry missing. It wasn't compared on main.
>
> - **Linux clippy** (`-D warnings`; core, tui, cli, security-level, app-server-client) on the RTX box: clean at
>   `9b776a204b`, `ff0e076d90`, `3e1ba16e13` and `9fda6ceb5e`; later commits only change messages, comments and argument comments.
>
> - **Review (Opus 5.5 High, installed `corbanu exec`, read-only):**
>   - Round 1: REQUEST CHANGES (10 findings).
>   - Round 2: REQUEST CHANGES (5).
>   - Round 3: APPROVE WITH FIXES (4).
>   - Round 4: **APPROVE**.
>   - Each finding is fixed or recorded in `review/disposition.md`; the rounds are in `review/round1..4.md`.
>
> - **GLM 5.2 tmux pass (main `e4d17dbdc6`, `-m glm-5.2 -c model_provider="zai"`, disposable homes,
>   `CORBANU_TEST_NO_NATIVE_KEYRING=1`; `.codex-work/workers-20261002/sec-tui8.log`):** each case ends with GLM reading
>   a file in the hidden Corbanu home (`../home/team-notes.txt`):
>   - Confirm: read works under Permissive; after `/security` → Aggressive → Enter, "Core's level is Aggressive now in
>     this session", and the same read gets "Operation not permitted" without a restart.
>   - Cancel: Esc on the Aggressive review says "Cancelled. Nothing changed."; the read still works.
>   - Downgrade: from a confirmed and restarted Aggressive, Permissive lists the protections removed at the next start;
>     Enter keeps Core at Aggressive in this session; after `r` the read works again.
>   - Restart now: started with an initial prompt that GLM answered once; `r` restarted with the same options and the
>     prompt was not sent again; `/security` showed Aggressive active, commands now asked first (`untrusted`), and the
>     approved read got "Operation not permitted". The first take was a model slip, recorded as is: GLM rewrote the
>     command (`printf … home`, reading `../hohome/…`), so the run was repeated.
>   - Four GLM videos (below and in `qa/demos/index/PF-24-S02.md`).

Demo videos (8, index [qa/demos/index/PF-24-S02.md](../../../qa/demos/index/PF-24-S02.md)):

- `pf24s02-confirm-aggressive-restart`: Confirm Aggressive, then restart now: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s02-pf24s02-confirm-aggressive-restart-9fda6ceb5e02-2026-10-07.mp4
- `pf24s02-downgrade-shows-removed`: Downgrade shows removed protections first: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s02-pf24s02-downgrade-shows-removed-9fda6ceb5e02-2026-10-07.mp4
- `pf24s02-level-file-tamper`: Level file changed outside /security is caught: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s02-pf24s02-level-file-tamper-9fda6ceb5e02-2026-10-07.mp4
- `pf24s02-save-failure-stays-open`: A failed save changes nothing and stays open: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s02-pf24s02-save-failure-stays-open-9fda6ceb5e02-2026-10-07.mp4
- `pf24s02-glm-cancel`: Esc on the review changes nothing: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s02-pf24s02-glm-cancel-e4d17dbdc6f0-2026-10-07.mp4
- `pf24s02-glm-confirm`: Confirm raises Core's level for GLM's next command: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s02-pf24s02-glm-confirm-e4d17dbdc6f0-2026-10-07.mp4
- `pf24s02-glm-downgrade`: Downgrade lists removed protections and applies at the next start: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s02-pf24s02-glm-downgrade-e4d17dbdc6f0-2026-10-07.mp4
- `pf24s02-glm-restart-now`: Restart now keeps options and does not resend the prompt: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s02-pf24s02-glm-restart-now-e4d17dbdc6f0-2026-10-07.mp4

Carried forward (open, not claimed): Windows restart Ctrl-C, profile-v2 downgrade path, tmux startup warnings.

## PF-25-S01: Temporary grant TUI

Summary: tests core 62/62; tui 164/164; wider tui/app-server-client 4,414/4,416; Linux clippy on the RTX box. TUI run: GLM 5.2 tmux run. Review: Opus 5.5 High: four rounds, round 4 APPROVE.

Gate record: [qa/security-levels/sprints/PF-25-S01/README.md](../../../qa/security-levels/sprints/PF-25-S01/README.md).

Verbatim, sprint record `docs/sprints/archive/p0-security-levels/pf-25-s01-temporary-grant-tui.md`, Verification:

> - [x] `just fix -p codex-core -p codex-tui -p codex-app-server-client`; `just fmt`.
> - [x] `just test -p codex-core` (`grant_offer`, `aggressive`, `pf_23_s02`, `transition`); `just test -p codex-tui`
>   (`pf_25_s01`, `approval_overlay`, `security`).
> - [x] Snapshots reviewed and accepted (PF-25 grant output only).
> - [x] GLM 5.2 tmux run and three videos; Linux clippy on the RTX box; Opus 5.5 High review (four rounds, APPROVE).

Verbatim, gate record `qa/security-levels/sprints/PF-25-S01/README.md`:

> - **Focused tests (final tree):**
>   - `just test -p codex-core` (`grant_offer`, `aggressive`, `pf_25_s01`, `pf_23_s02`, `transition`): 62/62.
>   - `just test -p codex-tui` (`pf_25_s01`, `approval_overlay`, `security`): 164/164.
>   - Core integration (`core/tests/suite/pf_25_s01.rs`, macOS seatbelt): the offer key is the approval id the TUI
>     answers and Core's command equals the approval's; the approved "1 run" reads the protected file and the next
>     identical run, only approved, gets "Operation not permitted"; a declined approval leaves nothing.
>
> - **Wider run** (round-1 tree): `just test -p codex-tui -p codex-app-server-client` 4414/4416. The two failures are
>   the known command-menu snapshot and kitty pet image tests.
>
> - **Linux clippy** (`-D warnings`; core, tui, app-server-client, `--tests`) on the RTX box: clean at `535350b9cf`,
>   `c5fa36b31f` and `d76c577e8c` (at `aee150e46a` it caught an `eprintln!` in a test, removed).
>
> - **Review (Opus 5.5 High, installed `corbanu exec`, read-only):** round 1 APPROVE WITH FIXES (9 findings), round 2
>   APPROVE WITH FIXES (8 low), round 3 APPROVE WITH FIXES (1 medium, 5 low/info), round 4 **APPROVE**. Each finding
>   is fixed or recorded in `review/disposition.md`.

Demo videos (3, index [qa/demos/index/PF-25-S01.md](../../../qa/demos/index/PF-25-S01.md)):

- `pf25s01-grant-once`: Grant one run of a denied command: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-25-s01-pf25s01-grant-once-aee150e46ae7-2026-10-07.mp4
- `pf25s01-grant-esc`: Esc in the grant review grants nothing: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-25-s01-pf25s01-grant-esc-aee150e46ae7-2026-10-07.mp4
- `pf25s01-grant-until-expiry`: A grant until expiry is listed in /security: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-25-s01-pf25s01-grant-until-expiry-aee150e46ae7-2026-10-07.mp4

Carried forward (open, not claimed): Revocation and kill switch to PF-25-S02 (PR #261, not merged).
