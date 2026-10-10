---
sprint_id: "PF-84-S03"
title: "Account selection per session, CLI and worker"
status: draft
plan_file: "docs/plans/active/unified-provider-auth.md"
plan_feature: "PF-84"
execution_order: 25
owner: "UNALLOCATED"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-84-S02"
created: 2026-10-08
updated: 2026-10-08
---

# PF-84-S03 — Account selection per session, CLI and worker

## Execution mandate

- Deliver: the account is part of each thread's provider configuration. It is set
  by config, by `corbanu exec --account`, or by the spawn tool, and it is recorded
  for resume. A worker the coordinator spawns can run on a different account than
  the coordinator, both in-process and as an out-of-process tmux worker.
- Precedence: explicit (`--account` / spawn `account`) > session choice >
  `[provider_accounts] <provider> = "<name>"` > `default`. An unknown or removed
  account fails closed with recovery text. It never silently falls back.
- Excludes: `/providers` UI (S04), accounting display (S05).

## Plan linkage

- Plan: [Unified provider onboarding and management](../../../plans/active/unified-provider-auth.md)
- Feature: `PF-84`
- Acceptance advanced: "Worker on another account" and "CLI account".

## Code boundaries

Paths are under `codex-rs/`, at `63ea3d0cbd`.

- CLI: `exec/src/cli.rs:14, 47` (add `--account [<provider>:]<name>`); same flag on the TUI entry.
- Spawn: `core/src/tools/handlers/multi_agents_common.rs:178-198` (child copies the
  parent provider, so it must also copy the account), `302-341` (eligibility and
  model overrides take `account`); `core/src/tools/handlers/multi_agents_v2/spawn.rs`;
  schema in `multi_agents_spec.rs`; `core/src/agent/control/spawn.rs:212-252, 493`.
- Auth child: `model-provider-info/src/lib.rs:1281-1298` passes
  `internal-claude-oauth-token --account <name>` as an argument, not env.
  `login/src/auth/external_bearer.rs:173-192` makes the child inherit env;
  `arg0/src/lib.rs:168-178` makes it resolve the real binary.
- Session metadata/resume: rollout and state thread records store the account name only.

## Preconditions

- [ ] Plan is active.
- [ ] Dependencies are completed.
- [ ] Worktree, branch, and base commit are exact and match the plan.
- [ ] If parallel, owner/lane/scopes are disjoint and the receiving integration gate is recorded.
- [ ] Decision D3 (model-chosen spawn account and security levels) recorded in the plan.

## Done

- [x] Sprint record created and linked to one plan feature.

## Remaining

- [ ] Thread config carries `account`; children inherit it unless overridden.
- [ ] Spawn `account` accepts configured names for the target provider only.
  The spawn event shows provider/account; D3 applies under `/security`.
- [ ] `corbanu exec --account` and `-c provider_accounts.<id>=<name>`.
- [ ] Resume uses the recorded account, or blocks with recovery if it is gone.
- [ ] Code-blind functional design frozen before test-result disclosure, or non-user-facing N/A reason recorded.

## Verification

- [ ] Focused: `just test -p codex-core -p codex-exec -p codex-cli` (spawn, CLI, resume).
- [ ] tmux SOP (`docs/tmuxHarness.md`): coordinator on ZAI `default` spawns a
  subagent with `account = "fake"`. The child gets a 401, the coordinator keeps
  replying, and the parent account is unchanged.
- [ ] Out-of-process: a tmux worker runs `corbanu exec --account fake ...` and
  gets a 401; `--account default` replies. Runs through the installed-style
  wrapper too (S01 fix).
- [ ] GLM run drives the coordinator: `-m glm-5.3-flash -c model_provider="zai"`.
- [ ] Videos: `pf84-spawn-account.toml`, `pf84-exec-account.toml` (`--sprint PF-84-S03 --publish`).
- [ ] One independent review (Opus 5.5 High).

## Security notes

- The model sees account names, never values. No env var carries a credential to children.
- Child isolation is proven with canaries: B's failure never retries on A.
- Disposable homes and `CORBANU_TEST_NO_NATIVE_KEYRING=1` for every candidate run.

## Exit evidence

- [ ] Implementation commit and PR recorded.
- [ ] Test output, tmux logs and video paths linked.
- [ ] Code-blind handoff checker passes, or limited-testing agreement recorded.
- [ ] `Done` and `Remaining` ledgers reflect reality.
- [ ] Completed record moved to `docs/sprints/archive/unified-provider-auth/`.
