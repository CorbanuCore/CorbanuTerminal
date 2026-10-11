---
sprint_id: "PF-84-S06"
title: "Named AWS profile and ChatGPT/OpenAI sign-in accounts"
status: draft
plan_file: "docs/plans/active/unified-provider-auth.md"
plan_feature: "PF-84"
execution_order: 28
owner: "UNALLOCATED"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-84-S05"
created: 2026-10-11
updated: 2026-10-11
---

# PF-84-S06 — Named AWS profile and ChatGPT/OpenAI sign-in accounts

## Execution mandate

- Deliver, behind `named_accounts`, in two slices merged in order (Travis, 2026-10-11, option (a)):
  - **A — AWS profile accounts.** An account stores only an AWS profile name (not secret) and Bedrock uses it as `aws.profile`.
  - **B — ChatGPT/OpenAI sign-in accounts.** Each account has its own sign-in store in the vault. Refreshed tokens are saved back to that account, and login can target a named account.
- Excludes: static AWS keys in the vault, SSO set-up UI, cross-home import (D2), and pricing/usage changes (S05).

## Plan linkage

- Plan: [Unified provider onboarding and management](../../../plans/active/unified-provider-auth.md)
- Feature: `PF-84`
- Acceptance advanced: "Named AWS and ChatGPT accounts" (decision D4).

## Code boundaries

Paths are under `codex-rs/`, at `ee530a0e26`.

- Fail-closed gate to lift per slice: `core/src/config/provider_accounts.rs::supports_named_accounts`.
- Kinds already reserved: `vault/src/provider_accounts.rs` `AwsProfile`, `ChatgptAuth`.
- A: `model-provider/src/amazon_bedrock/{mod,auth,mantle}.rs` (managed key, then the `AWS_BEARER_TOKEN_BEDROCK` env var, then the SDK chain); `aws-auth/src/config.rs`.
- B: `login/src/auth/storage.rs` (`AuthStorageBackend`, the `Codex Auth` keyring item, `auth.json`); `login/src/auth/manager.rs` (`persist_tokens`, `refresh_and_persist_chatgpt_token`, `save_auth`); `login/src/server.rs`, `device_code_auth.rs`; `cli/src/account_cmd.rs`; the `/providers` and `corbanu login` hosts.
- Tests: unit tests per resolver, a mock refresh server, tmux scenarios.

## Preconditions

- [ ] Plan is active.
- [ ] Dependencies are completed (PF-84-S05 merged behind `named_accounts`).
- [ ] Worktree, branch, and base commit are exact and match the plan.
- [ ] If parallel, owner/lane/scopes are disjoint and the receiving integration gate is recorded.

## Done

- [x] Sprint record created as a draft from Travis's 2026-10-11 decision (option (a)).

## Remaining

- [ ] A1: `corbanu account add amazon-bedrock <name> --kind aws-profile --value <p>` (and the S04 row action) stores the profile name as metadata. Values are validated, never secret, and shown in clear.
- [ ] A2: a named account sets `aws.profile` and uses a profile-only provider. It ignores the managed Bedrock key, `AWS_BEARER_TOKEN_BEDROCK`, `AWS_PROFILE` and `AWS_ACCESS_KEY_ID`/`AWS_SECRET_ACCESS_KEY`.
- [ ] A3: refuse with recovery text, before any request, when a profile is missing from the AWS config/credentials files or its credentials fail. Never fall back to `default` or to the SDK default chain. Unit tests cover each env var present.
- [ ] B1: `AuthStorageBackend` per account over vault label `provider/openai/accounts/<name>/chatgpt_auth` (whole `AuthDotJson`: ChatGPT tokens or an OpenAI API-key login). It creates no `Codex Auth` keychain item.
- [ ] B2: refresh writes back through that account's storage only, under a per-account refresh lock. A mock-server test proves that refreshing `work` never touches `default` and that a rotated refresh token lands in `work`.
- [ ] B3: `corbanu login --account <name>` (browser and device code) and the S04 "add account" row sign into the named account. Logout or remove clears only that account and revokes only its tokens.
- [ ] B4: migration. `default` keeps reading today's `Codex Auth` item or `auth.json` in place, byte for byte, with no copy or delete (D1). A test proves an upgraded home still resolves the same login.
- [ ] B5: the row shows only the email hint, plan type and the 12-hex fingerprint. A second sign-in to the same ChatGPT user ID gets a warning (open point 2).
- [ ] Code-blind functional design frozen before test-result disclosure.

## Verification

- [ ] Focused: `just test -p codex-core -p codex-login -p codex-model-provider -p codex-aws-auth -p codex-vault` (plus touched crates); Linux clippy `-D warnings` on glitch.
- [ ] tmux SOP (disposable homes, `CORBANU_TEST_NO_NATIVE_KEYRING=1`): A, with a fake profile and a missing profile, refused while env AWS keys are set. B, with an expired fake `work` store, shows "sign in again" while `default` stays intact. Restart is durable.
- [ ] GLM run: coordinator `-m glm-5.3-flash -c model_provider="zai"`; workers `--account amazon-bedrock:fake` and `--account openai:work` (fake) get visible refusals, with no fallback, and the coordinator still replies.
- [ ] A live: **NOT VERIFIABLE** without a test AWS profile. Travis would need to provide an AWS account with Bedrock model access in one region and a named model. He would also need a least-privilege IAM user or role (`bedrock:InvokeModel*`), its profile in a disposable `AWS_CONFIG_FILE`, with keys in the vault fetched only via `auth-helper`, and a spend cap.
- [ ] B live: a second real ChatGPT login signed in by Travis via device code in a disposable home; otherwise only the mock-server and fake-token tests count.
- [ ] Videos: `pf84-aws-profile-account.toml`, `pf84-chatgpt-account.toml` (`--sprint PF-84-S06 --publish`). One Opus 5.5 High review per slice.

## Security notes

- Tokens live only in the encrypted vault. Named accounts add no keychain item, and no token reaches logs, rollouts, the state DB, snapshots or videos (canary scan).
- The AWS profile name is metadata. The sprint never reads or stores AWS secrets and never sets env AWS keys for children.
- A refresh or logout is scoped to one account. Under Aggressive, switching account still needs approval (D3).

## Open points for Travis

1. Default ChatGPT login: keep it in place (recommended) or move it into the vault.
2. Same ChatGPT user in two accounts: warn (recommended) or refuse.
3. AWS: provide a test profile, or accept limited testing (A merges unverified live).
4. Allow AWS SSO profiles (`aws sso login` stays outside Corbanu)? Recommended: yes, as profile names only.

## Exit evidence

- [ ] Implementation commits and PRs (A, then B) recorded.
- [ ] Test output, tmux logs, canary scan and video paths linked.
- [ ] Code-blind handoff checker passes, or limited-testing agreement recorded (A live).
- [ ] `Done` and `Remaining` ledgers reflect reality.
- [ ] Completed record moved to `docs/sprints/archive/unified-provider-auth/`.
