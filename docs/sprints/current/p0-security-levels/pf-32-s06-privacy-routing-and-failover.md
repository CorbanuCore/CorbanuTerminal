---
sprint_id: "PF-32-S06"
title: "Private query routing and bounded failover"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-32"
execution_order: 58
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-32-S03, PF-32-S04, PF-32-S05"
created: 2026-08-28
updated: 2026-08-28
---

# PF-32-S06 — Private query routing and bounded failover

## Execution mandate

- Deliver: Routing and recovery preserve privacy, screening and scope instead of silently changing lanes.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-32).
- Feature: `PF-32`.
- Product citation: **Non-negotiable controls** — “Classify instruction intent and provenance before external content can influence tools or financial actions.”
- Acceptance advanced: Routing and recovery preserve privacy, screening and scope instead of silently changing lanes.
- Sources and archive disposition: [PF-32 reconciliation](../../../plans/security-source-reconciliation.md#pf-32).

## Code boundaries

- OpenClaw adoption reference: [OC-9](../../../plans/openclaw-source-review-2026-08-28.md#oc-9), [OC-10](../../../plans/openclaw-source-review-2026-08-28.md#oc-10) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: PF-32 provider adapters and registry.
- Planned: codex-rs/ext/web-search/src/broker/{router,privacy,health}.rs.
- Tests: planned colocated Rust test modules prefixed `pf_32_s06`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [ ] Active plan; PF-32-S03, PF-32-S04, PF-32-S05 completed and archived.
- [ ] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [ ] Confirm source pins, declared crate/module paths, and backend/API availability; unresolved security prerequisites block readiness.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

## Remaining

- [ ] Test that failover preserves query minimization, source identity, screening, budgets and endpoint policy; cache partitions must include relevant policy/authority generations and cancellation must not publish stale results.

- [ ] Build deterministic profile/capability/destination routing; human-configured provider precedence and Aggressive grants bound provider, request purpose, expiry and cost ceiling.
- [ ] Minimize queries to explicit search terms; deny raw portfolio/context/credential payloads, unexpected history attachments and sensitive provider diagnostics.
- [ ] Permit failover only among already-authorized same-role providers with compatible privacy terms; never fetch→host-browser, classifier→unscreened, or paid→unconsented escalation.
- [ ] Bound retries, backoff, budgets and health cache; expose selected provider, cost counters, privacy reason and failure without logging raw sensitive queries.
- [ ] Test all adapters, exhausted budget, stale health, provider outage, unsupported operation, denied fallback and cancel/restart.
- [ ] Add named `pf_32_s06` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-web-search-extension pf_32_s06`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: none; integration flows are re-run by PF-26-S02
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-32-S06/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
