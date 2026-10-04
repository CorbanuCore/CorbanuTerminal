# Final focused verification command

Working directory: this worktree's `codex-rs`. Log: `focused-final-8.log`.
This is a support run, not independent functional acceptance.

```sh
RUSTUP_TOOLCHAIN=1.95.0 \
CARGO_TARGET_DIR=/Volumes/CorbanuDrive/Corbanu/worktrees/live-permission-transition-20260913/.codex-work/live-permission-transition-target \
just test -p codex-app-server -p codex-app-server-protocol -p codex-tui -p codex-core \
  --locked --offline --test-threads 1 \
  -E 'package(codex-app-server-protocol) or test(settings_confirmation) or test(permission_confirmation) or test(thread_settings_update) or test(note_thread_settings_reports_only_effective_changes) or test(chatwidget::tests::permissions::) or test(apply_permission_profile_selection_preserves_loader_overrides) or test(refreshed_mcp_binding_captures_current_approval_authority) or test(mcp_elicitation_reviewer_uses_latest_runtime_authority) or test(overrides_turn_context_but_keeps_cached_prefix_and_key_constant) or test(enqueue_primary_thread_session_replays_buffered_approval_after_attach) or test(resolved_buffered_approval_does_not_become_actionable_after_drain) or test(interrupt_without_active_turn_is_treated_as_handled)'
```

Preceded by `RUSTUP_TOOLCHAIN=1.95.0 just fmt` from the worktree root. Baseline
formatter churn was reversed as documented in `implementation-checkpoint.md`.
`git diff --check`, `python3 docs/plans/check.py`, and
`python3 docs/sprints/check.py` passed. No direct cargo test, all-features,
whole-workspace test, install, live credentials or external inference request.
Native/RPC tests use disposable homes and local fixtures; selected existing Core
tests use their own local mocks. Their passes do not replace real-key PTY proof.

The candidate patch contains tracked changes plus all new source/schema/snapshot
files; `git apply --reverse --check implementation.patch` succeeded against the
current tree without applying anything. File hashes are in `candidate-manifest.md`.
Source accounting counts additions plus deletions and new leaf lines, splits
the existing native-client test module and four test-registration lines out of
non-test counts, and excludes generated schemas, snapshots and API prose.
