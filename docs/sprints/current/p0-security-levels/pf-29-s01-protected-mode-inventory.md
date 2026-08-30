---
sprint_id: "PF-29-S01"
title: "Protected-mode inventory and activation preflight"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-29"
execution_order: 35
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-28-S02, PF-20-S02"
created: 2026-08-28
updated: 2026-08-28
---

# PF-29-S01 — Protected-mode inventory and activation preflight

## Execution mandate

- Deliver: Protected-mode activation cannot claim a clean boundary while a known raw-secret route or contaminated resume remains usable.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-29).
- Feature: `PF-29`.
- Product citation: **Non-negotiable controls** — “Permit agents to reference credentials only by label; resolve them solely inside the trusted execution boundary.”
- Acceptance advanced: Protected-mode activation cannot claim a clean boundary while a known raw-secret route or contaminated resume remains usable.
- Sources and archive disposition: [PF-29 reconciliation](../../../plans/security-source-reconciliation.md#pf-29).

## Code boundaries

- OpenClaw adoption reference: [OC-1](../../../plans/openclaw-source-review-2026-08-28.md#oc-1), [OC-6](../../../plans/openclaw-source-review-2026-08-28.md#oc-6) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: codex-rs/config/src/config_toml.rs; codex-rs/vault/src/lib.rs; codex-rs/core/src/exec_env.rs.
- Planned: codex-rs/core/src/security/{inventory,preflight}.rs.
- Tests: planned colocated Rust test modules prefixed `pf_29_s01`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [ ] Active plan; PF-28-S02, PF-20-S02 completed and archived.
- [ ] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [ ] Confirm source pins, declared crate/module paths, and backend/API availability; unresolved security prerequisites block readiness.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

## Remaining

- [ ] Represent skipped/consent-required exec-provider checks as incomplete readiness, enumerate hidden provider headers/transport auth, and distinguish agent-readable environment entries from protected credentials without exposing raw values.

- [ ] Inventory supported auth/config/env files, MCP/plugin/child launch routes, browser profiles, protected mounts, persisted transcripts and memories by safe finding IDs; do not log discovered values.
- [ ] Classify managed secrets, custody keys, financial records, permitted derived values and ordinary env data; describe heuristic/unmanaged-secret limits explicitly.
- [ ] Build a dry-run migration manifest with file identities, hashes, scope, permissions and unsupported sources; do not recursively scan arbitrary user directories or mutate data.
- [ ] Require broker/backend/output-gate readiness before a protected-mode transition; block unmediated integrations and contaminated session resumes until isolated, migrated, or restarted clean.
- [ ] Test symlinks, shadowed env/config, old memory content, denied reads, locked vault, corrupt snapshots and drift between preflight and activation.
- [ ] Add named `pf_29_s01` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-core pf_29_s01`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: none; integration flows are re-run by PF-26-S02
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-29-S01/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
