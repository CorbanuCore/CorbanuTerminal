---
sprint_id: "PF-41-S02"
title: "Tamper-evident audit and safe support export"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-41"
execution_order: 72
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-41-S01, PF-41-S03"
created: 2026-08-28
updated: 2026-08-28
---

# PF-41-S02 — Tamper-evident audit and safe support export

## Execution mandate

- Deliver: Security decisions are inspectable and exportable without creating another secret-disclosure channel.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-41).
- Feature: `PF-41`.
- Product citation: **Non-negotiable controls** — “Record tamper-evident policy decisions, tool calls, approvals, signatures, and transaction or order IDs without secrets.”
- Acceptance advanced: Security decisions are inspectable and exportable without creating another secret-disclosure channel.
- Sources and archive disposition: [PF-41 reconciliation](../../../plans/security-source-reconciliation.md#pf-41).

## Code boundaries

- OpenClaw adoption reference: [OC-2](../../../plans/openclaw-source-review-2026-08-28.md#oc-2), [OC-3](../../../plans/openclaw-source-review-2026-08-28.md#oc-3), [OC-7](../../../plans/openclaw-source-review-2026-08-28.md#oc-7) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: PF-19 event history; PF-28 gate; PF-38 receipts; PF-40 sanitized events.
- Planned: codex-rs/core/src/security/audit.rs; codex-rs/tui/src/bottom_pane/security_audit_export.rs.
- Tests: planned colocated Rust test modules prefixed `pf_41_s02`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [ ] All dependencies in front matter are completed and archived; plan remains active.
- [ ] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [ ] Confirm source pins, declared crate/module paths, and backend/API availability; unresolved security prerequisites block readiness.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

## Remaining

- [ ] Compose the completed PF-41-S03 producer chains and recovery semantics; this sprint owns inspection/export integration, not a late incompatible event-ID redesign. Audit failure cannot block emergency restriction or hide uncertain effects.

- [ ] Record payload-free authorization/lifecycle events and fully scrub support exports; prove tamper detection, missing-event handling and protected-sink coverage independently of diagnostic logs and prefix/suffix masks.

- [ ] Verify the shared PF-41-S03 event IDs and tamper-evident chaining across level changes, requests/decisions, grants, use/revoke, ingress outcomes, broker actions and financial receipts.
- [ ] Bound retention and encrypt sensitive metadata at rest; detect gaps, truncation and corruption without pretending a local hash chain defeats a fully compromised host.
- [ ] Provide human-initiated minimized support export with explicit destination/content preview and PF-39 disclosure checks; omit credential refs usable as capabilities, raw prompts and financial records.
- [ ] Test rotation, disk full, crash/restart, tampering, invalid chain, export cancel and secret canaries in every error path.
- [ ] Expose audit integrity/failure in the inspector and pause affected protected operations where a required audit record cannot be committed.
- [ ] Add named `pf_41_s02` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-core pf_41_s02 && just test -p codex-tui pf_41_s02`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: required: inspect decision chain → preview safe export → Esc → approve exact file → tamper detected after restart.
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-41-S02/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
