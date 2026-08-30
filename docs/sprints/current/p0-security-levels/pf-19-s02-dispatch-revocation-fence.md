---
sprint_id: "PF-19-S02"
title: "Dispatch revocation fence contract"
status: ready
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-19"
execution_order: 23
owner: "Codex revocation/fence lane"
parallel_lane: "revocation-fence"
write_scope: "codex-rs/security-policy/src/revocation.rs, codex-rs/security-policy/src/security_policy_tests.rs, qa/security-levels/sprints/PF-19-S02/, docs/sprints/current/p0-security-levels/pf-19-s02-dispatch-revocation-fence.md"
integration_gate: "The Codex ingress/classifier integration lane receives the PF-19-S02 candidate, audits the literal scope against the other two round-two lanes, rebases and merges it first, reruns the complete codex-security-policy suite and governance checkers, archives PF-19-S02, and does not activate any transport consumer."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-revocation-fence"
branch: "feat/p0-security-revocation-fence"
base_commit: "5521b681fff0ecb50b17c10bc1dd1356cbecc1b6"
depends_on: "PF-19-S01"
created: 2026-08-28
updated: 2026-08-30
---

# PF-19-S02 — Dispatch revocation fence contract

## Execution mandate

- Deliver: Extend accepted revocation types with a precise dispatch fence for queued and admitted work.
- Excludes: transport adapters, durable store wiring, financial recovery implementation and kill-switch TUI.

## Plan linkage

- Plan: [P0 security levels](../../../plans/active/p0-security-levels.md).
- Feature: `PF-19`.
- Product citation: **Reconciled security scope — TO BUILD** — “Unknown or unsupported protected paths fail visibly rather than falling back to raw secrets or unscreened execution.”
- Acceptance advanced: [architecture refinements](../../../plans/security-architecture-refinements-2026-08-28.md); preserve the completed S01 and its historical evidence, with only added guarantees in this follow-up.

## Code boundaries

- OpenClaw adoption: OC-2 in the [pinned source review](../../../plans/openclaw-source-review-2026-08-28.md), commit `13adff02ca3897768d80d2bca18f5acf08c55d91`; named source tests are references, not candidate passes.

- Writable: `codex-rs/security-policy/src/revocation.rs` and
  `security_policy_tests.rs`. Existing `grant.rs` and `mandate.rs` are read-only
  contract context in this lane.
- Consumers: PF-13-S04/PF-27 transport, PF-38-S03 financial recovery and PF-25 stop UI.

## Preconditions

- [x] Plan active; all dependencies completed and archived.
- [x] Assign a named execution owner and exact plan-matching worktree/branch/base; reserve disjoint scopes and integration gate if parallel.
- [ ] Read root and nearest implementation AGENTS.md; run the sprint checker before readiness.

## Done

- [x] Follow-up separated from the accepted upstream foundation; no new implementation or qualification claimed.

## Remaining

- [ ] Define queued, admitted, uploading and established-channel transitions and the linearization point after which no new protected dispatch or channel write is permitted.
- [ ] Require current run/revocation generation at every protected dispatch; reject stale authority and specify how adapters fence open channels, in-flight uploads and queued work without revoking unaffected siblings.
- [ ] Emergency kill and restriction take effect even when audit storage is unavailable or an earlier financial effect is submitted/unknown. Preserve uncertainty for later reconciliation; never claim completed effects can be undone.
- [ ] Add deterministic interleaving, repeated kill, stale-generation, open-channel, audit-unavailable and unknown-financial-outcome contract tests. Retain the accepted generation/order/idempotency tests; actual transport/restart proof remains with consumers.

## Verification

- [ ] `cd codex-rs && just fix -p codex-security-policy && just fmt` before final affected tests.
- [ ] `cd codex-rs && just test -p codex-security-policy revocation && just test -p codex-security-policy`.
- [ ] TUI applicability: none; PF-25 and PF-26 retain interactive kill/recovery proof.

## Exit evidence

- [ ] Record implementation commit, changed paths, contract version and exact final-tree commands/results under `qa/security-levels/sprints/PF-19-S02/`.
- [ ] Preserve S01 archive/evidence unchanged; do not relabel historical passes as proof of these new cases.
- [ ] Record consumer integration handoff; complete all ledgers before archive and update plan/navigation.
