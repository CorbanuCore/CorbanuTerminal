---
sprint_id: "PF-57-S01"
title: "Latest-main integration and credential-store liveness"
status: completed
plan_file: "docs/plans/active/unified-provider-auth.md"
plan_feature: "PF-57"
execution_order: 16
owner: "Codex primary integration agent"
parallel_lane: "provider-auth-main-integration"
write_scope: "codex-rs/Cargo.toml; codex-rs/keyring-store/Cargo.toml; codex-rs/keyring-store/src/lib.rs; codex-rs/secrets/src/lib.rs; codex-rs/secrets/src/local.rs; codex-rs/vault/src/lib.rs; codex-rs/vault/src/tests.rs; codex-rs/vault/src/claude_auth.rs; codex-rs/login/src/auth/provider_key_vault.rs; codex-rs/provider-auth/src/status.rs; codex-rs/provider-auth/src/status_tests.rs; codex-rs/provider-auth/src/runtime_selection.rs; codex-rs/provider-auth/src/runtime_selection_tests.rs; codex-rs/tui/src/provider_status_host.rs; codex-rs/tui/src/provider_status_host_tests.rs; codex-rs/tui/src/startup_provider.rs; codex-rs/tui/src/app_event.rs; codex-rs/tui/src/app/event_dispatch.rs; codex-rs/tui/src/app/tests.rs; codex-rs/tui/src/chatwidget/wallet_menu.rs; codex-rs/tui/src/onboarding/auth.rs; codex-rs/tui/src/onboarding/provider_setup.rs; codex-rs/tui/src/onboarding/provider_setup_tests.rs; codex-rs/tui/src/onboarding/snapshots/; codex-rs/tui/tests/; docs/features/model-providers.md; docs/plans/active/p0-security-levels.md; docs/sprints/current/p0-security-levels/pf-30-s01-typed-source-envelope.md; docs/sprints/index.md; docs/sprints/current/unified-provider-auth/index.md; docs/plans/active/unified-provider-auth.md; docs/sprints/current/unified-provider-auth/pf-57-s01-latest-main-integration.md; mkdocs.yml; qa/provider-auth/pf-57/"
integration_gate: "Codex primary integration agent merges origin/main into the preserved PF-48–PF-56 lineage, resolves only the declared conflicts/findings, bounds locked OS-keyring access and coalesces logical vault mutations without weakening encrypted-vault fallback or scrypt strength, reruns post-fork main regression tests plus provider-auth and true-TMUX suites on the combined tree, obtains one Fable 5.1 review through Corbanu/TMUX, and records the exact candidate before any main update."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/integrate-unified-provider-auth-final"
branch: "integration/unified-provider-auth-final"
base_commit: "06211dbfca61d3f36df3bf069a79ed53ad7a6fa2"
depends_on: "PF-56-S01"
created: 2026-09-02
updated: 2026-09-03
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
- Planned: semantic conflict resolution, reproduced fail-closed eligibility fixes, a bounded OS-keyring operation with safe fallback and stuck-worker control, single-write logical vault mutations, combined-tree evidence, and integration handoff.
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
- [x] Combined login/model test run reproduced two 60-second timeouts; a live process sample established repeated age/scrypt encryption during one provider-vault mutation as the CPU-bound cause rather than OS-keyring I/O.
- [x] Validated allocation committed before implementation.
- [x] Latest `origin/main` merged; the two Rust and three governance conflicts were resolved semantically.
- [x] Two independent fail-closed eligibility findings reproduced, repaired, and covered by focused regression tests.
- [x] Expanded remediation scope after the required Fable 5.1 Max review reported local/no-auth provider blocking, eager startup command execution, keyring deadline/circuit semantics, dead legacy provider UI, and incomplete Claude-token batching. Production work is limited to reproduced provider usability, lazy command validation, bounded-keyring recovery, and atomic managed-Claude mutations; dead-code cleanup is disposition-only unless it blocks a required gate.
- [x] Expanded the second review cycle only for the reproduced fresh-install regression: implicit local/status-only catalog entries must not suppress onboarding or satisfy its completion gate unless they are the explicit current provider; shipped command-auth documentation must describe lazy request-time validation.
- [x] Kept the recovery contract convergent after structured review: a usable non-interactive replacement is exposed as an explicit onboarding choice, and **Done** remains blocked until a current provider is preserved or a replacement is selected and queued for persistence.
- [x] Bounded OS-keyring reads, writes, and deletes behind one shared operation gate with per-operation deadlines, stuck-worker suppression, and late-success recovery at `458ac28b0`.
- [x] Coalesced vault add, update, delete, bulk-delete, and managed Claude-token mutations into one encrypted-file rewrite per logical operation without changing the vault format or pinned scrypt work factor.
- [x] Optimized scrypt/Salsa only in development and test profiles while preserving production cryptographic parameters and on-disk compatibility.
- [x] Verified fail-closed existing-vault behavior and usable new-profile fallback across the final provider/login test matrix.
- [x] Preserved command-auth laziness, local/status-only provider usability, and explicit established-profile recovery without silent provider substitution at `004556c52`, `247d5bbb`, and `a935e507b`.
- [x] Ran formatting, affected automated tests, combined true-TMUX flows, and final build on the combined tree.
- [x] Ran Claude Fable 5.1 Plan max through Corbanu/TMUX, remediated all three actionable findings, and obtained `FINAL_REREVIEW: CLEAN` on `a935e507b`.
- [x] Recorded the exact candidate, lineage, test counts, binary digest, review, and truthful remaining release gates in [PF-57 qualification](../../../../qa/provider-auth/pf-57/qualification.md).

## Remaining

No sprint-scoped implementation or qualification work remains. Named human,
live-account, physical-platform, release/tag, upstream-disposition, and benchmark
gates remain at plan/release level and are not claimed by this archive.

## Verification

- [x] Focused provider/core matrix: 390 tests passed with external target storage.
- [x] Integration tests: final provider-auth, login, keyring, secrets, vault, model-provider, and affected TUI suites passed after formatting.
- [x] TUI: final `a935e507b` candidate launched through the repository harness with text and Enter sent separately; the remote final run passed all 12 PF-55 convergence journeys plus two PF-53 onboarding journeys in 147.84 seconds.

## Exit evidence

- [x] Implementation and merge commits recorded through final code candidate `a935e507b0173f4ee9c1f0aa539eea6e24ed200f`.
- [x] Final-tree outputs and TMUX/review artifacts linked under `qa/provider-auth/pf-57/`.
- [x] Scope audit confirms latest-main baseline `81dcbef5d` and PF-48–PF-56 tip `06211dbfc` are both ancestors of the candidate.
- [x] `Done` and `Remaining` ledgers reflect reality.
- [x] Completed record moved to `docs/sprints/archive/unified-provider-auth/`.
