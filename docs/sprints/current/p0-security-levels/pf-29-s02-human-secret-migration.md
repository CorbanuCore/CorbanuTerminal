---
sprint_id: "PF-29-S02"
title: "Human-reviewed credential migration and recovery"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-29"
execution_order: 36
owner: "first-free lane worker (codex, 2026-10-06)"
parallel_lane: "tui"
write_scope: "codex-rs/core/src/security/migration.rs, codex-rs/core/src/security/pf_29_s02_tests.rs, codex-rs/tui/src/security/migration.rs, codex-rs/tui/src/bottom_pane/security_migration.rs, qa/security-levels/sprints/PF-29-S02/, qa/demos/index/PF-29-S02.md, docs/sprints/current/p0-security-levels/pf-29-s02-human-secret-migration.md"
integration_gate: "PR to main under the per-sprint gate (sec-common decision 5): focused tests, GLM 5.2 tmux run, one Opus 5.5 High review, SOP videos; behind protected_mode_preflight (PF-29-S01, default off). Builds on PF-29-S01's files (inventory.rs source lines and assignment span, preflight.rs readiness, the picker and its tests, security/mod.rs and bottom_pane/mod.rs module lines), owned by PF-29-S01 in the same lane; new demo specs qa/demos/specs/pf29s02-*.toml under the directory PF-23-S01 reserves (new files only); the plan worktree coordinates and the index row."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf-29-s02-20261006"
branch: "pf-29-s02-20261006"
base_commit: "72d9a5dfbf01370d481630742d038c317c2dd624"
depends_on: "PF-29-S01, PF-24-S01"
merged_behind_flag: "protected_mode_preflight"
gate_evidence: "qa/security-levels/sprints/PF-29-S02/README.md"
created: 2026-08-28
updated: 2026-10-06
---

# PF-29-S02 — Human-reviewed credential migration and recovery

**October 6:** merged (PR #235, `65d42158d7`) behind `protected_mode_preflight`; returned to `draft` for its open items.

## Execution mandate

- Deliver: Migration is explicit, encrypted, recoverable and re-audited before Moderate/Aggressive becomes active.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-29).
- Feature: `PF-29`.
- Product citation: **Non-negotiable controls** — “Permit agents to reference credentials only by label; resolve them solely inside the trusted execution boundary.”
- Acceptance advanced: Migration is explicit, encrypted, recoverable and re-audited before Moderate/Aggressive becomes active.
- Sources and archive disposition: [PF-29 reconciliation](../../../plans/security-source-reconciliation.md#pf-29).

## Code boundaries

- OpenClaw adoption reference: [OC-6](../../../plans/openclaw-source-review-2026-08-28.md#oc-6) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Core: `core/src/security/migration.rs` (plan, journal, run, recover), exported as `protected_preflight::migration`.
- TUI: `tui/src/security/migration.rs` (vault store, debug-only failure hook); screens in
  `bottom_pane/security_migration.rs` and the `/security` picker (PF-29-S01).
- Flag: `protected_mode_preflight` (PF-29-S01). Tests: `pf_29_s02` in codex-core (6) and codex-tui (4).

## Preconditions

- [x] Active plan; PF-24-S01 archived. PF-29-S01 merged behind `protected_mode_preflight` (#228) with gate evidence,
  not archived; started on that basis (PF-28-S01 precedent), same lane and worktree lineage.
- [x] Read root and nearest AGENTS.md; coordinates recorded above and in the plan.
- [x] Module paths as planned. Migration is offered from the Aggressive review; Moderate is not offered yet.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input.
- [x] Preview (`m` in a blocked Aggressive review): source IDs and locations, vault labels, the reference each line
  becomes, owner-only access, restart, unsupported items with what to do, rotation. No values. Esc changes nothing.
- [x] Confirm rechecks the reviewed preflight; any drift moves nothing. Values go to the encrypted vault first
  (`ManualSecret`, existing labels never replaced), then each line is replaced atomically by
  `"$(corbanu vault auth-helper LABEL)"` (fish: `(…)`), `0600`, synced. No plaintext backup or rollback copy.
- [x] Journal `security_migration.toml`: created exclusively (one owner), labels and stages only, generation per
  write. A line is rewritten only while it still holds the stored value; an unstored entry is skipped if its file
  changed after confirmation, so a later owner is never overwritten. Linked profiles are not rewritten.
- [x] Interruption at any stage (prepared, stored, rewritten, committed; store failure) keeps the level, reports
  "Migration stopped" and locks Aggressive (preflight readiness) until `r` recovers. Recovery only rolls forward,
  converges on the uninterrupted result, never restores plain text and never writes the level.
- [x] After a move or recovery the review reruns the preflight (re-audit) before Aggressive can be saved; saving
  resets activation, so earlier conversations stay unresumable (PF-29-S01). Rotation is listed; nothing is rotated.

## Remaining

- [ ] Only shell-profile exports are migrated; config literals, env variables and memories are listed with actions.
- [ ] Broker leases and other live capabilities are not revoked on migration (restart covers this session).

## Verification

- [x] `just fix -p codex-core -p codex-tui`, `just fmt`; final diff inspected.
- [x] Focused `pf_29_s02`: core 6, tui 4 (with the PF-29-S01 tests: core 20, tui 12).
- [x] Suites, three GLM 5.2 videos, Opus review (changes, then approved); CI green; merged as #235 ([evidence](../../../../qa/security-levels/sprints/PF-29-S02/README.md)).
- [ ] PF-26 final-candidate requalification (milestone gate).

## Exit evidence

- [x] Commits, commands, outcomes, videos and review under `qa/security-levels/sprints/PF-29-S02/`.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
