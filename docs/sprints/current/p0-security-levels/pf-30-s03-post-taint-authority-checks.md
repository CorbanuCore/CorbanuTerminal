---
sprint_id: "PF-30-S03"
title: "Post-taint authority checks"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-30"
execution_order: 39
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-30-S02, PF-13-S05"
created: 2026-08-28
updated: 2026-08-28
---

# PF-30-S03 — Post-taint authority checks

## Execution mandate

- Deliver: A classifier false negative or tainted summary cannot mint or widen protected authority.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-30).
- Feature: `PF-30`.
- Product citation: **Non-negotiable controls** — “Classify instruction intent and provenance before external content can influence tools or financial actions.”
- Acceptance advanced: A classifier false negative or tainted summary cannot mint or widen protected authority.
- Sources and archive disposition: [PF-30 reconciliation](../../../plans/security-source-reconciliation.md#pf-30).

## Code boundaries

- OpenClaw adoption reference: [OC-4](../../../plans/openclaw-source-review-2026-08-28.md#oc-4), [OC-5](../../../plans/openclaw-source-review-2026-08-28.md#oc-5), [OC-11](../../../plans/openclaw-source-review-2026-08-28.md#oc-11) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: codex-rs/core/src/tools/router.rs; PF-16 authorization; PF-27 broker client.
- Planned: codex-rs/core/src/security/tainted_action.rs.
- Tests: planned colocated Rust test modules prefixed `pf_30_s03`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [ ] Active plan; PF-30-S02, PF-13-S05 completed and archived.
- [ ] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [ ] Confirm source pins, declared crate/module paths, and backend/API availability; unresolved security prerequisites block readiness.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

## Remaining

- [ ] Implement the action/profile matrix using data plus control-flow ancestry; unchanged narrow grants authorize only unchanged scope. Human-origin argument labels do not declassify model-selected actions; require trusted reconstruction or exact bounded human mandate.
- [ ] Record realistic research-workflow approval counts and latency for later PF-26 validation; product-owned usability targets cannot relax deterministic denial or erase taint.

- [ ] Exercise tainted transcript to maintenance flush to memory recall to protected action, with a new user message and exact human approval interposed; preserve ancestry and authorize only the approved effect. A wrapper/classifier result cannot reset taint.

- [ ] Attach current source lineage and security-policy generation to every protected action and outbound disclosure request.
- [ ] Deny external-origin instructions that request policy changes, vault enumeration, credential extraction or unapproved actions even when a detector returns benign.
- [ ] Require fresh exact human authority for sensitive tainted follow-on actions; narrow existing grants may satisfy only their unchanged scope, never taint-driven expansion.
- [ ] Invalidate stale decisions on new taint, resume, grant change or revocation; recompute at execution rather than only at prompt ingestion.
- [ ] Test forced classifier allow, quoted malicious trades, memory-triggered exfiltration, stale approvals and child confused-deputy attempts against deterministic fake actions.
- [ ] Add named `pf_30_s03` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-core pf_30_s03 && just test -p codex-secret-broker pf_30_s03`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: none; integration flows are re-run by PF-26-S02
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-30-S03/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
