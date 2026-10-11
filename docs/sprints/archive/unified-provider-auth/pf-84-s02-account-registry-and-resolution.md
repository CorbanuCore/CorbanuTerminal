---
sprint_id: "PF-84-S02"
title: "Named account registry, storage and credential resolution"
status: completed
plan_file: "docs/plans/active/unified-provider-auth.md"
plan_feature: "PF-84"
execution_order: 24
owner: "Codex PF-84 account-registry worker"
parallel_lane: "account-registry"
write_scope: "codex-rs/vault/, codex-rs/login/, codex-rs/cli/, codex-rs/network-proxy/, codex-rs/model-provider-info/, codex-rs/model-provider/, codex-rs/features/, codex-rs/config/, codex-rs/core/, codex-rs/arg0/, codex-rs/tui/, codex-rs/state/, codex-rs/telegram/, codex-rs/provider-auth/, codex-rs/memories/, codex-rs/app-server/, codex-rs/exec/, docs/provider-accounts.md, mkdocs.yml, qa/demos/specs/pf84-account-isolation.toml, qa/demos/specs/pf84-account-isolation-kimi.toml, qa/demos/index/PF-84-S02.md, qa/provider-auth/pf-84/s02-gate.md, docs/plans/active/unified-provider-auth.md, docs/sprints/current/unified-provider-auth/pf-84-s04-provider-account-management-ui.md, docs/sprints/current/unified-provider-auth/pf-84-s02-account-registry-and-resolution.md"
integration_gate: "Codex PF-84 lane owner merges to main after just test -p codex-vault -p codex-login -p codex-cli -p codex-network-proxy -p codex-model-provider-info -p codex-model-provider (with and without developer-accounting), RTX clippy and the tmux/GLM run"
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf84-s02-accounts"
branch: "feat/pf-84-s02-account-registry"
base_commit: "051f9747225776a5d85ad00c2e5d0a8f5f4036bf"
depends_on: "none"
created: 2026-10-08
updated: 2026-10-11
merged_behind_flag: "named_accounts"
gate_evidence: "qa/provider-auth/pf-84/s02-gate.md"
---

# PF-84-S02 — Named account registry, storage and credential resolution

## Closure — 2026-10-11

Completed. Travis **accepted** PF-84-S02 on 2026-10-11 (in chat with the coordinator) behind the default-off `named_accounts` flag; the flag stays off. Both acceptance runs were source-blind, not the full isolated code-blind run; Travis accepted on that evidence, which is recorded here as the limited-testing agreement. Archived by the PF-84 integration worker.

Evidence:

- [Gate evidence](../../../../qa/provider-auth/pf-84/s02-gate.md) (tests, tmux/GLM 5.3 Flash runs, review dispositions, videos).
- [S01/S02 independent acceptance](../../../../qa/provider-auth/pf-84/independent-acceptance-20261010/README.md) (PR #420, [review](../../../../qa/provider-auth/pf-84/independent-acceptance-20261010/REVIEW.md)): 12 pass, S02-3b (#414), S02-9b (#415) and S02-14 (#416) failed, S02-15 not verifiable.
- [acceptance fixes #414–#419](../../../../qa/provider-auth/pf-84/acceptance-fixes-20261010/README.md) (PR #422); the [S03 independent acceptance and S02 re-check](../../../../qa/provider-auth/pf-84/independent-acceptance-s03-20261010/README.md) (PR #429, [review](../../../../qa/provider-auth/pf-84/independent-acceptance-s03-20261010/REVIEW.md)) re-ran S02-3b, S02-9b and S02-14: all pass.
- PRs: #407, #409 (implementation), #420 (acceptance), #422 (#414–#419 fixes), #429 (re-check).

Follow-ups (tracked, not blockers):

- S02-15 stays open: which model routes count as "siblings" for model correction was not verifiable from
  outside; it needs someone to name a sibling route pair.
- Named ChatGPT/OpenAI sign-ins and AWS profiles still fail closed; they moved to PF-84-S06 (decision D4).

## Execution mandate

- Deliver: a typed `(provider id, account name)` model, stored in the existing
  per-home vault, so that every credential resolver takes an account and reads
  only that account's material. Covers API keys, Claude Plan tokens, Claude Code
  login, ChatGPT login, `auth.command` and AWS profiles.
- Existing credentials become the implicit `default` account with no rewrite
  and no reauthentication.
- Excludes: choosing an account (S03), UI (S04), accounting (S05).

## Plan linkage

- Plan: [Unified provider onboarding and management](../../../plans/active/unified-provider-auth.md)
- Feature: `PF-84`
- Acceptance advanced: "Existing single-account migration" and "Account isolation".

## Code boundaries

Refs are at `origin/main` `63ea3d0cbd`, from multiacct1; paths are under `codex-rs/`.

- Vault key (unchanged, no new item): `secrets/src/lib.rs:23, 213-225`; `secrets/src/local.rs:380-404, 549-600`.
- Claude Plan: `vault/src/claude_auth.rs:25, 32-42, 157-158, 495-537`; resolver `cli/src/main.rs:2401-2406`,
  `cli/src/claude_oauth.rs:174-236` (env exception `178-183, 231-233` stays `default`-only).
- Claude Code login: `cli/src/claude_oauth.rs:662-676, 751-761, 873-931, 955-963`;
  `vault/src/claude_auth.rs:164-245`. Each account stores its own `CLAUDE_CONFIG_DIR`.
- API keys: `login/src/auth/provider_key_vault.rs:39, 119-231`; `login/src/auth/manager.rs:1278-1291`;
  `network-proxy/src/credential_broker/isolated/server.rs:653-690`. ChatGPT: `login/src/auth/storage.rs:231-245`.
- Command/AWS: `login/src/auth/external_bearer.rs:173-192, 247-258`;
  `model-provider-info/src/lib.rs:947, 1281-1298`.

## Preconditions

- [x] Plan is active.
- [x] Dependencies are completed.
- [x] Worktree, branch, and base commit are exact and match the plan.
- [x] If parallel, owner/lane/scopes are disjoint and the receiving integration gate is recorded.
- [x] Reconcile with P0 PF-76-S01: reconciled in the plan 2026-10-10 (profiles select named accounts).

## Done

- [x] Sprint record created and linked to one plan feature.
- [x] Names, labels `provider/<id>/accounts/<name>/<kind>` and the vault registry frozen; `default` keeps today's labels.
- [x] Resolvers take an account; named accounts never read provider env vars or fall back.
- [x] `auth.command` gets `CORBANU_PROVIDER_ACCOUNT`; unenrolled accounts never run the command.
- [x] `internal-claude-oauth-token --account` (token or Claude Code dir); unknown fails closed.
- [x] Default-off `named_accounts` flag; `[provider_accounts]`; `corbanu account list|add|remove` (stdin only).
- [x] Descoped 2026-10-10 to PF-84-S04: named ChatGPT/OpenAI sign-ins and AWS profiles (they fail closed now).
- [x] Focused tests (with/without developer-accounting), isolation canaries, migration bytes check.
- [x] tmux + GLM 5.3 Flash run (Z.AI `main` pong, `fake` 401); Kimi second provider; videos published.
- [x] Independent Opus 5.5 High review (three passes); dispositions in the [gate evidence](../../../../qa/provider-auth/pf-84/s02-gate.md).

## Remaining

- [x] Independent acceptance: a source-blind run instead of the isolated code-blind run (PRs #420 and #429); Travis accepted it 2026-10-11 as limited testing.

## Verification

- [x] Focused: `just test -p codex-vault -p codex-login -p codex-cli -p codex-network-proxy` (plus model-provider, model-provider-info, config, features, core filters).
- [x] tmux + GLM run (`-m glm-5.3-flash -c model_provider="zai"`) with the 401 proof.
- [x] Videos: `pf84-account-isolation`, `pf84-account-isolation-kimi` (`qa/demos/index/PF-84-S02.md`).
- [x] One independent review (Opus 5.5 High).
- [x] Independent acceptance run linked (PRs #420 and #429); Travis signed off 2026-10-11.

## Security notes

- No OS keychain items are created, read or probed by debug candidates
  (`CORBANU_TEST_NO_NATIVE_KEYRING=1`, disposable homes). The product adds zero
  keychain items per account.
- Only names and 12-hex fingerprints appear in logs, errors and rollouts; `vault auth-helper` rules still apply.

## Exit evidence

- [x] Implementation commit and PR recorded (gate evidence).
- [x] Test output, canary scan, tmux log and video paths linked.
- [x] Limited-testing agreement recorded (Closure, 2026-10-11).
- [x] `Done` and `Remaining` ledgers reflect reality.
- [x] Completed record moved to `docs/sprints/archive/unified-provider-auth/`.
