---
sprint_id: "PF-32-S03"
title: "Exa brokered search adapter"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-32"
execution_order: 55
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-32-S02"
created: 2026-08-28
updated: 2026-08-28
---

# PF-32-S03 — Exa brokered search adapter

## Execution mandate

- Deliver: Exa search is usable through the common screened contract without raw credential or history disclosure.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-32).
- Feature: `PF-32`.
- Product citation: **Non-negotiable controls** — “Classify instruction intent and provenance before external content can influence tools or financial actions.”
- Acceptance advanced: Exa search is usable through the common screened contract without raw credential or history disclosure.
- Sources and archive disposition: [PF-32 reconciliation](../../../plans/security-source-reconciliation.md#pf-32).

## Code boundaries

- OpenClaw adoption reference: [OC-1](../../../plans/openclaw-source-review-2026-08-28.md#oc-1), [OC-9](../../../plans/openclaw-source-review-2026-08-28.md#oc-9), [OC-10](../../../plans/openclaw-source-review-2026-08-28.md#oc-10) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: PF-32 provider registry and normalized results; PF-33 destination policy.
- Planned: codex-rs/ext/web-search/src/broker/exa.rs; codex-rs/ext/web-search/tests/exa.rs.
- Tests: planned colocated Rust test modules prefixed `pf_32_s03`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [ ] Active plan; PF-32-S02 completed and archived.
- [ ] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [ ] Confirm source pins, declared crate/module paths, and backend/API availability; unresolved security prerequisites block readiness.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

## Remaining

- [ ] Record the Exa adapter's own pinned API contract and tests; OpenClaw's generic handoff/fetch patterns are reference only. Require brokered auth, minimal query disclosure and screened metadata/errors without claiming this review qualified Exa.

- [ ] Pin the reviewed Exa API schema and documentation at implementation; declare exact endpoint/auth/search-only capabilities, pagination, limits and unsupported commands.
- [ ] Resolve API credentials only in the trusted broker with exact destination/method/path bindings; validate redirects and reflected output.
- [ ] Normalize titles/snippets/URLs/results through provenance, sanitation and local screening; provider output never gains authority.
- [ ] Use redacted recorded responses or local fixtures for malformed JSON, rate limit, timeout, auth failure, hostile snippets and pagination; no paid calls or live keys required for tests.
- [ ] Record health, metered usage and safe error codes; do not select this provider automatically or grant broader egress on failure.
- [ ] Add named `pf_32_s03` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-web-search-extension pf_32_s03`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: none; integration flows are re-run by PF-26-S02
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-32-S03/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
