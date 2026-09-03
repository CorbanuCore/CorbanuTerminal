---
sprint_id: "PF-57-S01"
title: "Latest-main integration and credential-store liveness"
status: in_progress
plan_file: "docs/plans/active/unified-provider-auth.md"
plan_feature: "PF-57"
execution_order: 16
owner: "Codex primary integration agent"
parallel_lane: "provider-auth-main-integration"
write_scope: "codex-rs/keyring-store/src/lib.rs; codex-rs/secrets/src/local.rs; codex-rs/provider-auth/src/runtime_selection.rs; codex-rs/provider-auth/src/runtime_selection_tests.rs; codex-rs/tui/src/app/event_dispatch.rs; codex-rs/tui/src/onboarding/auth.rs; codex-rs/tui/tests/; docs/plans/active/p0-security-levels.md; docs/sprints/current/p0-security-levels/pf-30-s01-typed-source-envelope.md; docs/sprints/index.md; docs/sprints/current/unified-provider-auth/index.md; docs/plans/active/unified-provider-auth.md; docs/sprints/current/unified-provider-auth/pf-57-s01-latest-main-integration.md; mkdocs.yml; qa/provider-auth/pf-57/"
integration_gate: "Codex primary integration agent merges origin/main into the preserved PF-48–PF-56 lineage, resolves only the declared conflicts/findings, bounds the reproduced OS-keyring hang without weakening encrypted-vault fallback semantics, reruns post-fork main regression tests plus provider-auth and true-TMUX suites on the combined tree, obtains one Fable 5.1 review through Corbanu/TMUX, and records the exact candidate before any main update."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/integrate-unified-provider-auth-final"
branch: "integration/unified-provider-auth-final"
base_commit: "06211dbfca61d3f36df3bf069a79ed53ad7a6fa2"
depends_on: "PF-56-S01"
created: 2026-09-02
updated: 2026-09-02
---

# PF-57-S01 — Latest-main integration and credential-store liveness

## Execution mandate

- Deliver: one merge-based candidate combining PF-48–PF-56 with latest `origin/main` while preserving archived commit identities and post-fork fixes, including a bounded shared-keyring repair for the reproduced provider-vault liveness failure.
- Excludes: rewriting archived sprint history, new provider product behavior, release or human-acceptance claims, credential-format migration, and weakening keyring or vault custody.

## Plan linkage

- Plan: [Unified provider onboarding and management](../../../plans/active/unified-provider-auth.md).
- Feature: `PF-57`.
- Acceptance advanced: provider setup, current-provider selection, and credential recovery remain coherent after integration with latest main.

## Code boundaries

- Existing: shared runtime selection, onboarding/auth event adapters, recent main provider regressions, sprint navigation, and TUI harness suites.
- Planned: semantic conflict resolution, reproduced fail-closed eligibility fixes, a bounded OS-keyring operation with safe fallback and stuck-worker control, combined-tree evidence, and integration handoff.
- Tests: provider-auth, login, affected TUI/provider regressions, governance, formatting, true-TMUX, and secret-canary checks.

## Preconditions

- [x] Plan is active.
- [x] PF-56-S01 and every predecessor are completed and archived.
- [x] Worktree, branch, base commit, owner, lane, scope, and receiving gate are exact.
- [x] Latest `origin/main` is `81dcbef5dbd500326a14acf8584263d4d950009b`; feature tip is `06211dbfca61d3f36df3bf069a79ed53ad7a6fa2`.

## Done

- [x] PF-57 record created and linked to one plan feature.
- [x] Merge-based strategy selected so archived implementation SHAs remain valid.
- [x] Branch relationship and five textual conflicts audited before allocation.
- [x] Latest-main merge and semantic conflict resolution committed at `14959ccc0` after focused provider-auth, onboarding, model, and TUI regression tests.
- [x] Combined login/model test run reproduced two 60-second timeouts and repeated 8–40 second OS-keyring waits, establishing the provider-vault follow-up as a PF-57 integration regression.
- [x] Validated allocation committed before implementation.
- [x] Latest `origin/main` merged; the two Rust and three governance conflicts were resolved semantically.
- [x] Two independent fail-closed eligibility findings reproduced, repaired, and covered by focused regression tests.

## Remaining

- [ ] Bound OS-keyring calls, prevent repeated stuck-worker accumulation, and verify existing encrypted-vault fallback behavior for read, write, delete, and startup/logout callers.
- [ ] Run formatting, governance, affected automated tests, combined true-TMUX flows, and secret-canary checks.
- [ ] Run one Fable 5.1 independent review through Corbanu/TMUX and remediate applicable findings.
- [ ] Record the candidate, test artifacts, exact integration commit, and truthful remaining release gates.

## Verification

- [ ] Focused test: `CARGO_TARGET_DIR=/Volumes/CorbanuDrive/Corbanu/.codex-work/targets/pf57 CARGO_INCREMENTAL=0 just test -p codex-provider-auth -j 1 --retries 0`.
- [ ] Integration tests: affected `codex-login` and `codex-tui` suites after `just fmt`.
- [ ] TUI: final combined candidate launched through the repository harness with text and Enter sent separately; startup, provider setup/replacement, restart, failure, and recovery checkpoints recorded.

## Exit evidence

- [ ] Implementation and merge commits recorded.
- [ ] Final-tree outputs and TMUX/review artifacts linked under `qa/provider-auth/pf-57/`.
- [ ] Scope audit confirms post-fork main fixes and PF-48–PF-56 behavior are both preserved.
- [ ] `Done` and `Remaining` ledgers reflect reality.
- [ ] Completed record moved to `docs/sprints/archive/unified-provider-auth/`.
