---
sprint_id: "PF-13-S03"
title: "OpenAI exact-host proxy substitution"
status: completed
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-13"
execution_order: 11
owner: "Jim Ricketts"
worktree: "/home/pfrpc/repos/CorbanuTerminal-pf13-s02"
branch: "feat/pf-13-s02-scoped-vault-resolver"
base_commit: "1bdc515bff48a4d9048dae7d06c6214e884265bc"
depends_on: "PF-13-S02"
created: 2026-08-24
updated: 2026-08-25
---

# PF-13-S03 — OpenAI exact-host proxy substitution

## Execution mandate

- Deliver: substitute one vault-backed bearer credential only for approved OpenAI HTTPS requests.
- Excludes: other providers, redirects, arbitrary headers, legacy CLI gating, receipts, and TUI.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-13`
- Acceptance advanced: approved `POST https://api.openai.com/v1/*` receives `Authorization: Bearer <secret>` only at transport.

## Code boundaries

- Proxy: `network-proxy/src/credential_broker.rs`; `credential_broker/resolver.rs`; `credential_broker/providers/openai.rs`
- Transport: `network-proxy/src/{mitm,http_proxy,runtime}.rs`
- Core: `core/src/config/network_proxy_credential.rs`; `core/src/config/network_proxy_spec.rs`
- Tests: `network-proxy/src/credential_broker_tests.rs`; `core/src/security/credential_capability_tests.rs`; affected Cargo/Bazel files

## Preconditions

- [x] PF-13-S02 is completed and archived.
- [x] Read root, `codex-rs/AGENTS.md`, and `codex-rs/core/AGENTS.md`.
- [x] Exact worktree coordinates match the active plan.

## Done

- [x] Sprint record names the provider, scheme, host, method, path, and header.
- [x] Replaced scoped OpenAI broker-owned raw state with a capability reference and secret-free resolver interface; legacy values now zeroize on drop.
- [x] Passed scheme, normalized host, port, method, path, actor/session/task, and capability id into validation before resolution.
- [x] Permitted only HTTPS, port 443, exact host `api.openai.com`, method `POST`, an exact authorized path under `/v1/`, and the OpenAI bearer header.
- [x] Resolved at header injection, released Vault's zeroizing source allocation immediately afterward, and refused redirect/retry reuse or conflicting authorization.
- [x] Denied lookalike/subdomain hosts, alternate ports/schemes/methods/paths, redirects, stale references, and explicit-header collisions.
- [x] Preserved existing Permissive broker behavior outside the opt-in scoped route.

## Remaining

- None.

## Verification

- [x] Fix: `cd codex-rs && just fix -p codex-network-proxy && just fix -p codex-core`.
- [x] Format: `cd codex-rs && just fmt`; then inspect the final diff.
- [x] Focused test: `cd codex-rs && just test -p codex-network-proxy credential_broker`.
- [x] Full proxy regression: `cd codex-rs && just test -p codex-network-proxy`.
- [x] Core integration: `cd codex-rs && just test -p codex-core network_proxy_spec`.
- [x] Dependency parity: `just bazel-lock-update && just bazel-lock-check`.
- [x] TUI applicability: none; no user-facing surface changes.

## Exit evidence

- [x] Implementation commit and changed paths recorded in `qa/security-levels/sprints/PF-13-S03/evidence.md`.
- [x] Captured exact-host request and denial matrix linked under `qa/security-levels/sprints/PF-13-S03/`.
- [x] Review confirms the scoped proxy route stores no raw credential across requests.
- [x] Ledgers reflect reality and the completed record is archived.
