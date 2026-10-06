---
sprint_id: "PF-27-S05"
title: "Core model-client auth and vault labels through the broker"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-27"
execution_order: 46
owner: "broker lane"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-27-S02"
created: 2026-10-06
updated: 2026-10-06
---

# PF-27-S05 — Core model-client auth and vault labels through the broker

Created by the coordinator's 2026-10-06 decision (Travis may revisit): moving Core's own model-client auth into
the broker is its own sprint, after PF-27-S02 in the broker lane. It touches the model clients rather than the
proxy, and it needs PF-27-S02's containment before the broker is a real boundary.

## Execution mandate

- Deliver: Core's own model-provider requests are authorized by the isolated broker; Core holds opaque references,
  not raw provider keys, and vault-label credentials are resolved only inside the broker.
- Excludes: agent launch containment (PF-27-S02), output gates (PF-28), Windows (PF-27-S06), Permissive changes.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-27).
- Feature: `PF-27`.
- Product citation: **Required trust boundaries** — “Credentials are referenced by label and resolved only inside a trusted execution boundary.”
- Acceptance advanced: raw credentials exist only in the trusted broker, including the ones Core uses itself.

## Code boundaries

- Existing: `codex-rs/model-provider/src/auth.rs`; `codex-rs/login/` (API-key auth); `codex-rs/core/src/client.rs`;
  `codex-rs/network-proxy/src/credential_broker/isolated/`; `codex-rs/vault/src/`.
- Planned: a broker route for model-provider requests; a vault-label resolver inside the broker process.
- Tests: colocated `pf_27_s05` modules; synthetic keys and fake providers only.

## Preconditions

- [ ] PF-27-S02 completed and archived; plan active; exact worktree recorded in the plan.

## Done

- [x] Record created from the PF-27-S04 closure with scope and dependencies.

## Remaining

- [ ] Inventory every place Core reads provider keys (env keys, `auth.json` API key, provider profiles, ChatGPT tokens) and how each request is signed.
- [ ] Route model-provider requests that use API keys through the broker; Core keeps references; no raw-key fallback when the broker is down.
- [ ] Remove env-sourced provider keys from Core's process environment once the broker holds them, so unsandboxed same-user processes (MCP servers, hooks) cannot read them from Core's launch environment (the macOS limit PF-27-S02 records).
- [ ] Resolve vault-label credentials inside the broker only; the agent and Core see labels and dummies.
- [ ] Decide streaming/websocket and token-refresh handling for ChatGPT sign-in; unsupported routes stay explicit.
- [ ] Add `pf_27_s05` tests: broker-only key use, broker death fails closed, no raw key in Core memory dumps/logs.

## Verification

- [ ] `just fix -p <crate>` and `just fmt`; focused `just test -p codex-core pf_27_s05` and the affected crates.
- [ ] GLM 5.2 tmux run, one Opus 5.5 High review, SOP demo videos (decision 5).

## Exit evidence

- [ ] Outputs under `qa/security-levels/sprints/PF-27-S05/`; Done/Remaining reflect reality; record archived.
