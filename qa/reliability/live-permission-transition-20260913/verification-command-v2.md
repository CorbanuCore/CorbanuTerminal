# PF83 cycle 1 / v2 exact-tree verification

Worktree `/Volumes/CorbanuDrive/Corbanu/worktrees/live-permission-transition-20260913`;
base `005cc644f59b1e762e5497b329e106c67925d4ed`, branch
`fix/live-permission-transition-20260913`. Commands invoked from this worktree
root; `justfile` sets the Rust working directory to `codex-rs`.

Final formatting: `RUSTUP_TOOLCHAIN=1.95.0 just fmt`, exit 0;
`fmt-v2-7.log`. The formatter's unrelated baseline churn was captured in
`fmt-v2-7-baseline-churn.patch`, reverse-checked and reversed only for the same
initially clean paths documented in v1: Core accounting fixture, login fixture,
responses-api-proxy and scripts/initiative_control. No parent edits were undone.
No source/formatting changes are permitted after the final successful run.

```sh
RUSTUP_TOOLCHAIN=1.95.0 \
CARGO_TARGET_DIR=/Volumes/CorbanuDrive/Corbanu/worktrees/live-permission-transition-20260913/.codex-work/live-permission-transition-target \
just test -p codex-app-server -p codex-app-server-protocol -p codex-tui -p codex-core \
  --locked --offline --test-threads 1 \
  -E 'package(codex-app-server-protocol) or test(settings_confirmation) or test(permission_confirmation) or test(thread_settings_update) or test(note_thread_settings_reports_only_effective_changes) or test(chatwidget::tests::permissions::) or test(apply_permission_profile_selection_preserves_loader_overrides) or test(refreshed_mcp_binding_captures_current_approval_authority) or test(mcp_elicitation_reviewer_uses_latest_runtime_authority) or test(overrides_turn_context_but_keeps_cached_prefix_and_key_constant) or test(enqueue_primary_thread_session_replays_buffered_approval_after_attach) or test(resolved_buffered_approval_does_not_become_actionable_after_drain) or test(interrupt_without_active_turn_is_treated_as_handled)'
```

Final log: `focused-v2-final-3.log`. Consult `repair-v2-results.md` for its actual
result and hash. Rust reports `rustc 1.95.0 (59807616e 2026-04-14)`.
No direct cargo test, full workspace/all-features run, install or live inference.

Earlier diagnostic/support commands use the same environment and
`just test -p codex-tui --locked --offline --test-threads 1` with
`-E 'test(permission_confirmation)'` for `tui-v2-attempt-{1,2,3}.log` and
`-E 'test(permission_confirmation_native_requests_carry_effective_defaults)'`
for `tui-v2-diagnostic-1.log`. `focused-v2-final-{1,2}.log` used the full command above.
Each attempt follows its corresponding `fmt-v2-{1,2,3,4,5,6}.log` and selective
baseline-churn restoration. Old logs and v1 patch/manifest remain unchanged.

Freeze/audit after success:
`python3 qa/reliability/live-permission-transition-20260913/freeze-v2.py`.
The script checks literal scope, base, no staging, immutable v1 hashes and size,
generates v2 patch/manifest, and reverse-checks the patch without applying it.
`--check` performs the same read-only size/scope checks without generating files.
Governance: `python3 docs/plans/check.py`, `python3 docs/sprints/check.py`;
whitespace: `git diff --check`. Parent retains sprint/plan bookkeeping.
