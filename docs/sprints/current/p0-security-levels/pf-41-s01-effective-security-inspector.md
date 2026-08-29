---
sprint_id: "PF-41-S01"
title: "Effective security inspector and degradation state"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-41"
execution_order: 71
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-23-S03, PF-29-S02, PF-32-S06, PF-37-S02, PF-40-S03, PF-24-S02"
created: 2026-08-28
updated: 2026-08-28
---

# PF-41-S01 — Effective security inspector and degradation state

## Execution mandate

- Deliver: The user can inspect the protection actually enforced, including degradation and recent denials.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-41).
- Feature: `PF-41`.
- Product citation: **Non-negotiable controls** — “Record tamper-evident policy decisions, tool calls, approvals, signatures, and transaction or order IDs without secrets.”
- Acceptance advanced: The user can inspect the protection actually enforced, including degradation and recent denials.
- Sources and archive disposition: [PF-41 reconciliation](../../../plans/security-source-reconciliation.md#pf-41).

## Code boundaries

- OpenClaw adoption reference: [OC-6](../../../plans/openclaw-source-review-2026-08-28.md#oc-6), [OC-7](../../../plans/openclaw-source-review-2026-08-28.md#oc-7), [OC-8](../../../plans/openclaw-source-review-2026-08-28.md#oc-8) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: codex-rs/tui/src/slash_command.rs; PF-22 runtime policy; PF-24 security view.
- Planned: codex-rs/core/src/security/inspection.rs; codex-rs/tui/src/bottom_pane/security_inspector.rs.
- Tests: planned colocated Rust test modules prefixed `pf_41_s01`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [ ] Active plan; PF-23-S03, PF-29-S02, PF-32-S06, PF-37-S02, PF-40-S03, PF-24-S02 completed and archived.
- [ ] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [ ] Confirm source pins, declared crate/module paths, and backend/API availability; unresolved security prerequisites block readiness.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

## Remaining

- [ ] Display configured, resolved and observed protection separately, including broker/engine health, snapshot generation, incomplete inventory and stale probes. sandboxExplain-style policy resolution alone cannot produce a green live-protection claim.

- [ ] Expose a read-only snapshot of requested/effective level, actual sandbox/backend, egress path, broker/classifier health, active references/grants and expiry, taint, retention and recent denial reasons.
- [ ] Show resolved runtime facts and their source/generation, not just configuration intent; label unsupported or degraded components and prevent a misleading healthy protected-mode badge.
- [ ] Correlate session/task/child policy and recent decisions without returning secret values, opaque authorization tokens or raw financial records.
- [ ] Offer trusted navigation from /security to grants, migration, quarantine and Sweep; inspector reading cannot mutate authority or clear a stop.
- [ ] Test conflicting config, unsupported platform, stale health, broker crash, expired grant, tainted memory and inherited stricter child policy.
- [ ] Add named `pf_41_s01` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-core pf_41_s01 && just test -p codex-tui pf_41_s01`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: required: open /security inspector → inspect backend/taint/grant → inject failure → visible blocked state → recover/restart.
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-41-S01/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
