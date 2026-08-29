---
sprint_id: "PF-40-S03"
title: "Agent Sweep alerts revocation and recovery"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-40"
execution_order: 70
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-40-S02, PF-25-S02"
created: 2026-08-28
updated: 2026-08-28
---

# PF-40-S03 — Agent Sweep alerts revocation and recovery

## Execution mandate

- Deliver: A user can understand and recover from a security stop without reviving stale authority.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-40).
- Feature: `PF-40`.
- Product citation: **Non-negotiable controls** — “Support allowlists, denylists, rate limits, daily loss/notional/leverage caps, cooldowns, revocation, and a kill switch.”
- Acceptance advanced: A user can understand and recover from a security stop without reviving stale authority.
- Sources and archive disposition: [PF-40 reconciliation](../../../plans/security-source-reconciliation.md#pf-40).

## Code boundaries

- OpenClaw adoption reference: [OC-2](../../../plans/openclaw-source-review-2026-08-28.md#oc-2), [OC-7](../../../plans/openclaw-source-review-2026-08-28.md#oc-7) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: codex-rs/tui/src/bottom_pane/approval_overlay.rs; PF-25 kill/revoke UI.
- Planned: codex-rs/tui/src/bottom_pane/sweep_review.rs; codex-rs/core/src/security/sweep/recovery.rs.
- Tests: planned colocated Rust test modules prefixed `pf_40_s03`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [ ] Active plan; PF-40-S02, PF-25-S02 completed and archived.
- [ ] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [ ] Confirm source pins, declared crate/module paths, and backend/API availability; unresolved security prerequisites block readiness.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

## Remaining

- [ ] Test audit unavailable and unknown financial effect during Sweep kill/recovery; stop new authority immediately, preserve unresolved receipt identity and require fresh human authority without clearing taint.

- [ ] Show observation failures and stale health explicitly; test alert-driven revocation against already-open channels and avoid representing an advisory finding or missing event as a completed prevention action.

- [ ] Show anomaly reason, affected task/authority and redacted event lineage; distinguish deterministic actions from advisory findings.
- [ ] Wire pause, revoke and kill through PF-19 durable invalidation, close pending broker/browser/financial operations and show irreversible actions already submitted.
- [ ] Require human review to resume with fresh narrow authority and current taint; retrying the same task cannot restore an old grant or lower the level.
- [ ] Provide safe false-positive recovery without erasing audit or turning off deterministic rules; preserve paused state across restart.
- [ ] Test real-key alert inspection, Esc, revoke, kill, safe recovery, expired grants, child agents and restart.
- [ ] Add named `pf_40_s03` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-core pf_40_s03 && just test -p codex-tui pf_40_s03`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: required: trigger fake anomaly → inspect → Esc remains paused → revoke/kill → restart → human fresh-grant recovery.
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-40-S03/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
