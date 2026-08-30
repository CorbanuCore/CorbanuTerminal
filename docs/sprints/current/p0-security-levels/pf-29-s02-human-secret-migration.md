---
sprint_id: "PF-29-S02"
title: "Human-reviewed credential migration and recovery"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-29"
execution_order: 36
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-29-S01, PF-24-S01"
created: 2026-08-28
updated: 2026-08-28
---

# PF-29-S02 — Human-reviewed credential migration and recovery

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

- Existing/foundation: codex-rs/tui/src/app/config_persistence.rs; codex-rs/tui/src/bottom_pane/approval_overlay.rs; PF-29 inventory.
- Planned: codex-rs/core/src/security/migration.rs; codex-rs/tui/src/bottom_pane/security_migration.rs.
- Tests: planned colocated Rust test modules prefixed `pf_29_s02`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [ ] Active plan; PF-29-S01, PF-24-S01 completed and archived.
- [ ] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [ ] Confirm source pins, declared crate/module paths, and backend/API availability; unresolved security prerequisites block readiness.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

## Remaining

- [ ] Crash at each ownership transfer/commit/cleanup point; stale rollback cannot overwrite a later owner or restore a revoked level. No plaintext backup/recovery copy; contaminated resume requires a genuinely clean context.

- [ ] Port preflight/consent and post-commit-publication-failure cases; add power-loss, encrypted recovery, stale preview and concurrent owner changes. Prove whole-migration recovery rather than inferring it from atomic individual file replacement or best-effort rollback.

- [ ] Present source IDs, destinations, access restrictions, restart requirements and unsupported items without secret values; require human confirmation and recheck manifest hashes before writes.
- [ ] Move supported credentials to encrypted vault-backed storage, replace configuration values with references and restrict originals; never make plaintext rollback copies.
- [ ] Journal an atomic migration with encrypted recovery data and durable stage markers; crash/partial failure retains the prior level but locks affected unsafe routes, never announces protected-mode success.
- [ ] Re-audit before activation, invalidate old sessions/capabilities and require clean context when historic secret exposure cannot be excluded; retain recovery material only for a bounded approved period.
- [ ] Cancel changes nothing; recovery does not restore agent-readable plaintext. Tell the human which external credentials require rotation/revocation; do not silently rotate or delete unrelated data.
- [ ] Add named `pf_29_s02` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-core pf_29_s02 && just test -p codex-tui pf_29_s02`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: required: /security preflight → preview → Esc unchanged → confirm → injected failure → recovery → restart; actual keys and sanitized artifacts.
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-29-S02/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
