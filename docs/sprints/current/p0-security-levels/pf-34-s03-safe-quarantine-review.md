---
sprint_id: "PF-34-S03"
title: "Safe quarantine review and recovery"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-34"
execution_order: 52
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-34-S02, PF-24-S01"
created: 2026-08-28
updated: 2026-08-28
---

# PF-34-S03 — Safe quarantine review and recovery

## Execution mandate

- Deliver: Reviewing suspicious content cannot itself execute instructions or weaken policy.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-34).
- Feature: `PF-34`.
- Product citation: **Non-negotiable controls** — “Classify instruction intent and provenance before external content can influence tools or financial actions.”
- Acceptance advanced: Reviewing suspicious content cannot itself execute instructions or weaken policy.
- Sources and archive disposition: [PF-34 reconciliation](../../../plans/security-source-reconciliation.md#pf-34).

## Code boundaries

- OpenClaw adoption reference: [OC-4](../../../plans/openclaw-source-review-2026-08-28.md#oc-4), [OC-10](../../../plans/openclaw-source-review-2026-08-28.md#oc-10) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: codex-rs/tui/src/bottom_pane/approval_overlay.rs.
- Planned: codex-rs/tui/src/bottom_pane/quarantine_review.rs.
- Tests: planned colocated Rust test modules prefixed `pf_34_s03`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [ ] Active plan; PF-34-S02, PF-24-S01 completed and archived.
- [ ] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [ ] Confirm source pins, declared crate/module paths, and backend/API availability; unresolved security prerequisites block readiness.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

## Remaining

- [ ] Test safely escaped metadata, clipped wrappers and spilled/downloaded content in human review; viewing or rescanning an artifact cannot promote it or execute embedded model/terminal controls.

- [ ] Show source, reason, safe excerpt, digest, retention and allowed recovery actions; escape terminal sequences, links and forged approval language.
- [ ] Provide reject, cancel, safe re-fetch and sanitized rescan; do not render raw active HTML, auto-follow links or let review grant tool/secret authority.
- [ ] Bind decisions to exact artifact/actor/session; altered, expired or revoked content requires fresh review and screening.
- [ ] Display classifier unavailable and storage failure as blocked ingestion with recovery guidance, never a misleading safe badge.
- [ ] Add snapshots and actual-key success, hostile rejection, cancel, failure, retry and restart tests.
- [ ] Add named `pf_34_s03` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-tui pf_34_s03 && just test -p codex-content-security pf_34_s03`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: required: open quarantine → inspect escaped excerpt → Esc → re-fetch/rescan → reject hostile → restart/resume.
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-34-S03/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
