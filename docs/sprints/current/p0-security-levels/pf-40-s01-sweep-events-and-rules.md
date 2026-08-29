---
sprint_id: "PF-40-S01"
title: "Agent Sweep sanitized events and deterministic rules"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-40"
execution_order: 68
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-30-S03, PF-38-S03, PF-39-S02, PF-41-S03"
created: 2026-08-28
updated: 2026-08-28
---

# PF-40-S01 — Agent Sweep sanitized events and deterministic rules

## Execution mandate

- Deliver: Behavioral anomalies can stop protected work using deterministic policy and secret-free evidence.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-40).
- Feature: `PF-40`.
- Product citation: **Non-negotiable controls** — “Support allowlists, denylists, rate limits, daily loss/notional/leverage caps, cooldowns, revocation, and a kill switch.”
- Acceptance advanced: Behavioral anomalies can stop protected work using deterministic policy and secret-free evidence.
- Sources and archive disposition: [PF-40 reconciliation](../../../plans/security-source-reconciliation.md#pf-40).

## Code boundaries

- OpenClaw adoption reference: [OC-5](../../../plans/openclaw-source-review-2026-08-28.md#oc-5), [OC-11](../../../plans/openclaw-source-review-2026-08-28.md#oc-11) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: PF-16/19 decision and revocation records; PF-28 output gate.
- Planned: codex-rs/core/src/security/sweep/{events,rules}.rs.
- Tests: planned colocated Rust test modules prefixed `pf_40_s01`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [ ] All dependencies in front matter are completed and archived; plan remains active.
- [ ] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [ ] Confirm source pins, declared crate/module paths, and backend/API availability; unresolved security prerequisites block readiness.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

## Remaining

- [ ] Consume PF-41-S03 producer/deduplication/commit contracts; saturation, chain gaps and corrupt inputs must become visible degradation, not implicit grants or healthy audit state.

- [ ] Carry host-assigned source and lineage through sanitized Sweep events; missing metadata must not become trusted merely because it came from a transcript or memory file. Keep monitoring separate from synchronous prevention.

- [ ] Emit sanitized tool/authority/destination/rate/budget/provenance/denial events with stable IDs and integrity chain; exclude secrets, raw prompts and protected financial payloads.
- [ ] Implement bounded deterministic rules for repeated extraction attempts, privilege escalation, anomalous destinations, rapid retries, cap evasion and mandate drift.
- [ ] Define severity and automatic pause/revoke/kill actions with explainable reason IDs; never lower security level or authorize financial actions.
- [ ] Bound event queues and retention; overflow, missing critical audit integrity or monitor failure visibly pauses affected protected work rather than silently losing protection.
- [ ] Test benign workload false positives, hostile sequences, event tampering, concurrent workers and restart/revocation races.
- [ ] Add named `pf_40_s01` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-core pf_40_s01`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: none; integration flows are re-run by PF-26-S02
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-40-S01/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
