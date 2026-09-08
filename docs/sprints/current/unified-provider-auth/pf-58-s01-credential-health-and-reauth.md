---
sprint_id: "PF-58-S01"
title: "Credential-scoped runtime health and keyboard reauthentication"
status: in_progress
plan_file: "docs/plans/active/unified-provider-auth.md"
plan_feature: "PF-58"
execution_order: 22
owner: "Codex /root"
parallel_lane: "provider-reauth-health"
write_scope: "codex-rs/.config/nextest.toml, codex-rs/Cargo.lock, MODULE.bazel.lock, codex-rs/provider-auth/, codex-rs/login/, codex-rs/model-provider/, codex-rs/codex-mcp/, codex-rs/rmcp-client/, codex-rs/tui/, codex-rs/protocol/, codex-rs/app-server-protocol/, codex-rs/app-server/, codex-rs/core/src/session/mcp_runtime.rs, codex-rs/core/src/session/mcp.rs, codex-rs/core/tests/suite/mcp_auth_refresh.rs, docs/plans/active/unified-provider-auth.md, docs/sprints/current/unified-provider-auth/, docs/sprints/index.md, docs/authentication.md, docs/features/model-providers.md, qa/provider-auth/pf-58/, humanTest.html"
integration_gate: "Codex /root audits the credential-specific boundary and scope, formats then tests final source on RTX including true-TMUX and safe synthetic auth failures/recovery, obtains Astra High and Fable5.1 High reviews within five total, and preserves a separate branch/candidate before updating integration or main. No privileged setup or automatic billing-context switch."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/provider-reauth-health"
branch: "feat/provider-reauth-health"
base_commit: "1b6921112d73217e1e2a78b5adc43e8ce24764ab"
depends_on: "PF-57-S01"
created: 2026-09-05
updated: 2026-09-08
---

# PF-58-S01 — Credential-scoped runtime health and keyboard reauthentication

## Execution mandate

- Deliver: trustworthy auth-health status and source-appropriate keyboard recovery for configured providers, including the reported OpenAI connected-app expiry.
- Excludes: automatic credential/account/billing fallback, new providers, adjacent model-picker repairs, cloud login on the user's behalf, privileged setup and release claims.

## Plan linkage

- Plan: [Unified provider auth](../../../plans/active/unified-provider-auth.md), feature PF-58.
- Product heading: **Shipping MVP — LIVE**; excerpt: “Encrypted `/vault`, masked entry, metadata-only inspection, and operational credential use without placing raw values in chat.”
- User authorized generalizing reauth on 2026-09-05 after the misleading Active label.

## Code boundaries

- Existing: login AuthManager metadata/refresh, model-provider auth, MCP transport startup classification, shared provider status/controller and TUI notification/selection adapters.
- Planned: typed credential-scoped failure/recovery metadata; thin TUI keyboard/status projection; reuse established recovery backends.
- Tests: synthetic status/refresh/classification and stale-result regressions, snapshots and typed TMUX.

## Preconditions

- [x] Active plan and completed archived PF-57 dependency verified.
- [x] Exact branch/worktree/base recorded; scope disjoint from frozen broker reservation.
- [x] RTX affected credential identified from metadata only; values never exposed.

## Done

- [x] User amendment and serial allocation recorded before source changes.
- [x] Working human-test candidate left unchanged and security monitor left paused.

## Remaining

- [x] Trace refresh failure and exact credential/service identity through transport and status projection.
- [x] Implement source-scoped auth health without treating network/429/generic403 as expired credentials or disabling unrelated providers.
- [x] Reuse existing account/API-key/source recovery from an advertised keyboard action; preserve cancel/current model and reject stale recovery.
- [x] Refresh status and reconnect the affected app service after successful account recovery without process restart (synthetic account/request and core MCP transport proof; live sign-in remains a human check).
- [x] Generalize managed, environment, command/AWS and local capability handling with truthful unsupported-action guidance.
- [x] Run final focused automated/TMUX matrix and five bounded reviews; findings corrected. The fifth correction is test-verified without a sixth review under the user cap. Separate native-child integration failure remains documented.

## Verification

- [x] Final scoped fix/fmt then affected provider-auth/login/MCP/TUI tests on RTX.
- [x] True-TMUX: account failure/recovery/cancel and API-key replacement; unit/snapshot coverage for unaffected providers, stale failures and narrow rendering. Final focused TMUX run: 6/6.
- [x] Credential canaries absent from status, snapshots, transcripts and errors.
- [x] Live-repository applicability recorded; human acceptance pending in checks 24–26.

## Exit evidence

- [x] Exact changed-source manifest/candidate hashes and final test/review reports linked under qa/provider-auth/pf-58.
- [x] Human guide updated without marking user checks accepted.
- [x] Source commit `2545eedee` pushed to `origin/feat/provider-reauth-health`; not merged to main. Handoff bookkeeping follows separately.
- [ ] Ledgers accurate and completed sprint archived only after required evidence.
