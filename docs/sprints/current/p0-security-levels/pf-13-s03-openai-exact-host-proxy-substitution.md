---
sprint_id: "PF-13-S03"
title: "OpenAI exact-host proxy substitution"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-13"
execution_order: 23
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-13-S02, PF-13-S06, PF-22-S02"
created: 2026-08-24
updated: 2026-08-28
---

# PF-13-S03 — OpenAI exact-host proxy substitution

## Execution mandate

- Deliver: substitute one vault-backed bearer credential only for approved OpenAI HTTPS requests.
- Excludes: other providers, redirects, arbitrary headers, legacy CLI gating, receipts, and TUI.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-13`
- Reconciliation: [source decisions and archive mapping](../../../plans/security-source-reconciliation.md).
- Product citation: **P0 `/security` levels** — “Existing approval, sandbox, vault, wallet, tool, network, and agent policies are unchanged.”
- Acceptance advanced: approved `POST https://api.openai.com/v1/*` receives `Authorization: Bearer <secret>` only at transport.

## Code boundaries

- OpenClaw adoption reference: [OC-1](../../../plans/openclaw-source-review-2026-08-28.md#oc-1), [OC-2](../../../plans/openclaw-source-review-2026-08-28.md#oc-2), [OC-9](../../../plans/openclaw-source-review-2026-08-28.md#oc-9) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing: `network-proxy/src/credential_broker.rs::CredentialBroker`; `credential_broker/providers/openai.rs`
- Planned: `network-proxy/src/credential_broker/resolver.rs`; Core adapter in `core/src/config/network_proxy_spec.rs`
- Tests: `network-proxy/src/credential_broker_tests.rs`; affected Cargo/Bazel files

## Preconditions

- [ ] All dependencies in front matter are completed and archived.
- [ ] Read root, `codex-rs/AGENTS.md`, and `codex-rs/core/AGENTS.md`.
- [ ] Exact worktree coordinates match the active plan.

## Done

- [x] Sprint record names the provider, scheme, host, method, path, and header.

## Remaining

- [ ] Enforce the granted operation and usage reservation at transport, not merely POST /v1/*; reject an adjacent valid provider operation, changed model/resource and parallel budget overspend. Reserve worst-case bounded usage before dispatch; use trusted metering/reconciliation, retaining uncertainty after ambiguous failures.

- [ ] Port reference cases for exact host versus traffic allowlist, changed HTTPS port/method/path and unknown references. Authorize the complete operation before dispatch; a late streaming-substitution error must not be treated as proof that no earlier bytes or side effects occurred.

- [ ] Replace broker-owned `real_value: String` state with a capability reference plus a non-secret resolver interface.
- [ ] Pass scheme, normalized host, port, method, path, actor/session/task, and capability id into validation before resolution.
- [ ] Permit only HTTPS, default port 443, exact host `api.openai.com`, method `POST`, path prefix `/v1/`, and the OpenAI bearer header.
- [ ] Resolve at header injection, zeroize immediately afterward, and refuse redirect reuse or caller-supplied conflicting authorization.
- [ ] Deny lookalike/subdomain hosts, alternate ports/schemes/methods/paths, redirects, stale references, and explicit-header collisions.
- [ ] Preserve existing Permissive broker behavior outside the new opt-in route.

## Verification

- [ ] Fix: `cd codex-rs && just fix -p codex-network-proxy && just fix -p codex-core`.
- [ ] Format: `cd codex-rs && just fmt`; then inspect the final diff.
- [ ] Focused test: `cd codex-rs && just test -p codex-network-proxy credential_broker`.
- [ ] Core integration: `cd codex-rs && just test -p codex-core network_proxy_spec`.
- [ ] TUI applicability: none; no user-facing surface changes.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Captured exact-host request and denial matrix linked under `qa/security-levels/sprints/PF-13-S03/`.
- [ ] Review confirms the proxy stores no raw credential across requests.
- [ ] Ledgers reflect reality and the completed record is archived.
