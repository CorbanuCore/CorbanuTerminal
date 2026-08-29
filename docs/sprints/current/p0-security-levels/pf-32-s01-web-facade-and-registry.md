---
sprint_id: "PF-32-S01"
title: "Stable web facade and provider registry"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-32"
execution_order: 53
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-34-S03, PF-31-S03, PF-13-S05"
created: 2026-08-28
updated: 2026-08-28
---

# PF-32-S01 — Stable web facade and provider registry

## Execution mandate

- Deliver: Protected web tools have one screened facade with deterministic provider capabilities and stable provenance.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-32).
- Feature: `PF-32`.
- Product citation: **Non-negotiable controls** — “Classify instruction intent and provenance before external content can influence tools or financial actions.”
- Acceptance advanced: Protected web tools have one screened facade with deterministic provider capabilities and stable provenance.
- Sources and archive disposition: [PF-32 reconciliation](../../../plans/security-source-reconciliation.md#pf-32).

## Code boundaries

- OpenClaw adoption reference: [OC-4](../../../plans/openclaw-source-review-2026-08-28.md#oc-4), [OC-9](../../../plans/openclaw-source-review-2026-08-28.md#oc-9), [OC-10](../../../plans/openclaw-source-review-2026-08-28.md#oc-10) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: codex-rs/ext/web-search/src/{tool,schema,output,history}.rs.
- Planned: codex-rs/ext/web-search/src/broker/{mod,registry,normalized}.rs.
- Tests: planned colocated Rust test modules prefixed `pf_32_s01`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [ ] Active plan; PF-34-S03, PF-31-S03, PF-13-S05 completed and archived.
- [ ] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [ ] Confirm source pins, declared crate/module paths, and backend/API availability; unresolved security prerequisites block readiness.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

## Remaining

- [ ] Define the same source envelope and budget for native/provider, cache, fallback, error and metadata results; route every enabled facade path through the screening gate and record unsupported paths explicitly.

- [ ] Preserve the web.run input surface and normalized output/reference behavior; define typed provider search/fetch capabilities, supported commands and explicit unsupported errors.
- [ ] Separate search discovery from isolated fetch/open/click/find; provider results/snippets, citations and errors all pass PF-30/34/35 before model visibility.
- [ ] Assign stable session-scoped result IDs bound to provider, source URL, digest and policy generation; reject forged/stale references and unsafe metadata.
- [ ] Define broker-only credential references and exact service endpoints per adapter; never widen PF-13 OpenAI substitution into arbitrary headers/hosts.
- [ ] Keep Permissive on the existing facade/routing/history path and test schema/reference compatibility including non-search commands.
- [ ] Add named `pf_32_s01` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-web-search-extension pf_32_s01`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: none; integration flows are re-run by PF-26-S02
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-32-S01/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
