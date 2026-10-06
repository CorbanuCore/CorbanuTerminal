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

## Preconditions

- [ ] Plan active; PF-24-S01 archived (done); exact worktree/branch/base and disjoint `write_scope` recorded.
- [ ] Read root, `codex-rs`, TUI and TUI style AGENTS.md.

## Done

- [x] Sprint record created and linked to PF-24 (2026-10-06).

## Remaining

- [ ] Add the `security_levels` flag; with it off, `/security` and every other path is unchanged (snapshot proof).
- [ ] With it on, list Permissive (current) and Aggressive; show Moderate as “not available yet”.
- [ ] Before confirmation, show the exact differences from the table above; `Esc` changes nothing.
- [ ] Apply through the PF-83 confirmed path; show the level as active only after the applied outcome.
- [ ] Persist the level; restart restores it; an unknown stored value fails visibly, never Permissive.
- [ ] Return to Permissive restores the saved prior settings exactly and invalidates incompatible pending approvals.
- [ ] No agent tool, prompt, config overlay or project file can change the level.
- [ ] Regressions: flag off, cancel, confirm, restart, unknown value, child inheritance, vault/network/outside-write probes.
- [ ] Code-blind design for the Aggressive milestone frozen before results are disclosed.

## Verification

- [ ] `cd codex-rs && just fmt && just fix -p codex-tui && just fix -p codex-features`, then focused `just test -p codex-tui security` and `just test -p codex-features`.
- [ ] tmux functional run, product on GLM 5.2 (`-m glm-5.2 -c model_provider="zai"`): flag off, select/cancel, select Aggressive, probes denied, restart, back to Permissive.
- [ ] One independent Opus 5.5 High review; findings dispositioned.
- [ ] Short videos of each core feature end to end (`qa/demos/README.md` SOP, else asciinema `.cast`).

## Exit evidence

- [ ] Commit, changed paths, test output, tmux keys and `.cast` files under `qa/security-levels/sprints/PF-24-S03/`.
- [ ] Merged behind the flag; milestone code-blind run and human sign-off stay with “Aggressive ships”.
- [ ] Done/Remaining reflect reality; completed record archived.
