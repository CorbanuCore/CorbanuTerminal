I fixed main's CI in one PR, #614, which merged after all 33 checks passed (merge commit 94b82f147a). Of the three red items, only the Windows clippy failure needed a code change. The vault-typing test failure is a flaky test, not damage from #592, and I filed issue #615 for it rather than fixing it.

**What shipped (item 1, Windows clippy):** test helpers that only unix tests use are now compiled only on unix, using `#[cfg(unix)]` or `#[cfg(all(test, unix))]` rather than a blanket `allow(dead_code)`. Besides the two in the brief, three more had the same problem and were hidden because cargo stops at the first crate that fails:
- `without_protected_paths` in `core/src/security/protected_additional.rs`
- `isolation_problems` in `tui/src/security/aggressive_tests.rs`
- `test_git` in `core-plugins/src/startup_sync_tests.rs`
- `deny_in_workspace` in `protocol/src/permissions.rs`
- the test module in `core/src/tools/writable_root_notices.rs`, added by #591 after the last postmerge run

**Item 2 (`a_session_account_that_cannot_load_changes_nothing`):** no change needed. The failure happened before #585, and #585 already fixed it by switching the test from `openai` to `ollama`. It passes in the postmerge runs for a6f30dad0f and d878532cf6.

**Item 3 (`pf_23_s01_typing_a_protected_command_into_a_running_shell_needs_the_human`):** this is an intermittent test failure, not something #592 broke, so the vault-typing protection is intact.
- On glitch the test passed 15 of 15 times alone at d878532cf6, and its test group passed 30 of 30 times at 398a3332e6. The failed jobs on #591 and #600 both passed on rerun.
- I did reproduce the failure: with an empty HOME, a sibling test (`pf_23_s02_typing_into_a_shell_started_before_untrusted_content_is_confined`) failed 2 times in 25 with `write_stdin failed: Unknown process id 1000`. That means the shell was already gone when the test tried to type into it, so the vault check never ran.
- I didn't find the cause. #615 says what to capture next to decide whether it's a product race or a test timing problem. Until then, rerunning the job is the workaround.

**Gate:**
- **Windows:** a Windows-target clippy on the Mac (via `cargo xwin`, whole workspace) was clean, and the dispatched full CI's Windows clippy job passed.
- **macOS:** clippy was clean.
- **Linux:** clippy with `-D warnings` on glitch was clean at the PR head (101dc3e909) for the four changed crates.
- **Review:** the Opus 5.5 High review returned APPROVE (one optional nit about a code comment).

**Cleanup:** I commented on #591 and #600 that main is fixed. My Mac worktree and build targets and my glitch worktree, target and logs are removed, and nothing is left uncommitted. The report is at `.codex-work/workers-20261002/main-ci2.result.md`.

**Open:** only #615.