# Worker brief: make rust-ci-full green again on PR #155 after the integration merge

You are an Opus 5.5 High worker. Work autonomously; do not ask questions.

## Context
- PR https://github.com/CorbanuCore/CorbanuTerminal/pull/155 (branch `codex/fix-rust-ci-full-20261004`, worktree
  `/Volumes/CorbanuDrive/Corbanu/worktrees/fix-rust-ci-full-20261004`; mirror branch `codex/full-ci-20261004` triggers
  rust-ci-full). It was green before main received the big integration merge (#138, merge commit `3ecc3c4066`:
  local cost ledger/accounting, management workstreams, secret-broker work).
- I merged main into the branch (`5e04f0372c`, taking main's versions of 7 conflicted dead-code files and
  `core/src/client_tests.rs`) and added `2540c9ed85` (allow `clippy::duplicate_mod` for codex-state accounting test
  fixtures). Lint/Build should now pass.
- The new rust-ci-full run on `2540c9ed85` (mirror branch; `gh run list -R CorbanuCore/CorbanuTerminal -b
  codex/full-ci-20261004`) and the PR's own checks show new test failures from the integration code, e.g. Tests
  shard 1/4: `Error: error returned from database: (code: 1) no such table: draft_accounting_attempts` and
  app-server `suite::v2::thread_read::paginated_history_lists_use_projected_turns_and_items`,
  `paginated_thread_name_set_is_reflected_in_read_list_and_metadata_resume`,
  `thread_search_occurrences_reads_paginated_projection` panicking at `app-server/tests/common/test_app_server.rs:1628`.
  Main's own postmerge-ci on `3ecc3c4066` also failed — same backlog.
- Previous context: `.codex-work/workers-20261002/postmerge-full.result.md`, `fix-155.result.md` (same directory).

## Do
1. Wait for the current rust-ci-full run to finish, then collect every failing test from all shards (and the PR's
   other checks). Classify and fix the real causes (e.g. a test fixture/DB that skips the accounting migrations, a
   migration ordering issue, a projection/pagination regression). Never weaken test intent.
2. Verify on Linux on the RTX box (`ssh ambient@100.99.88.49`, fresh clone under `~/fix-155b-20261004/`, TMPDIR on
   ext4, clean up afterwards) with the workflow's own commands.
3. Push to the PR branch and the mirror (normal pushes), iterate until rust-ci-full and the PR checks are green.
   Cap: 3 CI rounds. Comment a short summary on the PR. Don't merge.

## Never
Never print secrets. `rm -f` is blocked by policy. No force-push. Don't change CI timeouts. Don't touch
`worktrees/management-workstreams-20260911`.

## Final report (your last message)
Failures → cause → fix (commit) → verification; final rust-ci-full and PR check status.
