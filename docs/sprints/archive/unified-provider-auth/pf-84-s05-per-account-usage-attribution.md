---
sprint_id: "PF-84-S05"
title: "Per-account usage and rate-limit attribution"
status: completed
plan_file: "docs/plans/completed/main-2026-10-14-unified-provider-auth.md"
plan_feature: "PF-84"
execution_order: 27
owner: "Codex PF-84 lane owner"
parallel_lane: "account-usage"
write_scope: "codex-rs/state/, codex-rs/core/, codex-rs/tui/, codex-rs/model-provider/, codex-rs/model-provider-info/, codex-rs/vault/, codex-rs/login/, codex-rs/Cargo.lock, qa/provider-auth/pf-84/s05-gate.md, qa/provider-auth/pf-84/s05/, qa/demos/specs/pf84-usage-by-account.toml, qa/demos/specs/pf84-rate-limit-by-account.toml, qa/demos/index/PF-84-S05.md, docs/sprints/current/unified-provider-auth/pf-84-s05-per-account-usage-attribution.md, docs/sprints/current/unified-provider-auth/index.md"
integration_gate: "Codex PF-84 lane owner merges to main after just test -p codex-state -p codex-core -p codex-tui (plus touched crates) with and without developer-accounting, Linux clippy -D warnings on glitch, the tmux/GLM run and one Opus 5.5 High review"
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf84-s05-usage"
branch: "feat/pf-84-s05-usage-by-account"
base_commit: "c7e6e07cbc629b8fa366de30cfacefae23b06b69"
depends_on: "PF-84-S03, PF-60-S04"
created: 2026-10-08
updated: 2026-10-11
merged_behind_flag: "named_accounts"
gate_evidence: "qa/provider-auth/pf-84/s05-gate.md"
---

# PF-84-S05 — Per-account usage and rate-limit attribution

## Closure — 2026-10-11

Completed. Travis **accepted** PF-84-S05 on 2026-10-11 behind the default-off `named_accounts` flag
([comment on #483](https://github.com/CorbanuCore/CorbanuTerminal/pull/483)); the flag stays off. The independent run
was source-blind, not the isolated code-blind VM run; Travis accepted on that evidence, recorded here as the
limited-testing agreement. Archived by the PF-84-S06 worker.

Evidence:

- [Gate](../../../../qa/provider-auth/pf-84/s05-gate.md) and PR #469 (implementation).
- [S05 independent acceptance](../../../../qa/provider-auth/pf-84/independent-acceptance-s05-20261011/README.md)
  (PR #483, [review](../../../../qa/provider-auth/pf-84/independent-acceptance-s05-20261011/REVIEW.md)): throttle and
  `exec` rows partial; issues #474, #475, #476, #480, #481, #482, all fixed in #485, #486, #489 and #490 (not re-run by
  an independent tester).
- Follow-up #528 (Travis, option b): release `/status` shows token totals per account
  ([gate](../../../../qa/provider-auth/pf-84/s05-status-tokens-gate.md)); merged after the acceptance, not independently
  re-tested.

Known limits (tracked, not blockers):

- A named account's throttle row is keyed by name, so a replaced key keeps its cooldown. Travis decided on
  2026-10-11 to key it by key fingerprint; a separate worker owns that change.
- Only a turn's own model requests check and record the throttle (compaction, the turn-completion check and memory
  extraction bypass it); Z.AI's transport retries a 429 five times before the cooldown starts.
- Usage pages per account (`/cost`) exist only in developer-accounting builds; rate-limit snapshots per account need
  named ChatGPT sign-ins (PF-84-S06 slice B). The acceptance's 2 s fingerprint timeout and Claude panes rows were not
  verifiable.

## Execution mandate

- Deliver: every accounted request, rate-limit snapshot and provider-request lease
  records its provider account. `/status` shows the current account. Usage views
  group by (provider, account) across the coordinator and its workers in one home.
- Excludes: pricing changes; cross-home aggregation (separate homes stay separate);
  changes to the PF-60 accounting contract beyond adding the account field.

## Plan linkage

- Plan: [Unified provider onboarding and management](../../../plans/completed/main-2026-10-14-unified-provider-auth.md)
- Feature: `PF-84`
- Acceptance advanced: "Per-account usage".

## Code boundaries

Paths are under `codex-rs/`, at `63ea3d0cbd`.

- Accounting records: `core/src/accounting.rs:48-68, 225-269, 346-463` (keyed by `provider_id` today);
  schema in `state/accounting_migrations/` (`state/src/migrations.rs:7-10`).
- Rate-limit leases: `state/src/runtime/provider_requests.rs:8-12, 92, 127`. These
  are already keyed by `key_fingerprint`; make that the account fingerprint for
  every auth kind, including subscription logins.
- Display: `tui/src/chatwidget/tokens.rs:1085` (provider display name) and the `/status` card.
- Tests: accounting unit tests, a migration test, a status snapshot.

## Preconditions

- [x] Plan is active.
- [x] Dependencies are completed (PF-60-S04 archived 2026-10-10; PF-84-S03 archived; S04 `ready` behind the flag).
- [x] Worktree, branch, and base commit are exact and match the plan.
- [x] Serial in the PF-84 lane; the integration gate is recorded above.

## Done

- [x] Sprint record created and linked to one plan feature.
- [x] Accounted attempts record `account: {name, fingerprint}` (12 hex, salted per home)
  through ledger format 4 (`0004_account_dimension.sql`). Default-account attempts omit it,
  keep their bytes and read as `default`, and never upgrade the ledger.
- [x] Throttle state (429 cooldown, request lease) is per (provider, account); account B's 429
  never blocks A.
- [x] `/status` names the account (S04); `/cost` groups rows, pages and request links by
  provider, account and model, in this conversation (coordinator and spawned agents) and in
  other conversations (`corbanu exec` workers) of the same home, reconciling exactly.
- [x] Merged behind `named_accounts` with gate evidence: [gate](../../../../qa/provider-auth/pf-84/s05-gate.md).
- [x] Follow-up (Travis, 2026-10-11, option b): release `/status` shows token totals per account,
  no cost records ([#528](https://github.com/CorbanuCore/CorbanuTerminal/pull/528), [gate](../../../../qa/provider-auth/pf-84/s05-status-tokens-gate.md)).

## Remaining

- [x] Independent acceptance: a source-blind run (PR #483) instead of the isolated code-blind run; Travis accepted it 2026-10-11 as limited testing.
- [x] Known limit recorded (Closure): the throttle row is keyed by name until the fingerprint-keying follow-up.

## Verification

- [x] Focused: `just test` on codex-state, codex-core and codex-tui (accounting, migration,
  throttle, `/cost`), with and without developer-accounting; Linux clippy `-D warnings` on glitch.
- [x] tmux SOP: coordinator on `default`, spawned agents on `work` and `fake`, `corbanu exec`
  workers and Kimi; `/cost` shows one row per account with counts equal to provider-reported usage.
- [x] GLM run (`glm-5.3-flash`, `zai`): `work` (real key under a second name) accrues tokens;
  `fake` records one failed request with no tokens and no spend.
- [x] Video: `pf84-usage-by-account.toml` ([mp4](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s05-pf84-usage-by-account-a1921d736bfa-2026-10-11.mp4)).
- [x] One independent review (Opus 5.5 High): APPROVE, non-blocking items handled (gate).
- [x] Independent acceptance run linked (PR #483); Travis signed off 2026-10-11.

## Security notes

- Accounting stores names and fingerprints only, salted per home so they cannot be correlated across machines.
- No credential value reaches the state DB, logs or exports (canary scan).

## Exit evidence

- [x] Implementation commit and PR recorded (branch `feat/pf-84-s05-usage-by-account`, PR [#469](https://github.com/CorbanuCore/CorbanuTerminal/pull/469)).
- [x] Test output, tmux log and video path linked ([gate](../../../../qa/provider-auth/pf-84/s05-gate.md)).
- [x] Limited-testing agreement recorded (Closure, 2026-10-11).
- [x] `Done` and `Remaining` ledgers reflect reality.
- [x] Completed record moved to `docs/sprints/archive/unified-provider-auth/`.
