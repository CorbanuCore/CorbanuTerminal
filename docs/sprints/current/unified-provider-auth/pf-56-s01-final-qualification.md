---
sprint_id: "PF-56-S01"
title: "Unified provider final qualification"
status: in_progress
plan_file: "docs/plans/active/unified-provider-auth.md"
plan_feature: "PF-56"
execution_order: 15
owner: "GPT-5.6 Sol high implementation agent"
parallel_lane: "provider-auth-serial"
write_scope: "codex-rs/tui/tests/suite/multi_provider_onboarding.rs; codex-rs/tui/tests/suite/provider_management.rs; codex-rs/tui/tests/suite/provider_convergence.rs; codex-rs/tui/tests/suite/mod.rs; docs/authentication.md; docs/features/model-providers.md; docs/features/claude-plan-authentication.md; docs/corbanu-product-spec.md; docs/plans/active/unified-provider-auth.md; docs/sprints/current/unified-provider-auth/pf-56-s01-final-qualification.md; qa/provider-auth/pf-56/"
integration_gate: "PF-56 only: qualify the complete PF-48–PF-55 tree, run the bounded final primary review plus one Claude Fable 5 high external review controlled through TMUX, rerun the integrated automated and true-TMUX matrices, exercise disposable TensorCash and Isometric Game worktrees, update finished provider/auth documentation, and record release blockers truthfully. Production fixes or additional paths require a documented literal scope expansion before editing. No fabricated live-account, named-human, platform, benchmark, merge, tag, or release evidence."
worktree: "/home/pfrpc/repos/worktrees/corbanu-main-f7356a94e0"
branch: "feat/unified-provider-auth"
base_commit: "f7356a94e032234022a462d65b576a7de2854859"
depends_on: "PF-55-S01"
created: 2026-09-01
updated: 2026-09-02
---

# PF-56-S01 — Unified provider final qualification

## Execution mandate

- Deliver: formatted integrated candidate with bounded reviews, extensive TMUX, docs, and release evidence.
- Excludes: waiving live-account, human, platform, benchmark, or release gates.

## Plan linkage

- Plan: [Unified provider onboarding and management](../../../plans/active/unified-provider-auth.md).
- Feature: `PF-56`.
- Acceptance advanced: the complete P1 feature is fit for purpose on the final tree.

## Code boundaries

- Existing: all PF-48–PF-55 changed paths, TUI TMUX harness, provider docs, QA/release records.
- Planned: final regression matrix, artifacts, review dispositions, finished docs, and release linkage.
- Tests: complete automated, adversarial, PTY, live-repository, restart/resume, and platform matrix.

## Preconditions

- [x] Plan is active.
- [x] PF-55-S01 and every predecessor are completed and archived; PF-55 implementation is `21cf3199f2` and archive is `5a1fd449e7`.
- [x] Exact serial allocation matches the plan and retains the GPT-5.6 Sol high implementation owner.
- [x] Candidate `21cf3199f2`, disposable TensorCash worktree `/home/pfrpc/repos/worktrees/pf56-tensorcash-dd6e9202` at `dd6e92024254090de0f596b090bd5c74c4d97b90`, disposable Isometric Game worktree `/home/pfrpc/repos/worktrees/pf56-isometricgame-59821b7a` at `59821b7a85524f186f946c4670480c7ee96483cb`, pending live-account/platform/release gates, and Claude Fable 5 high reviewer runtime are recorded.

## Done

- [x] Draft sprint record created and linked to PF-56.
- [x] Allocated serially at post-PF-55 archive commit `5a1fd449e7` with literal test, documentation, QA-ledger, and review scope.

## Remaining

- [ ] Run fixes/formatting before final automated and TMUX evidence.
- [ ] Rerun all affected crate, integration, snapshot, migration, schema, redaction, and canary tests.
- [ ] Execute substantial true-TMUX matrix for multi-provider, OpenAI, Claude, API keys, custom providers, deferred Corbanu, eligibility, current replacement, failure/cancel, restart/resume.
- [ ] Run real workflows in disposable TensorCash and Isometric Game worktrees.
- [ ] Complete at most four formal review passes unless a documented major issue justifies more.
- [ ] Spawn and control one Claude Fable 5 high external reviewer through TMUX and disposition findings.
- [ ] Update finished provider/auth docs only after candidate behavior is verified.
- [ ] Record named human, physical platform, upstream, release, and benchmark evidence or leave shipment blocked.

## Verification

- [ ] Focused test: every final command and result is recorded in the QA ledger.
- [ ] Integration test: combined-tree build/request/restart and live-repository suites pass.
- [ ] TUI: final formatted candidate passes all required PTY flows with real keys and artifacts.

## Exit evidence

- [ ] Candidate commit/version/binary hash and implementation commits recorded.
- [ ] Final-tree outputs, TMUX bundles, live-repository bases, and canary scans linked.
- [ ] Four-review budget and every finding disposition recorded.
- [ ] Named human/release evidence linked or remaining blockers stated without a pass claim.
- [ ] `Done` and `Remaining` reflect reality.
- [ ] Completed record archived.
