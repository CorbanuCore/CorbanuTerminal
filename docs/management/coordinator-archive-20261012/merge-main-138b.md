# Worker brief: merge main into the integration branch again (PR #138, round 2)

You are an Opus 5.5 High worker. Work autonomously; do not ask questions.

## Context
- Repo CorbanuCore/CorbanuTerminal. `main` now also has #151 #152 #153 #154 #156 (CI permissions, nightly cold-Bazel lanes,
  tmux deflake incl. a composer paste-burst race fix, Bazel clippy dead-code cleanup, two test timing budgets).
  Main's blocking CI and tmux smoke are green.
- `integrate/management-workstreams-20260911` (PR #138 → main) is at `6d4fe69dd2` and is shared: other workers push
  to it, fast-forward only.
- Prepared worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/merge-main-into-138` (detached at the integration tip).
  A trial `git merge` of current main conflicts in: `codex-rs/core/src/client.rs`,
  `tui/src/app/tests/dispatch_integration.rs`, `tui/src/app_event.rs`, `tui/src/chatwidget/claude_code_login.rs`,
  `tui/src/chatwidget/provider_credentials.rs`, `tui/src/chatwidget/wallet_menu.rs`,
  `tui/src/chatwidget/wallet_receipt.rs`, `tui/src/tui/terminal_ownership_tests.rs`, `tui/src/update_action.rs`,
  `tui/src/updates.rs`. Most come from #156 (`10c3c86fba` marks code orphaned on main `allow(dead_code)`;
  `7e90bf2dc5` deletes dead test-only code). Where the integration branch still uses code main marked dead or
  deleted, keep the integration usage and drop the now-unneeded `allow(dead_code)`; where it's dead on both sides,
  take main's version. The previous round's report (merge-main-138.result.md, same directory) explains the
  conventions used last time (snapshot `<V>` version handling, secret-broker fixture crate layout, etc.).

## Do
1. `git merge` current `origin/main` in that worktree and resolve every conflict on its merits: keep both sides' intent.
   For the secret-broker fixture, carry integration's edits into main's new crate layout. Regenerate
   `Cargo.lock` with cargo (don't hand-merge it). For snapshots, resolve the code first, then regenerate with
   `cargo insta` and review each snapshot change: accept only changes explained by the merged code.
2. Build and test: `cargo build -p codex-cli`, `cargo test` for codex-tui, codex-core (accounting, multi-agent,
   exec_env), codex-protocol, codex-api, the secret-broker crates, codex-state; `just fmt`; clippy on touched
   crates; `python3 -m unittest discover -s .github/scripts -p 'test_*.py'` (4 setup-telegram failures are a known
   macOS bash 3.2 issue). Use the RTX box (`ssh ambient@100.99.88.49`, a fresh clone under `~/merge-138-20261003/`,
   cleaned up afterwards) for the Linux test runs if that's faster.
3. Before pushing, fetch the integration branch again. If it moved, merge its new tip in too (never rebase or
   force), then push fast-forward to `integrate/management-workstreams-20260911`.
4. Watch PR #138's checks (`gh pr checks 138 -R CorbanuCore/CorbanuTerminal`, poll every ~10 min) and fix real
   failures (push follow-up commits, fast-forward). Retry likely flakes once: known flaky tmux smoke tests are
   `tmux_fresh_a_then_b_preserves_first_success_across_restart` and
   `tmux_resumed_main_session_retains_exact_runtime_identity`. Cap: 3 CI rounds.
5. Update #138's description with a short "merged main <sha>" note and the check status.

## Never
Never print secrets. `rm -f` is blocked by policy. No force-push. Don't merge #138 itself. Don't work in
`worktrees/management-workstreams-20260911` (it has another session's uncommitted files). Don't change CI timeouts.

## Final report (your last message)
How each conflict group was resolved, test results, the merge commit and pushes, #138 check status, and anything
left.
