# RETURN — tui-snapshot-rot-32

Allocation digest: 171e8bcb10e0d4d4306fc5a67b5fd4970748dc3d76d1c088277975157b1ad40e.
Claim: 0878ece7-e746-4543-9a73-f2e1c88935ec.
Runtime: gpt-6-astra, high.
Base: dc7f8da90d2b9ff34d8c43cd12afb15239619d7c.
Worktree: /Volumes/CorbanuDrive/Corbanu/worktrees/tui-snapshot-rot-20260916.

The brief was read first. `shasum -a 256` matched
9eaf22ef410b0912a2020585d2716634f2304c2a5e1d4d5d88488ee374882f0a exactly.
Its older allocation labels (outer 28, inner 27) are preserved; this receipt
belongs to action 32.

## Changes and inherited work verified

The supplied base already contains the update-notice normalization, three
normalized history goldens, backend normalization routing, short-marker padding
fix and all eleven individually justified snapshot acceptances. This continuation
retains those changes after checking the current rendering code and originating
commits. It adds one ten-line test,
`status_update_version_normalization_preserves_frame_across_releases`, to
verify identical fixed-width notice output across patch-version digit counts and
a prerelease version.

The helper masks only the actual compiled version, uses `<V>`, preserves target
versions, and does not mask malformed, missing or wrong current versions.
The shorter marker handles the flush-frame case without saturation or widening;
existing zero-through-three-padding cases pass. The Unix/macOS backend early
return passes through path/version normalization, and its regression passes.

The three history goldens are
`pnpm_update_available_history_cell_snapshot`,
`standalone_unix_update_available_history_cell_snapshot`, and
`standalone_windows_update_available_history_cell_snapshot`.
The pnpm file is orphaned, so only the two standalone goldens are exercised.

[Per-snapshot verdicts](tui-snapshot-rot-32-verdicts.md) give all eleven names,
individual reasons, originating commits and remaining limitations. All eleven
are retained as stale-golden corrections; none lacks information needed for
that decision. All eleven corresponding tests pass in the status gate.

## Fresh-target verification

Dedicated target, created empty for this action and never shared with another
worktree:

`/Volumes/CorbanuDrive/Corbanu/worktrees/tui-snapshot-rot-20260916/qa/initiative-control/pf-80-s01/tui-snapshot-rot-32-target`

All test commands ran from `codex-rs` with this `CARGO_TARGET_DIR` and
`INSTA_UPDATE=no`. The guarded `just test` wrapper supplied a disposable
profile, stripped live aliases/credentials and enabled native-keyring denial.
No native credential prompt was observed. No raw cargo test/nextest command,
bulk snapshot acceptance, workspace formatter or fix command was run.

| Run | Exact command | Executed | Passed | Failed | Skipped | Exit |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| Initial permission | `just test -p codex-tui permission` | 91 | 90 | 1 | 4078 | 100 |
| Final permission | `just test -p codex-tui permission` | 91 | 91 | 0 | 4078 | 0 |
| Final status | `just test -p codex-tui status` | 390 | 390 | 0 | 3779 | 0 |
| Final usage | `just test -p codex-tui usage` | 91 | 91 | 0 | 4078 | 0 |
| Supplemental notices | `just test -p codex-tui -E 'test(update_available_history_cell_snapshot) \| test(update_prompt_snapshot)'` | 2 | 2 | 0 | 4167 | 0 |

The sole initial failure was exactly
`suite::provider_convergence::tmux_permission_reload_preserves_active_runtime_over_saved_default`
in `codex-tui::all`: the empty target lacked the `codex` executable.
A debug `cargo build -p codex-cli` in the same target, with both dev/test
debug-assertions enabled, succeeded. The fresh permission rerun then passed,
including that real-key TUI fixture. There are no failure names in the final
three gates.

Logs are preserved as `tui-snapshot-rot-32-{permission,permission-final,status,usage,update-notices,cli-build}.log`.
The two passing TUI fixtures' raw artifacts are in
`tui-snapshot-rot-32-permission-artifacts/` and
`tui-snapshot-rot-32-status-artifacts/`.
Those fixtures use a hardcoded `codex-rs/tui/target/tmux-artifacts` output;
their newly generated directories were moved into the allowed QA scope.
Insta removed twelve tracked historical `.snap.new` files when their snapshots
passed; those files were restored byte-for-byte from HEAD to preserve prior
attempt evidence. No accepted golden was changed by these test runs.

Only `rustfmt --edition 2024 tui/src/status/snapshot_helpers_tests.rs` was run.
It preceded final test compilation; it reported only the repository's existing
stable-toolchain warning about nightly-only imports_granularity.
`git diff --check` passes. Final changes are confined to the writable scope.

## Non-test code and limits

Production/non-test Rust lines changed relative to the supplied base: **0**.
The helper is behind the existing `#[cfg(test)]` module declaration.
New non-test files are QA logs, receipts, captured test artifacts and an ignored
build cache; they document this task rather than change product behavior.
No push was made.

The brief is incomplete about remaining version-bearing goldens:
`codex_tui__update_prompt__tests__update_prompt_modal.snap` still embeds
`0.1.1`. Its assertion in `update_prompt.rs:271` bypasses normalization and
its constructor uses the package version. That file is outside writable scope.
The entire module is disabled with debug assertions, so the guarded lane did
not execute this test. It needs an authorized assertion edit and the separately
isolated release/native validation lane, plus individual review of its stale
URL/installer differences. This allocation does not claim the broader absence
of release-sensitive goldens.

The pnpm snapshot has no current test and cannot fail on a version bump as the
brief says. Its inherited normalized content is preserved, but no execution
pass is fabricated. Explicit `<VERSION>` markers in two app goldens are stable
injected test inputs and should remain unchanged.
