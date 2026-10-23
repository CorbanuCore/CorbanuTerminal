---
sprint_id: "PF-84-S06"
title: "Named AWS profile and ChatGPT/OpenAI sign-in accounts"
status: completed
plan_file: "docs/plans/completed/main-2026-10-14-unified-provider-auth.md"
plan_feature: "PF-84"
execution_order: 28
owner: "Codex PF-84-S06 worker"
parallel_lane: "account-kinds"
write_scope: "codex-rs/core/, codex-rs/vault/, codex-rs/login/, codex-rs/model-provider/, codex-rs/model-provider-info/, codex-rs/aws-auth/, codex-rs/cli/, codex-rs/tui/, codex-rs/exec/, codex-rs/app-server/, codex-rs/app-server-protocol/, codex-rs/Cargo.lock, MODULE.bazel.lock, docs/provider-accounts.md, qa/provider-auth/pf-84/s06a-gate.md, qa/provider-auth/pf-84/s06b-gate.md, qa/provider-auth/pf-84/s06/, qa/demos/specs/pf84-aws-profile-account.toml, qa/demos/specs/pf84-chatgpt-account.toml, qa/demos/index/PF-84-S06.md, docs/sprints/current/unified-provider-auth/pf-84-s06-aws-and-chatgpt-accounts.md, docs/sprints/current/unified-provider-auth/index.md"
integration_gate: "Codex PF-84-S06 worker merges each slice (A, then B) to main after just test on the touched crates with and without developer-accounting, Linux clippy -D warnings on glitch, macOS clippy, the tmux run in a disposable home and one Opus 5.5 High review per slice"
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf84-s06b-chatgpt"
branch: "feat/pf-84-s06b-chatgpt-accounts"
base_commit: "511ec41249ba7897d1474915a17333360e070edd"
depends_on: "PF-84-S05"
created: 2026-10-11
updated: 2026-10-13
merged_behind_flag: "named_accounts"
gate_evidence: "qa/provider-auth/pf-84/s06b-gate.md"
---

# PF-84-S06 — Named AWS profile and ChatGPT/OpenAI sign-in accounts

## Closure — 2026-10-13

Completed. Travis **accepted** both slices behind the default-off `named_accounts` flag as **limited testing**; the
flag stays off.

- **Slice A** (AWS profiles), 2026-10-12 ([comment on #574](https://github.com/CorbanuCore/CorbanuTerminal/pull/574)).
  Mock-only verification accepted 2026-10-13; #572 fixed by #610 and re-verified (A10).
- **Slice B** (ChatGPT/OpenAI sign-ins), 2026-10-13 ([comment on #574](https://github.com/CorbanuCore/CorbanuTerminal/pull/574)),
  on the code-blind re-acceptance
  ([README](../../../../qa/provider-auth/pf-84/independent-acceptance-s06b-20261013/README.md), PR #663): slice B and
  A10 PASS; #555 19 of 22 blockers met. Defects it found are fixed: #654 (#669), #656 (#668), #658 (#676).

Limited-testing agreement (what was not verified):

- Mock issuer and fake tokens only: no live ChatGPT sign-in (no disposable account) and no live Bedrock (no test AWS
  account); B1's keychain, B3's browser flow and B7's spawned-agent, `fork` and daemon paths not exercised.
- Isolation by instruction (not enforced), debug builds (not the packaged candidate); slice B criteria derived from
  the gate, not a frozen design. The "code-blind functional design frozen" item stays open.
- Partial #555 legs: ESC-03's draft; REG-01's flag-off Esc-in-picker leg; GH-02 not naming `ghost`; the z4 PR-01
  re-runs without `/status`; ESC-04 without a before-build comparison; ESC-01's terminal restore; D3's "refused with
  approvals off" leg. REG-02 not met (inherited: `login status` shows a masked key fragment). REG-06 ran a Linux subset.

Decisions recorded with the acceptance (Travis, 2026-10-13):

- **PR-02:** a missing `--account` (of any provider) keeps refusing the whole launch.
- **D3:** the person's own *Use for this session* counts as the approval; only a model-chosen (spawn) account prompts
  under Aggressive.
- **#595 / #676:** a named-account session starts its own app server instead of reusing a running daemon; the
  remote-control handshake and managed-settings download stay on the default login (documented exceptions).
- **#588:** a fork of an earlier turn uses the source conversation's current account, as resume does.

Flag-removal prerequisites: see the plan's PF-84 section (live ChatGPT and live Bedrock checks with test accounts,
the S03 limits #425/#426/#428 re-verified, the partial #555 legs, #588/#609/#689, an isolated code-blind run).

## Execution mandate

- Deliver, behind `named_accounts`, in two slices merged in order (Travis, 2026-10-11, option (a)):
  - **A — AWS profile accounts.** An account stores only an AWS profile name (not secret) and Bedrock uses it as `aws.profile`.
  - **B — ChatGPT/OpenAI sign-in accounts.** Each account has its own sign-in store in the vault. Refreshed tokens are saved back to that account, and login can target a named account.
- Excludes: static AWS keys in the vault, SSO set-up UI, cross-home import (D2), and pricing/usage changes (S05).

## Plan linkage

- Plan: [Unified provider onboarding and management](../../../plans/completed/main-2026-10-14-unified-provider-auth.md)
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

- [x] Plan is active.
- [x] Dependencies are completed (PF-84-S05 merged behind `named_accounts` and accepted by Travis 2026-10-11).
- [x] Worktree, branch, and base commit are exact and match the plan.
- [x] Serial in the PF-84 lane; the integration gate is recorded above.

## Done

- [x] Sprint record created as a draft from Travis's 2026-10-11 decision (option (a)).
- [x] Slice A (A1–A3), merged behind `named_accounts`: AWS profile accounts for Amazon Bedrock, profile-only
  signing, refusal before any request; CLI add/list/rename/remove and the `/providers` row. Gate:
  [s06a-gate.md](../../../../qa/provider-auth/pf-84/s06a-gate.md) (live Bedrock pending: no test profile). PR #543.
- [x] Slice B (B1–B5), merged behind `named_accounts`: named ChatGPT/OpenAI sign-in accounts in the vault, refresh
  written back per account, `corbanu login|logout --account`, `/providers` device-code and API-key adds, the default
  login untouched, rows with email hint/plan/fingerprint and the same-user warning. Gate:
  [s06b-gate.md](../../../../qa/provider-auth/pf-84/s06b-gate.md) (live ChatGPT pending: no disposable account).
- [x] Mid-session switch: connectors, plugin installs, accounting and the model list follow the active account ([gate](../../../../qa/provider-auth/pf-84/s06b-gate.md#follow-up-mid-session-account-switch-fixpf84-switch-follow)).
- [x] Early `/status` rate limits, the `/apps` directory, plugin listings, ChatGPT settings and analytics use the session's account, never the default login (#658).

## Remaining

- [x] Independent acceptance: code-blind re-acceptance by a separate executor (PR #663), accepted 2026-10-13 as
  limited testing (Closure). A frozen code-blind design and live checks move to the flag-removal prerequisites.

## Verification

- [x] Focused: `just test -p codex-core -p codex-login -p codex-model-provider -p codex-aws-auth -p codex-vault` (plus touched crates); Linux clippy `-D warnings` on glitch; macOS clippy. Both slices ([A](../../../../qa/provider-auth/pf-84/s06a-gate.md), [B](../../../../qa/provider-auth/pf-84/s06b-gate.md)).
- [x] tmux SOP (disposable homes, `CORBANU_TEST_NO_NATIVE_KEYRING=1`): A, with a fake profile and a missing profile, refused while env AWS keys are set. B, with an expired fake `work` store, shows "sign in again" while `default` stays intact. Restart is durable.
- [x] GLM run: coordinator `-m glm-5.3-flash -c model_provider="zai"`; workers `--account amazon-bedrock:fake` and `--account openai:work` (fake) get visible refusals, with no fallback, and the coordinator still replies.
- [x] A live (limited testing, accepted 2026-10-13 without it; carried to the flag-removal prerequisites): **NOT VERIFIABLE** without a test AWS profile. Travis would need to provide an AWS account with Bedrock model access in one region and a named model. He would also need a least-privilege IAM user or role (`bedrock:InvokeModel*`), its profile in a disposable `AWS_CONFIG_FILE`, with keys in the vault fetched only via `auth-helper`, and a spend cap.
- [x] B live (limited testing, accepted 2026-10-13 without it; carried to the flag-removal prerequisites; no disposable ChatGPT account, Travis's login not used): a second real ChatGPT login signed in by Travis via device code in a disposable home; otherwise only the mock-server and fake-token tests count.
- [x] Videos: `pf84-aws-profile-account.toml`, `pf84-chatgpt-account.toml` ([index](../../../../qa/demos/index/PF-84-S06.md)). One Opus 5.5 High review per slice (A and B: APPROVE WITH NITS, round 3).

## Security notes

- Tokens live only in the encrypted vault. Named accounts add no keychain item, and no token reaches logs, rollouts, the state DB, snapshots or videos (canary scan).
- The AWS profile name is metadata. The sprint never reads or stores AWS secrets and never sets env AWS keys for children.
- A refresh or logout is scoped to one account. Under Aggressive, switching account still needs approval (D3).

## Decisions (Travis, 2026-10-11)

1. The default ChatGPT login stays where it is (no move into the vault).
2. The same ChatGPT user signed in under two accounts gets a warning, not a refusal.
3. No test AWS profile: slice A is tested with a fake AWS config in a disposable home and a mock Bedrock endpoint;
   the live Bedrock check stays pending (limited testing).
4. AWS SSO profiles are allowed as profile names; `aws sso login` stays outside Corbanu (no SSO set-up UI).

## Exit evidence

- [x] Implementation commits and PRs (A #543, then B) recorded.
- [x] Test output, tmux logs, canary scan and video paths linked (gates).
- [x] Code-blind handoff checker passes, or limited-testing agreement recorded (Closure).
- [x] `Done` and `Remaining` ledgers reflect reality.
- [x] Completed record moved to `docs/sprints/archive/unified-provider-auth/`.
