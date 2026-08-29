---
sprint_id: "PF-34-S01"
title: "Render-aware content sanitization"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-34"
execution_order: 49
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-31-S02, PF-30-S01, PF-34-S04"
created: 2026-08-28
updated: 2026-08-28
---

# PF-34-S01 — Render-aware content sanitization

## Execution mandate

- Deliver: Only bounded sanitized content proceeds to screening, with original provenance and no implicit trust promotion.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-34).
- Feature: `PF-34`.
- Product citation: **Non-negotiable controls** — “Classify instruction intent and provenance before external content can influence tools or financial actions.”
- Acceptance advanced: Only bounded sanitized content proceeds to screening, with original provenance and no implicit trust promotion.
- Sources and archive disposition: [PF-34 reconciliation](../../../plans/security-source-reconciliation.md#pf-34).

## Code boundaries

- OpenClaw adoption reference: [OC-4](../../../plans/openclaw-source-review-2026-08-28.md#oc-4), [OC-10](../../../plans/openclaw-source-review-2026-08-28.md#oc-10) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: codex-rs/ext/web-search/src/output.rs; PF-31 raw artifact protocol.
- Planned: codex-rs/content-security/src/{sanitize,artifact}.rs.
- Tests: planned colocated Rust test modules prefixed `pf_34_s01`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [ ] All dependencies in front matter are completed and archived; plan remains active.
- [ ] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [ ] Confirm source pins, declared crate/module paths, and backend/API availability; unresolved security prerequisites block readiness.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

## Remaining

- [ ] Consume PF-34-S04 segment/transform fixtures and bind actual PF-30 source identities. Real rendering/sanitization output must remain compatible with the independently prepared classifier dataset; rerun evaluator fixtures for any schema change.

- [ ] Port expansion-safe truncation, metadata, zero-width/fullwidth and complete-before-clipped-marker cases; separately test rendered visibility and terminal controls. Wrapper heuristics alone do not establish render sanitization or classifier quality.

- [ ] Extract visible main content using rendering evidence, not just tag stripping; remove scripts/styles, hidden nodes, offscreen/CSS-hidden text, comments and non-body metadata from normal ingestion.
- [ ] Normalize terminal/control sequences, bidi/zero-width confusables, unsafe links and model special tokens with byte/token/time limits; preserve meaningful quoted/security-research text.
- [ ] Record raw and sanitized digests, transformation version and source lineage; retain raw data encrypted and quota/TTL bounded, never as an agent-readable debug dump.
- [ ] Label remaining visible instructions as untrusted; cleaning is not proof of benignness and must feed the classifier before ingestion.
- [ ] Test CSS/DOM disagreement, hidden payloads, visible malicious text, malformed HTML, encoding bombs, PDF/text fallback policy and format-unsupported quarantine.
- [ ] Add named `pf_34_s01` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-content-security pf_34_s01`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: none; integration flows are re-run by PF-26-S02
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-34-S01/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
