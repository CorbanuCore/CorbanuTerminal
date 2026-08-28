---
sprint_id: "PF-13-S04"
title: "Credential authority lifecycle and raw-secret bypass closure"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-13"
execution_order: 24
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-13-S03"
created: 2026-08-24
updated: 2026-08-28
---

# PF-13-S04 — Credential authority lifecycle and raw-secret bypass closure

## Execution mandate

- Deliver: enforce revocation/replay and prevent Moderate/Aggressive agents from using raw-secret escape paths.
- Excludes: changing Permissive behavior, new providers, general protected-data policy, and TUI.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-13`
- Reconciliation: [source decisions and archive mapping](../../../plans/security-source-reconciliation.md).
- Product citation: **P0 `/security` levels** — “Existing approval, sandbox, vault, wallet, tool, network, and agent policies are unchanged.”
- Acceptance advanced: credential use produces a secret-free decision/receipt and revoked authority cannot be replayed.

## Code boundaries

- OpenClaw adoption reference: [OC-2](../../../plans/openclaw-source-review-2026-08-28.md#oc-2) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing: `cli/src/main.rs::run_vault_auth_helper`; `vault/src/lib.rs::reveal_for_programmatic_use`
- Existing: `security-policy/src/{revocation,mandate}.rs`; `network-proxy/src/credential_broker.rs`
- Tests: CLI vault tests, vault capability tests, proxy broker tests, Core security integration tests

## Preconditions

- [ ] PF-13-S03 is completed and archived.
- [ ] Read root, `codex-rs/AGENTS.md`, and `codex-rs/core/AGENTS.md`.
- [ ] Exact worktree coordinates match the active plan.

## Done

- [x] Sprint record identifies the existing raw `auth-helper` and broker storage bypasses.

## Remaining

- [ ] Add the stale TLS-handler fixture: revoke then re-register the same run ID/instance with a new generation and open a fresh connection to a still-admitted host. Neither cached server closures nor pooled channels may reuse the old credential; assert unrelated runs continue.

- [ ] Test authority closure on an already-established CONNECT/keep-alive channel, during body upload, and after same-ID run re-registration. New-connection 407/refusal tests alone do not prove revocation; unrelated active runs must continue safely.

- [ ] Consume or reject the capability atomically so replay and revocation races fail before another resolution.
- [ ] Emit `ActionReceipt` metadata for capability id, policy reason, operation, destination, and outcome without label value or secret.
- [ ] Preserve `vault auth-helper` in Permissive; make Moderate/Aggressive agent execution refuse it and direct credential use through the broker.
- [ ] Block equivalent raw-secret returns through child environment, tool output, logs, audit events, errors, serialization, and command substitution.
- [ ] Add revocation-before-resolve, revoke-during-use, duplicate, replay, helper, environment, cancellation, and restart regressions.
- [ ] Keep future finished-doc edits out of this sprint; PF-26-S03 updates user guidance after acceptance.

## Verification

- [ ] Fix: `cd codex-rs && just fix -p codex-cli && just fix -p codex-vault && just fix -p codex-network-proxy`.
- [ ] Format: `cd codex-rs && just fmt`; then inspect the final diff.
- [ ] Focused tests: `cd codex-rs && just test -p codex-cli vault && just test -p codex-vault capability && just test -p codex-network-proxy credential_broker`.
- [ ] Security integration: `cd codex-rs && just test -p codex-core credential_authority`.
- [ ] TUI applicability: none; PF-26-S02 owns the user-visible proof.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Denial/receipt output linked under `qa/security-levels/sprints/PF-13-S04/`.
- [ ] Review proves Moderate/Aggressive have no supported raw-secret path.
- [ ] Ledgers reflect reality and the completed record is archived.
