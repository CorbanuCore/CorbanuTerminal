---
sprint_id: "PF-24-S03"
title: "Flagged /security picker: Permissive and Aggressive"
status: in_progress
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-24"
execution_order: 20
owner: "TUI lane worker (codex, 2026-10-06)"
parallel_lane: "tui"
write_scope: "codex-rs/tui/src/security, codex-rs/tui/src/bottom_pane/security_view.rs, codex-rs/tui/src/bottom_pane/security_view_tests.rs, codex-rs/tui/src/bottom_pane/security_level_picker.rs, codex-rs/tui/src/bottom_pane/security_level_picker_tests.rs, codex-rs/tui/src/bottom_pane/snapshots, codex-rs/tui/src/bottom_pane/mod.rs, codex-rs/tui/src/app/permission_confirmation.rs, codex-rs/tui/src/app/config_persistence.rs, codex-rs/tui/src/lib.rs, codex-rs/features/src/lib.rs, codex-rs/core/config.schema.json, qa/security-levels/sprints/PF-24-S03"
integration_gate: "PR to main, per-sprint gate (sec-common decision 5)"
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf-24-s03-security-picker"
branch: "codex/pf-24-s03-security-picker"
base_commit: "24242c1b0ee5388cbfbfe1b0d8f63459945b588d"
depends_on: "PF-24-S01"
created: 2026-10-06
updated: 2026-10-06
---

# PF-24-S03 — Flagged /security picker: Permissive and Aggressive

## Execution mandate

- Deliver: behind the `security_levels` flag, `/security` lets a human choose Permissive
  or Aggressive; Aggressive is built only from controls that already exist.
- Excludes: Moderate (hidden until its protections land), broker/taint enforcement,
  temporary grants, kill switch, new sandbox or policy mechanisms, flag removal.
- First sprint in the TUI lane (Travis, 2026-10-06, decision 2).

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#sprint-execution-map); feature `PF-24`.
- Product citation: **P0 `/security` levels** — “Existing approval, sandbox, vault, wallet, tool, network, and agent policies are unchanged.”
- Acceptance advanced: flag off, Open and cancel, Select Aggressive, Return to Permissive, Agent attempts a change.

## Code boundaries

- Existing: `tui/src/security/`, `tui/src/bottom_pane/security_view.rs`, `tui/src/slash_command.rs`,
  `tui/src/app/permission_confirmation.rs` (PF-83 confirmed-apply path), `features/src/lib.rs`.
- Planned: `Feature::SecurityLevels`, key `security_levels`, `Stage::UnderDevelopment`, default off;
  a persisted `security_level` (`permissive` | `aggressive`) plus the saved pre-Aggressive settings.
- Tests: sibling `*_tests.rs`, `insta` snapshots, app-server settings tests.

### Aggressive mapping onto existing controls

| Control | Aggressive value | Existing mechanism |
| --- | --- | --- |
| Sandbox | `workspace-write`, cwd only: no extra `writable_roots`, `exclude_slash_tmp`, `exclude_tmpdir_env_var` (`read-only` is the stricter alternative) | `SandboxPolicy` |
| Approvals | `approval_policy = "untrusted"`: every command not known-safe read-only asks | `AskForApproval::UnlessTrusted` |
| Network | `network_access = false`; web search disabled | sandbox policy, `web_search` config |
| Vault | `corbanu vault …` forbidden for agent commands; vault env vars stripped | `execpolicy` `decision = "forbidden"`, `shell_environment_policy` |
| Children | spawned agents get the same values | existing config inheritance |

If any row cannot be enforced with existing controls, stop and escalate; do not
label Aggressive as active while a row is missing.

### As built (2026-10-06)

- Sandbox: permission profile `corbanu-aggressive` (`:workspace`, tmp and `$CODEX_HOME` read-only,
  network off, deny-read vault store and `auth.json`). Under `untrusted` a legacy `SandboxPolicy`
  retries an approved, sandbox-blocked command unsandboxed; a denied-read entry forbids that.
- Env row also turns off shell snapshots and login shells (they re-export removed variables).
- Applies at the next start (web search, env and exec policy are session-static, so not via the
  PF-83 path); every config build is verified row by row; `config.toml` is never written.

## Preconditions

- [x] Plan active; PF-24-S01 archived (done); exact worktree/branch/base and disjoint `write_scope` recorded.
- [x] Read root, `codex-rs`, TUI and TUI style AGENTS.md.

## Done

- [x] Sprint record created and linked to PF-24 (2026-10-06).
- [x] `security_levels` flag; flag off and no state file: no change (old view snapshots unchanged; launch no-op test; video `pf24-flag-off`).
- [x] Picker: Permissive, Moderate “not available yet”, Aggressive; review shows every row first; `Esc` changes nothing.
- [x] Active only after the applied outcome: saved level shown as pending until a restart whose loaded config passes verification (see As built for the PF-83-path deviation).
- [x] Persisted in `$CODEX_HOME/security_level.toml`; restart restores it; unknown or corrupt state (or a deleted state file beside the rule file) enforces Aggressive with a warning.
- [x] Return to Permissive removes overlay and rule file (`config.toml` hash unchanged); pending approvals end at restart.
- [x] No agent path: file outside every config layer, agent writes denied; `/permissions`/auto-review refused.
- [x] Regressions: tests, tmux run and 9 videos; code-blind design frozen first (49 cases, `FROZEN.sha256`).

## Remaining

- [ ] Merge behind the flag; then archive this record.

Follow-ups (not this sprint's scope) are listed in `qa/security-levels/sprints/PF-24-S03/README.md`.

## Verification

- [x] `just fmt`, `just fix -p codex-tui -p codex-features`; `just test -p codex-tui -- security` 31/31, `just test -p codex-features` 33/33; full `codex-tui` log kept (host-only `SUN_LEN` failures).
- [x] tmux functional run on GLM 5.2: flag off, select/cancel, select Aggressive, probes denied, restart, back to Permissive (`qa/.../tmux-run/`).
- [x] Independent Opus 5.5 High review: request changes, 12 findings dispositioned (`qa/.../review/`).
- [x] Videos with the demo SOP script (PR #177 branch): 9 at `7f6c88912c` (`qa/.../demos/index.md`).
- [ ] PR checks green on the final head before merge.

## Exit evidence

- [x] Commit, paths, tests, tmux keys and video links in `qa/security-levels/sprints/PF-24-S03/` (casts: release assets).
- [ ] Merged behind the flag; milestone code-blind run and human sign-off stay with “Aggressive ships”.
- [ ] Done/Remaining reflect reality; completed record archived.
