---
sprint_id: "PF-84-S02"
title: "Named account registry, storage and credential resolution"
status: ready
plan_file: "docs/plans/active/unified-provider-auth.md"
plan_feature: "PF-84"
execution_order: 24
owner: "Codex PF-84 account-registry worker"
parallel_lane: "account-registry"
write_scope: "codex-rs/vault/, codex-rs/login/, codex-rs/cli/, codex-rs/network-proxy/, codex-rs/model-provider-info/, codex-rs/model-provider/, codex-rs/features/, codex-rs/config/, codex-rs/core/, codex-rs/arg0/, codex-rs/tui/, codex-rs/state/, codex-rs/telegram/, codex-rs/provider-auth/, codex-rs/memories/, codex-rs/app-server/, codex-rs/exec/, docs/provider-accounts.md, qa/demos/specs/pf84-account-isolation.toml, docs/sprints/current/unified-provider-auth/pf-84-s02-account-registry-and-resolution.md"
integration_gate: "Codex PF-84 lane owner merges to main after just test -p codex-vault -p codex-login -p codex-cli -p codex-network-proxy -p codex-model-provider-info -p codex-model-provider (with and without developer-accounting), RTX clippy and the tmux/GLM run"
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf84-s02-accounts"
branch: "feat/pf-84-s02-account-registry"
base_commit: "051f9747225776a5d85ad00c2e5d0a8f5f4036bf"
depends_on: "none"
created: 2026-10-08
updated: 2026-10-10
---

# PF-84-S02 — Named account registry, storage and credential resolution

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
- Claude Plan: `vault/src/claude_auth.rs:25, 32-42, 157-158, 495-537` (selection,
  sentinel, `provider/claude-code-oauth-token`) gets account-qualified forms.
- Token resolver: `cli/src/main.rs:2401-2406`, `cli/src/claude_oauth.rs:174-236`;
  env exception `178-183, 231-233` stays `default`-only.
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

## Remaining

- [ ] Freeze names (`[a-z0-9][a-z0-9-]{0,31}`, `default` reserved), labels `provider/<id>/accounts/<name>/<kind>`, vault registry.
- [ ] Resolvers take an account; `default` keeps today's labels. Named accounts never read provider env vars.
- [ ] Named ChatGPT logins live in the vault: no extra `Codex Auth` item, no extra `auth.json`.
- [ ] `auth.command` gets `CORBANU_PROVIDER_ACCOUNT=<name>`; AWS takes `aws.profile` per account.
- [ ] `internal-claude-oauth-token --account <name>`; unknown account fails closed.
- [ ] Default-off `named_accounts` flag; `[provider_accounts]` config selector; `corbanu account list|add|remove` (secrets from stdin only).
- [ ] Code-blind functional design frozen before test-result disclosure, or N/A reason recorded.

## Verification

- [ ] Focused: `just test -p codex-vault -p codex-login -p codex-cli -p codex-network-proxy`.
- [ ] Isolation canaries: a distinct fake value per account per kind. Resolving A
  never yields B's value. A failed B never falls back to A or to `default`.
- [ ] Migration: a fixture copy of a single-account home resolves identically
  before and after, and its vault bytes stay unchanged until the first named-account write.
- [ ] tmux + GLM run (`-m glm-5.3-flash -c model_provider="zai"`): default ZAI account
  replies; a named ZAI account holding a fake key gets a 401 (multiacct1 method).
- [ ] Video: `qa/demos/specs/pf84-account-isolation.toml` via `scripts/demo_video.py ... --sprint PF-84-S02 --publish`.
- [ ] One independent review (Opus 5.5 High).

## Security notes

- No OS keychain items are created, read or probed by debug candidates
  (`CORBANU_TEST_NO_NATIVE_KEYRING=1`, disposable homes). The product adds zero
  keychain items per account.
- Only names and 12-hex fingerprints appear in logs, errors and rollouts; `vault auth-helper` rules still apply.

## Exit evidence

- [ ] Implementation commit and PR recorded.
- [ ] Test output, canary scan, tmux log and video path linked.
- [ ] Code-blind handoff checker passes, or limited-testing agreement recorded.
- [ ] `Done` and `Remaining` ledgers reflect reality.
- [ ] Completed record moved to `docs/sprints/archive/unified-provider-auth/`.
