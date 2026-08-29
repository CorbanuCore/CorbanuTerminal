---
sprint_id: "PF-32-S02"
title: "Existing search adapter and native bypass closure"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-32"
execution_order: 54
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-32-S01"
created: 2026-08-28
updated: 2026-08-28
---

# PF-32-S02 — Existing search adapter and native bypass closure

## Execution mandate

- Deliver: No native provider search path can bypass pre-model screening in Moderate/Aggressive.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-32).
- Feature: `PF-32`.
- Product citation: **Non-negotiable controls** — “Classify instruction intent and provenance before external content can influence tools or financial actions.”
- Acceptance advanced: No native provider search path can bypass pre-model screening in Moderate/Aggressive.
- Sources and archive disposition: [PF-32 reconciliation](../../../plans/security-source-reconciliation.md#pf-32).

## Code boundaries

- OpenClaw adoption reference: [OC-1](../../../plans/openclaw-source-review-2026-08-28.md#oc-1), [OC-10](../../../plans/openclaw-source-review-2026-08-28.md#oc-10) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: codex-rs/ext/web-search/src/tool.rs::WebSearchTool; codex-rs/core/src/web_search.rs; codex-rs/core/tests/suite/web_search.rs.
- Planned: codex-rs/ext/web-search/src/broker/existing.rs.
- Tests: planned colocated Rust test modules prefixed `pf_32_s02`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [ ] Active plan; PF-32-S01 completed and archived.
- [ ] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [ ] Confirm source pins, declared crate/module paths, and backend/API availability; unresolved security prerequisites block readiness.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

## Remaining

- [ ] Prove native search/SDK handoffs cannot return unscreened results or credentials outside the facade; replay cached and error-result injection fixtures as well as successful search snippets.

- [ ] Adapt the existing SearchClient/SearchRequest through the registry; send only a policy-approved minimal query, never recent_input conversation history in protected modes.
- [ ] Enumerate native hosted web_search and tool-choice/provider-extension injection routes; suppress any route whose returned content cannot be screened before model use.
- [ ] Route content-returning search API responses through sanitizer/classifier and broker-side credential handling; reject unsupported auth or provider schemas visibly.
- [ ] Test tools advertised versus actually executed, retry/resume/tool-choice overrides, provider-native hidden search and query privacy; preserve Permissive snapshots.
- [ ] Provide secret-free route/unsupported reasons for the inspector; no automatic substitution of host browsing for missing support.
- [ ] Add named `pf_32_s02` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-web-search-extension pf_32_s02 && just test -p codex-core pf_32_s02`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: none; integration flows are re-run by PF-26-S02
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-32-S02/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
