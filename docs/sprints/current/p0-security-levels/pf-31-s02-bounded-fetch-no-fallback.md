---
sprint_id: "PF-31-S02"
title: "Bounded fetch adapter with no host fallback"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-31"
execution_order: 47
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-31-S01, PF-30-S01"
created: 2026-08-28
updated: 2026-08-28
---

# PF-31-S02 — Bounded fetch adapter with no host fallback

## Execution mandate

- Deliver: Every fetch stays in the isolated lane and failure never falls back to a privileged browser.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-31).
- Feature: `PF-31`.
- Product citation: **Non-negotiable controls** — “Default to no secret export, arbitrary egress, clipboard exposure, or sensitive logging.”
- Acceptance advanced: Every fetch stays in the isolated lane and failure never falls back to a privileged browser.
- Sources and archive disposition: [PF-31 reconciliation](../../../plans/security-source-reconciliation.md#pf-31).

## Code boundaries

- OpenClaw adoption reference: [OC-9](../../../plans/openclaw-source-review-2026-08-28.md#oc-9), [OC-10](../../../plans/openclaw-source-review-2026-08-28.md#oc-10) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: codex-rs/ext/web-search/src/{tool,output}.rs.
- Planned: codex-rs/web-retriever/src/{fetch,protocol}.rs; codex-rs/core/src/security/retrieval.rs.
- Tests: planned colocated Rust test modules prefixed `pf_31_s02`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [ ] Active plan; PF-31-S01, PF-30-S01 completed and archived.
- [ ] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [ ] Confirm source pins, declared crate/module paths, and backend/API availability; unresolved security prerequisites block readiness.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

## Remaining

- [ ] Test abort after headers, unread/error/oversize response cleanup, decompressed/decoded limits and sibling-request preservation. Cache hits and provider fallback still cross the same screening/provenance boundary.

- [ ] Define typed fetch/click/find requests with source IDs, parent navigation identity, policy generation and bounded budgets; no arbitrary JavaScript or browser automation command.
- [ ] Return untrusted raw artifact references and content metadata only to the sanitizer lane; never stream raw page content straight into model context.
- [ ] Fail closed for worker crash, timeout, unsupported content, navigation mismatch, blocked origin or unavailable sandbox; forbid host-browser and unscreened API fallback.
- [ ] Route login, CAPTCHA, MFA, payment and access challenges to a human-required state without automating circumvention or forwarding the user's host profile.
- [ ] Test visible/hidden injection pages, click-origin changes, redirects, worker death, cancel/retry and replay of stale navigation handles.
- [ ] Add named `pf_31_s02` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-web-retriever pf_31_s02 && just test -p codex-core pf_31_s02`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: none; integration flows are re-run by PF-26-S02
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-31-S02/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
