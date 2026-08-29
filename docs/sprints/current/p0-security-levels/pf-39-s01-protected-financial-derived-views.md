---
sprint_id: "PF-39-S01"
title: "Protected financial derived views"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-39"
execution_order: 66
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-28-S01, PF-38-S01"
created: 2026-08-28
updated: 2026-08-28
---

# PF-39-S01 — Protected financial derived views

## Execution mandate

- Deliver: The model receives only explicitly scoped derived financial information, not raw portfolio records.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-39).
- Feature: `PF-39`.
- Product citation: **Non-negotiable controls** — “Keep vault values, seeds, private keys, broker credentials, balances, positions, PNL, and identifying financial data out of model-visible context except for narrowly scoped derived values.”
- Acceptance advanced: The model receives only explicitly scoped derived financial information, not raw portfolio records.
- Sources and archive disposition: [PF-39 reconciliation](../../../plans/security-source-reconciliation.md#pf-39).

## Code boundaries

- OpenClaw adoption reference: [OC-3](../../../plans/openclaw-source-review-2026-08-28.md#oc-3), [OC-11](../../../plans/openclaw-source-review-2026-08-28.md#oc-11) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: codex-rs/wallet/src/balance.rs; PF-16 resource/action checks.
- Planned: codex-rs/secret-broker/src/financial/derived_view.rs.
- Tests: planned colocated Rust test modules prefixed `pf_39_s01`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [ ] Active plan; PF-28-S01, PF-38-S01 completed and archived.
- [ ] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [ ] Confirm source pins, declared crate/module paths, and backend/API availability; unresolved security prerequisites block readiness.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

## Remaining

- [ ] Keep source/protected lineage on derived financial views, including missing/corrupt memory metadata; masking recognizable token strings does not establish declassification of financial data.

- [ ] Classify balances, positions, PNL, account identifiers and financial records as protected; keep raw account reads outside model/tool context.
- [ ] Define bounded purpose-specific derived schemas such as affordability boolean or authorized aggregate; attach units, freshness, source and disclosure authority without leaking raw rows.
- [ ] Require exact human-approved resource/purpose/precision/expiry in Aggressive; Moderate permits only deterministic authorized derived views, never general portfolio dump.
- [ ] Prevent reconstruction through repeated queries with precision/query budgets, overlap checks and audit; avoid error/log leakage and unauthorized cross-account joins.
- [ ] Test portfolio extraction paraphrases, tiny-difference/repeated queries, stale values, forged view labels and raw upstream failures using synthetic accounts.
- [ ] Add named `pf_39_s01` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-secret-broker pf_39_s01 && just test -p codex-wallet pf_39_s01`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: none; integration flows are re-run by PF-26-S02
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-39-S01/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
