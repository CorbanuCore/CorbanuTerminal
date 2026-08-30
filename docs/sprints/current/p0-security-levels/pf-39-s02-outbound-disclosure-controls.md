---
sprint_id: "PF-39-S02"
title: "Outbound disclosure clipboard and export controls"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-39"
execution_order: 67
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-39-S01, PF-30-S03, PF-32-S06"
created: 2026-08-28
updated: 2026-08-28
---

# PF-39-S02 — Outbound disclosure clipboard and export controls

## Execution mandate

- Deliver: Protected data cannot escape through a non-chat sink or an indirect export route.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-39).
- Feature: `PF-39`.
- Product citation: **Non-negotiable controls** — “Keep vault values, seeds, private keys, broker credentials, balances, positions, PNL, and identifying financial data out of model-visible context except for narrowly scoped derived values.”
- Acceptance advanced: Protected data cannot escape through a non-chat sink or an indirect export route.
- Sources and archive disposition: [PF-39 reconciliation](../../../plans/security-source-reconciliation.md#pf-39).

## Code boundaries

- OpenClaw adoption reference: [OC-3](../../../plans/openclaw-source-review-2026-08-28.md#oc-3), [OC-9](../../../plans/openclaw-source-review-2026-08-28.md#oc-9) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: codex-rs/core/src/tools/router.rs; codex-rs/tui/src; PF-28 output gate.
- Sink adapters: `codex-rs/tui/src/clipboard_copy.rs`; `codex-rs/external-agent-migration/src/sessions/export.rs`; `codex-rs/model-provider/src/provider.rs`; `codex-rs/ext/web-search/src/tool.rs`.
- Planned: codex-rs/core/src/security/outbound_disclosure.rs; codex-rs/tui/src/bottom_pane/disclosure_preview.rs.
- Tests: planned colocated Rust test modules prefixed `pf_39_s02`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [ ] Active plan; PF-39-S01, PF-30-S03, PF-32-S06 completed and archived.
- [ ] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [ ] Confirm source pins, declared crate/module paths, and backend/API availability; unresolved security prerequisites block readiness.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

## Remaining

- [ ] Test full-value redaction and authority-aware disclosure at each outbound/clipboard/export sink, including short values, encoded/chunked output and redirect/failover paths; diagnostic partial masks are insufficient.

- [ ] Inventory model-provider requests, search queries, MCP/plugin/tool arguments, email/social/trollbox posts, clipboard, file/artifact exports and child handoffs as disclosure sinks.
- [ ] Require typed source/class/destination/purpose checks and exact content digest; deny raw secrets always in protected modes and default protected financial disclosure to denied.
- [ ] Allow only approved derived output with narrow expiry and safe preview; clipboard/export requires human action, not agent-written consent or indirect shell/network copy.
- [ ] Invalidate pending approvals on content, destination, taint, level or revocation change; preserve existing stricter low-level denials.
- [ ] Test encoded/fragmented exfiltration, screenshot/artifact routes, sensitive query construction, tool argument smuggling, cancel/retry and Permissive baseline.
- [ ] Add named `pf_39_s02` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-core pf_39_s02 && just test -p codex-tui pf_39_s02`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: required: denied synthetic portfolio export → preview scoped derived output → Esc → approve exact destination → tamper/revoke denied.
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-39-S02/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
