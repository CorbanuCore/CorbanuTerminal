# Worker brief: merge main into the integration branch (PR #138)

You are an Opus 5.5 High worker. Work autonomously; do not ask questions.

## Context
- Repo CorbanuCore/CorbanuTerminal. `main` is now `91ed4ab8a1`: it just received the CI baseline repair (#146,
  including the new non-blocking `nightly-ci.yml` and `cold_bazel` gating), contributor fixes #131 #132 #133 #142
  #143 #144 #145, the argument-comment annotations (#148) and the secret-broker fixture crate (#149). That set was
  proven green together on trial PR #147.
- `integrate/management-workstreams-20260911` (PR #138 → main) is at `4c748fe5e4` and is shared: other workers push
  to it, fast-forward only.
- Prepared worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/merge-main-into-138` (detached at `4c748fe5e4`). A
  trial `git merge 91ed4ab8a1` conflicts in ~80 files: `codex-rs/Cargo.lock`, `codex-api/src/telemetry.rs`,
  `core/src/exec_env.rs`, multi-agent tests, `protocol/src/security.rs`, many `secret-broker-fixture/src/launch/...`
  files (main moved the fixture into its own crate; integration kept editing it in place), `secret-broker-service`
  Cargo.toml, `tui/src/app/tests.rs`, `tui/src/provider_status_host.rs`, ~60 TUI insta snapshots, and
  `docs/install.md`, `docs/getting-started.md`.

## Do
1. `git merge 91ed4ab8a1` in that worktree and resolve every conflict on its merits: keep both sides' intent.
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
5. Update #138's description with a short "merged main 91ed4ab8a1" note and the check status.

## Never
Never print secrets. `rm -f` is blocked by policy. No force-push. Don't merge #138 itself. Don't work in
`worktrees/management-workstreams-20260911` (it has another session's uncommitted files). Don't change CI timeouts.

## Final report (your last message)
How each conflict group was resolved, test results, the merge commit and pushes, #138 check status, and anything
left.
