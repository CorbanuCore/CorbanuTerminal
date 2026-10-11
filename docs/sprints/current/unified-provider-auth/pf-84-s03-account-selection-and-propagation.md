---
sprint_id: "PF-84-S03"
title: "Account selection per session, CLI and worker"
status: ready
plan_file: "docs/plans/active/unified-provider-auth.md"
plan_feature: "PF-84"
execution_order: 25
owner: "Codex PF-84 lane owner"
parallel_lane: "account-selection"
write_scope: "codex-rs/core/, codex-rs/exec/, codex-rs/tui/, codex-rs/cli/, codex-rs/app-server/, codex-rs/app-server-protocol/, codex-rs/app-server-client/, codex-rs/app-server-test-client/, codex-rs/protocol/, codex-rs/model-provider-info/, codex-rs/model-provider/, codex-rs/login/, codex-rs/vault/, codex-rs/config/, codex-rs/features/, codex-rs/rollout/, codex-rs/state/, codex-rs/thread-store/, codex-rs/telegram/, codex-rs/thread-manager-sample/, docs/provider-accounts.md, qa/provider-auth/pf-84/s03-gate.md, qa/demos/specs/pf84-spawn-account.toml, qa/demos/specs/pf84-exec-account.toml, qa/demos/index/PF-84-S03.md, docs/sprints/current/unified-provider-auth/pf-84-s03-account-selection-and-propagation.md, docs/sprints/current/unified-provider-auth/index.md"
integration_gate: "Codex PF-84 lane owner merges to main after just test -p codex-core -p codex-exec -p codex-cli -p codex-app-server (with and without developer-accounting), Linux clippy -D warnings, the tmux/GLM run and one Opus 5.5 High review"
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf84-s03-select"
branch: "feat/pf-84-s03-account-selection"
base_commit: "0fa45b54f0ca65a6ce3485c27281624cb6f8b2cb"
depends_on: "PF-84-S02"
created: 2026-10-08
updated: 2026-10-10
merged_behind_flag: "named_accounts"
gate_evidence: "qa/provider-auth/pf-84/s03-gate.md"
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

- [x] Plan is active.
- [x] Dependencies are completed (PF-84-S02 merged behind `named_accounts`, `0fa45b54f0`).
- [x] Worktree, branch, and base commit are exact and match the plan.
- [x] If parallel, owner/lane/scopes are disjoint and the receiving integration gate is recorded.
- [x] Decision D3 (model-chosen spawn account and security levels) recorded in the plan.

## Done

- [x] Sprint record created and linked to one plan feature.
- [x] Thread config carries the account; spawned children (including role children and
  `thread/spawnAgent` workers) inherit it unless overridden.
- [x] Spawn `account` accepts configured names of the child's provider only; the spawn result
  shows `account`; D3 approval under Aggressive (level in force), refused with approvals off.
- [x] `--account` on exec, the TUI and resume; missing accounts are refused at thread start.
- [x] Resume uses the recorded account; a removed account blocks with recovery text.
- [x] TUI worker preflight checks the worker's own account (S02 known limit).
- [x] Gate passed; dispositions in the [gate evidence](../../../../qa/provider-auth/pf-84/s03-gate.md).

## Remaining

- [ ] Independent code-blind functional design and execution (acceptance step, not the implementer).

## Verification

- [x] Focused `just test` (core, exec, cli, app-server, model-provider, TUI filters); macOS clippy.
- [x] tmux + GLM 5.3 Flash: spawn on `fake` (401, coordinator still replies); launcher workers
  `--account main|fake|gone` and resume; Kimi second provider; TUI `--account`.
- [x] Videos ([index](../../../../qa/demos/index/PF-84-S03.md)); Opus 5.5 High review + one follow-up.
- [ ] Linux clippy on the RTX box (offline 2026-10-10; the PR's Ubuntu clippy job stands in).

## Security notes

- The model sees account names, never values. No env var carries a credential to children.
- Child isolation is proven with canaries: B's failure never retries on A.
- Disposable homes and `CORBANU_TEST_NO_NATIVE_KEYRING=1` for every candidate run.

## Exit evidence

- [x] Implementation commit and PR recorded (gate evidence).
- [x] Test output, tmux logs and video paths linked.
- [ ] Code-blind handoff checker passes, or limited-testing agreement recorded.
- [x] `Done` and `Remaining` ledgers reflect reality.
- [ ] Completed record moved to `docs/sprints/archive/unified-provider-auth/`.
