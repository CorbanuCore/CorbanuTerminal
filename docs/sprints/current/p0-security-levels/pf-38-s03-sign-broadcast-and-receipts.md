---
sprint_id: "PF-38-S03"
title: "Separate signing broadcasting and idempotent receipts"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-38"
execution_order: 65
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-38-S02, PF-41-S03"
created: 2026-08-28
updated: 2026-08-28
---

# PF-38-S03 — Separate signing broadcasting and idempotent receipts

## Execution mandate

- Deliver: Sign and broadcast are separate guarded operations; retries cannot silently duplicate a financial effect.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-38).
- Feature: `PF-38`.
- Product citation: **Non-negotiable controls** — “Simulate and display the complete expected effect before signing.”
- Acceptance advanced: Sign and broadcast are separate guarded operations; retries cannot silently duplicate a financial effect.
- Sources and archive disposition: [PF-38 reconciliation](../../../plans/security-source-reconciliation.md#pf-38).

## Code boundaries

- OpenClaw adoption reference: [OC-2](../../../plans/openclaw-source-review-2026-08-28.md#oc-2), [OC-6](../../../plans/openclaw-source-review-2026-08-28.md#oc-6) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: codex-rs/wallet/src/{envelope,payment}.rs; PF-19 invalidation.
- Planned: codex-rs/secret-broker/src/financial/{execute,receipt}.rs.
- Tests: planned colocated Rust test modules prefixed `pf_38_s03`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [ ] All dependencies in front matter are completed and archived; plan remains active.
- [ ] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [ ] Confirm source pins, declared crate/module paths, and backend/API availability; unresolved security prerequisites block readiness.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

## Remaining

- [ ] Use PF-41-S03 causal event/receipt IDs and durable intent/unknown recovery. With a submitted-unknown fake transfer, kill or tighten immediately, reject new sign/broadcast, restart and reconcile status without blind rebroadcast or false cancellation.

- [ ] Test stale/revoked authority and ownership-aware recovery around separate sign/broadcast operations; retries must not replay effects. Upstream credential lifecycle tests are not evidence of financial idempotence or custody safety.

- [ ] Keep custody material inside the trusted signer; validate action/approval/budget again immediately before sign and separately before broadcast.
- [ ] Persist idempotency keys, canonical digest, reservation and secret-free execution receipts with tamper-evident links; reject duplicates, conflicting requests, stale approvals and actor changes.
- [ ] Model submitted/confirmed/failed/unknown execution states; after timeout query authoritative receipt/status before any retry and never blindly re-broadcast.
- [ ] Revoke pending authority on kill/downgrade; do not claim revocation can undo an already-signed or broadcast transaction, and display irreversible effects.
- [ ] Test cancel between sign/broadcast, crash before/after submission, double-submit, receipt mutation, chain/fake-venue uncertainty and restart without using real funds.
- [ ] Add named `pf_38_s03` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-secret-broker pf_38_s03 && just test -p codex-wallet pf_38_s03`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: required: fake sign → cancel broadcast → fresh approval → uncertain submission → restart/status recovery without duplicate transfer.
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-38-S03/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
