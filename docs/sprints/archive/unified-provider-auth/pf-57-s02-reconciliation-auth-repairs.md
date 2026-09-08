---
sprint_id: "PF-57-S02"
title: "Reconciled release credential-lifecycle repairs"
status: completed
plan_file: "docs/plans/active/unified-provider-auth.md"
plan_feature: "PF-57"
execution_order: 17
owner: "Codex primary repair agent"
parallel_lane: "reconciliation-repairs"
write_scope: "codex-rs/wallet-daemon/, codex-rs/tasknode-session/, codex-rs/model-provider-info/, codex-rs/tui/src/provider_status_host.rs, codex-rs/tui/src/provider_status_host_tests.rs, codex-rs/tui/src/chatwidget/wallet_api.rs, codex-rs/tui/src/chatwidget/wallet_api_tests.rs, codex-rs/tui/src/chatwidget/wallet_http.rs, codex-rs/tui/src/chatwidget/wallet_http_tests.rs, codex-rs/tui/src/chatwidget/wallet_menu.rs, codex-rs/tui/src/chatwidget/wallet_usage.rs, codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__wallet_menu__tests__wallet_legacy_daemon_upgrade.snap, codex-rs/tui/tests/suite/multi_provider_onboarding.rs, docs/plans/active/unified-provider-auth.md, docs/sprints/current/unified-provider-auth/, docs/sprints/archive/unified-provider-auth/, docs/features/tasknode.md, docs/features/wallet-plan.md, docs/authentication.md, qa/release/0.1.38/"
integration_gate: "Codex primary agent serially integrates the three review fixes, runs formatting then affected suites and true-TUI regression proof, records site review findings separately, and preserves the reviewed integration ancestry and unrelated worktrees."
worktree: "/home/pfrpc/repos/worktrees/corbanu-reconcile-release-fixes"
branch: "fix/reconcile-release-0.1.37-review"
base_commit: "f03e95f7a65609bb442764d6306682d5fe43f6bb"
depends_on: "PF-57-S01"
created: 2026-09-04
updated: 2026-09-04
---

# PF-57-S02 — Reconciled release credential-lifecycle repairs

## Execution mandate

- Deliver the three Astra findings authorized by the user on 2026-09-04.
- Exclude site implementation, publication, live payments and unrelated security work.

## Plan linkage

- Plan: [Unified provider authentication](../../../plans/active/unified-provider-auth.md).
- Feature: `PF-57`; upstream-touch record and 2026-09-04 amendment in that plan.
- Product citation: **Shipping MVP — LIVE** — “Encrypted `/vault`, masked entry, metadata-only inspection, and operational credential use without placing raw values in chat.”
- Related product contracts: existing wallet scoped signing and linked Task Node identity.

## Code boundaries

- Wallet daemon client/protocol/server: negotiate compatibility before operations; explicit quiescence/restart for legacy servers that cannot safely drain payments.
- Task Node session store: durable unlink suppression, legacy import, explicit relinking, failure propagation and isolation tests.
- Model-provider aliases, shared TUI metadata and read-only account/usage requests: environment presence and precedence agree with runtime resolution.
- Colocated tests, the generated `codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__wallet_menu__tests__wallet_legacy_daemon_upgrade.snap`, and multi-provider TMUX fixtures; docs and release evidence.

## Preconditions

- [x] Active plan amended under explicit user repair authority; PF-57-S01 archived.
- [x] Exact worktree allocated at reviewed branch head; unrelated edits preserved.
- [x] Serial repair scope is disjoint from the two active security implementation lanes.
- [x] Read applicable repository/Rust/core/TUI policy and test-tui skill.

## Done

- [x] Reproduced old-daemon rejection and import/unlink/reload resurrection on reviewed source; baseline 158 focused tests passed.
- [x] Added protocol negotiation before passcodes/operations, home-specific manual recovery and direct legacy-compatible TUI Lock; Astra final production-diff review found no new P1/P2.
- [x] Added durable per-profile unlink state and failure-injection/relink tests; real encrypted-vault reopen probe no longer restores the session.
- [x] Shared Corbanu aliases and credential precedence across transport, metadata and read-only account/usage; 175 domain tests passed.
- [x] Completed read-only site/backend review and recorded two unfixed checkout/recovery findings separately.

## Remaining

No sprint-scoped work remains. Legacy daemon restart is a disclosed one-time user
action; site fixes, production rollout and broader release gates are outside this sprint.

## Verification

- [x] Scoped `just fix` and `just fmt`, then 175 domain tests and 82 focused TUI tests passed; final production diff re-reviewed by Astra.
- [x] Three true-TMUX alias/restart/account-read cases passed, including legacy daemon failure, direct Lock and retry; additional manual `just codex` provider check passed.
- [x] New wrapped recovery-message snapshot reviewed and accepted; real encrypted-vault reopen probe passed.
- [x] Live-repository applicability, source/build identity and platform limits recorded without a broader release claim.
- [x] Plan/sprint checks, portable-skill parity and `git diff --check` passed.

## Exit evidence

- [x] Implementation `41794c3ae7de689594b21837c18899945ee75cf5`; commands, hashes, test IDs and limits in [Astra fixes](../../../../qa/release/0.1.38/astra-fixes.md).
- [x] User documentation and site review match the scoped result; no merge, push or deployment performed.
- [x] Completed sprint archived and plan backlinks updated.
