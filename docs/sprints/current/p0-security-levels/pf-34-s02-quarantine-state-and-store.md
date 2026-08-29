---
sprint_id: "PF-34-S02"
title: "Quarantine state and encrypted retention"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-34"
execution_order: 51
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-35-S03, PF-41-S03"
created: 2026-08-28
updated: 2026-08-28
---

# PF-34-S02 — Quarantine state and encrypted retention

## Execution mandate

- Deliver: Detection causes an enforceable pre-model state transition that survives failures and restart.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-34).
- Feature: `PF-34`.
- Product citation: **Non-negotiable controls** — “Classify instruction intent and provenance before external content can influence tools or financial actions.”
- Acceptance advanced: Detection causes an enforceable pre-model state transition that survives failures and restart.
- Sources and archive disposition: [PF-34 reconciliation](../../../plans/security-source-reconciliation.md#pf-34).

## Code boundaries

- OpenClaw adoption reference: [OC-10](../../../plans/openclaw-source-review-2026-08-28.md#oc-10), [OC-11](../../../plans/openclaw-source-review-2026-08-28.md#oc-11) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: PF-34 artifact store; PF-35 screening outcomes.
- Planned: codex-rs/content-security/src/{quarantine,retention}.rs.
- Tests: planned colocated Rust test modules prefixed `pf_34_s02`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [ ] All dependencies in front matter are completed and archived; plan remains active.
- [ ] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [ ] Confirm source pins, declared crate/module paths, and backend/API availability; unresolved security prerequisites block readiness.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

## Remaining

- [ ] Use PF-41-S03 event IDs, durable acknowledgment and ownership recovery; failed required audit commit blocks release of content. Test real store/producer coupling, not only a mock chain.

- [ ] Persist provenance/content identity for cached and spilled artifacts with reservation-owned updates and safe capacity exhaustion; missing/corrupt lineage is quarantine, not trusted-local default. Keep raw retention outside ordinary workspace reads.

- [ ] Implement allow/sanitize-rescan/quarantine/reject state transitions with immutable source/digest/decision IDs and bounded retry counts.
- [ ] Keep raw artifacts encrypted with bounded disk quota, TTL, owner access and durable redacted audit chain; deny ingestion before any model-visible prefix.
- [ ] Separate retrieval failure from hostile/review-required content; restart restores restrictions rather than treating undecided content as safe.
- [ ] Allow human-triggered re-fetch/re-scan of exact content, not a generic trust or classifier bypass flag; preserve taint after release.
- [ ] Test quota/key loss, crash during write, expired retention, changed digest, repeated rescan and concurrent release/revoke races.
- [ ] Add named `pf_34_s02` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-content-security pf_34_s02`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: none; integration flows are re-run by PF-26-S02
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-34-S02/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
