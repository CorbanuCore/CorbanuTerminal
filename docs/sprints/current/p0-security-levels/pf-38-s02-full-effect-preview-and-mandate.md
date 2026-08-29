---
sprint_id: "PF-38-S02"
title: "Full-effect financial preview and exact mandate"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-38"
execution_order: 64
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-38-S01, PF-25-S01"
created: 2026-08-28
updated: 2026-08-28
---

# PF-38-S02 — Full-effect financial preview and exact mandate

## Execution mandate

- Deliver: Human approval binds the complete supported financial effect rather than an agent-written summary.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-38).
- Feature: `PF-38`.
- Product citation: **Non-negotiable controls** — “Simulate and display the complete expected effect before signing.”
- Acceptance advanced: Human approval binds the complete supported financial effect rather than an agent-written summary.
- Sources and archive disposition: [PF-38 reconciliation](../../../plans/security-source-reconciliation.md#pf-38).

## Code boundaries

- OpenClaw adoption reference: [OC-2](../../../plans/openclaw-source-review-2026-08-28.md#oc-2), [OC-5](../../../plans/openclaw-source-review-2026-08-28.md#oc-5) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: codex-rs/tui/src/bottom_pane/approval_overlay.rs; PF-18 canonical mandates.
- Planned: codex-rs/secret-broker/src/financial/preview.rs; codex-rs/tui/src/bottom_pane/financial_preview.rs.
- Tests: planned colocated Rust test modules prefixed `pf_38_s02`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [ ] Active plan; PF-38-S01, PF-25-S01 completed and archived.
- [ ] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [ ] Confirm source pins, declared crate/module paths, and backend/API availability; unresolved security prerequisites block readiness.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

## Remaining

- [ ] Validate full-effect authorization before dispatch, including tainted inputs and streamed/late validation failures; source wrappers, allowed hostnames and earlier approvals do not authorize changed financial effects.

- [ ] Simulate or deterministically derive the complete expected effect for supported actions; show account/destination/asset, amount, fees, slippage, approvals/allowances, worst-case bounds and expiry.
- [ ] Block unsupported/ambiguous effects instead of claiming a complete preview; the model may describe intent but cannot populate trusted effect fields.
- [ ] Bind human approval to canonical action digest, simulation/quote version, limits, identity, level and revocation generation; reject changes and stale previews.
- [ ] Require exact human approval for each sign and each broadcast in Aggressive; Moderate follows deterministic risk/mandate policy, never inferred conversational consent.
- [ ] Test cancel, modified amount/recipient, changed fee/quote, side effects, spoofed preview labels, timeout and approve-after-revoke; include real-key TUI proof.
- [ ] Add named `pf_38_s02` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-secret-broker pf_38_s02 && just test -p codex-tui pf_38_s02`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: required: inspect complete fake action → Esc → alter recipient/quote → stale denial → approve exact refreshed preview.
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-38-S02/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
