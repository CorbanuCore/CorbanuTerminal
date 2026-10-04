# RETURN — tui-snapshot-rot-21

Allocation digest: fbd2641d7165ebc7a5997340c020630bf0cbf8c571aabd5f5aa9960de015cbf2. Claim: 2c051841-ae63-4fac-bee8-b100b469d207.
Runtime: gpt-6-astra, high.
Base: e45b9d77d2bbd1eed35913753c7c73ad3bf970d8.
Worktree: /Volumes/CorbanuDrive/Corbanu/worktrees/tui-snapshot-rot-20260916.
Brief SHA-256 verified: b0828b103346474b923bc3227a4b9e816a87b889d5a48583383ad0f8031c328a.

Routine test-harness/test-data maintenance. Product citation, active plan, sprint,
interactive TUI proof, live-repository qualification, benchmark and human acceptance
are not applicable to this repair: no runtime user behavior changes or release are
included. Code-blind functional qualification is N/A for the same reason; this is
not a user-workflow qualification or a human-test readiness claim. No approvals
are claimed.

## Normalizer and limits

`status::snapshot_helpers::normalize_snapshot_version` exists only under
`#[cfg(test)]`. It matches the exact header text containing
`env!("CARGO_PKG_VERSION")`, substitutes `<VERSION>`, and adjusts the immediately
following spaces by the version-length difference. It preserves the right-frame
column, including when a release adds/removes a digit. Regression tests exercise
0.1.9, 0.1.42, 0.1.100 and 1.0.0-rc.1, repeated headers, and unrelated matching
version text outside the header.

Empty, malformed, and different well-formed versions do not match and remain
literal; comparison with a placeholder golden fails. Missing version text or
a missing header line is not synthesized and likewise fails the golden.
This deliberately does not check the correctness of the package manifest's version
or encode a particular release number in goldens. A wrong manifest version echoed
correctly by the renderer would pass. Real wrapping/geometry changes remain
observable; this does not erase borders, unrelated padding or other header text.

Application is centralized in status `sanitize_directory` (including transcript
buffer rendering), history `render_lines` plus the two direct buffer assertions,
the app `assert_app_snapshot!` macro, and chatwidget `normalize_snapshot_paths`.
Every release-version-bearing golden found under tui/src is covered below.

## Every changed snapshot and header test mapping

For every version row below the only content change is the stated version replacement
and three fewer adjacent spaces. The placeholder is three characters longer than
each old version, so all frame columns and non-header content remain unchanged.
These are intentional per-file edits, validated against actual rendered test output;
no `cargo insta accept`, bulk acceptance, or `INSTA_UPDATE=always` was used.

| Snapshot file | Test (module implied by path) | Assertion route | Exact diff and reason |
| --- | --- | --- | --- |
| `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_cached_limits_hide_credits_without_flag.snap` | `status_snapshot_cached_limits_hide_credits_without_flag` | sanitize_directory → normalizer | `v0.1.38` → `v<VERSION>`; remove 3 following spaces. |
| `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_includes_credits_and_limits.snap` | `status_snapshot_includes_credits_and_limits` | sanitize_directory → normalizer | `v0.1.38` → `v<VERSION>`; remove 3 following spaces. |
| `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_includes_enterprise_monthly_credit_limit.snap` | `status_snapshot_includes_enterprise_monthly_credit_limit` | sanitize_directory → normalizer | `v0.1.38` → `v<VERSION>`; remove 3 following spaces. |
| `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_includes_forked_from.snap` | `status_snapshot_includes_forked_from` | sanitize_directory → normalizer | `v0.1.38` → `v<VERSION>`; remove 3 following spaces. |
| `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_includes_monthly_limit.snap` | `status_snapshot_includes_monthly_limit` | sanitize_directory → normalizer | `v0.1.38` → `v<VERSION>`; remove 3 following spaces. |
| `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_includes_reasoning_details.snap` | `status_snapshot_includes_reasoning_details` | sanitize_directory → normalizer | `v0.1.38` → `v<VERSION>`; remove 3 following spaces. |
| `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_shows_active_user_defined_profile.snap` | `status_snapshot_shows_active_user_defined_profile` | sanitize_directory → normalizer | `v0.1.38` → `v<VERSION>`; remove 3 following spaces. |
| `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_shows_auto_review_permissions.snap` | `status_snapshot_shows_auto_review_permissions` | sanitize_directory → normalizer | `v0.1.38` → `v<VERSION>`; remove 3 following spaces. |
| `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_shows_chatgpt_plan_without_email.snap` | `status_snapshot_shows_chatgpt_plan_without_email` | sanitize_directory → normalizer | `v0.1.38` → `v<VERSION>`; remove 3 following spaces. |
| `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_shows_missing_limits_message.snap` | `status_snapshot_shows_missing_limits_message` | sanitize_directory → normalizer | `v0.1.38` → `v<VERSION>`; remove 3 following spaces. |
| `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_shows_refreshing_limits_notice.snap` | `status_snapshot_shows_refreshing_limits_notice` | sanitize_directory → normalizer | `v0.1.38` → `v<VERSION>`; remove 3 following spaces. |
| `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_shows_stale_limits_message.snap` | `status_snapshot_shows_stale_limits_message` | sanitize_directory → normalizer | `v0.1.38` → `v<VERSION>`; remove 3 following spaces. |
| `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_shows_unavailable_limits_message.snap` | `status_snapshot_shows_unavailable_limits_message` | sanitize_directory → normalizer | `v0.1.38` → `v<VERSION>`; remove 3 following spaces. |
| `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_treats_refreshing_empty_limits_as_unavailable.snap` | `status_snapshot_treats_refreshing_empty_limits_as_unavailable` | sanitize_directory → normalizer | `v0.1.38` → `v<VERSION>`; remove 3 following spaces. |
| `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_truncates_halfwidth_kana_in_narrow_terminal.snap` | `status_snapshot_truncates_halfwidth_kana_in_narrow_terminal` | sanitize_directory → normalizer | `v0.1.38` → `v<VERSION>`; remove 3 following spaces. |
| `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_truncates_in_narrow_terminal.snap` | `status_snapshot_truncates_in_narrow_terminal` | sanitize_directory → normalizer | `v0.1.38` → `v<VERSION>`; remove 3 following spaces. |
| `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_uses_command_backed_provider_account_identity.snap` | `status_snapshot_uses_command_backed_provider_account_identity` | sanitize_directory → normalizer | `v0.1.38` → `v<VERSION>`; remove 3 following spaces. |
| `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_uses_default_reasoning_when_config_empty.snap` | `status_snapshot_uses_default_reasoning_when_config_empty` | sanitize_directory → normalizer | `v0.1.38` → `v<VERSION>`; remove 3 following spaces. |
| `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_uses_generic_limit_labels_for_unsupported_windows.snap` | `status_snapshot_uses_generic_limit_labels_for_unsupported_windows` | sanitize_directory → normalizer | `v0.1.38` → `v<VERSION>`; remove 3 following spaces. |
| `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__status_snapshot_wraps_enterprise_monthly_credit_details_in_narrow_terminal.snap` | `status_snapshot_includes_enterprise_monthly_credit_limit (narrow assertion)` | sanitize_directory → normalizer | `v0.1.38` → `v<VERSION>`; remove 3 following spaces. |
| `codex-rs/tui/src/status/snapshots/codex_tui__status__tests__transcript_overlay_status_rate_limit_refresh.snap` | `transcript_overlay_remeasures_status_after_rate_limit_refresh` | buffer_to_text → sanitize_directory → normalizer | `v0.1.38` → `v<VERSION>`; remove 3 following spaces in both before/after headers. |
| `codex-rs/tui/src/history_cell/snapshots/codex_tui__history_cell__tests__session_info_availability_nux_tooltip_snapshot.snap` | `session_info_availability_nux_tooltip_snapshot` | render_transcript → render_lines → normalizer | `v0.1.31` → `v<VERSION>`; remove 3 following spaces. |
| `codex-rs/tui/src/snapshots/codex_tui__app__tests__model_change_refreshes_committed_session_header.snap` | `model_change_refreshes_committed_session_header_snapshot` | assert_app_snapshot! → normalizer | `v0.1.35` → `v<VERSION>`; remove 3 following spaces. |
| `codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__guardian_approved_request_permissions_renders_request_summary.snap` | `chatwidget::tests::guardian::guardian_approved_request_permissions_renders_request_summary` | `normalize_snapshot_paths` | `Ambient GLM 5.2 standard` → `Ambient GLM 5.2 via ambient standard`; matches the existing provider display contract. |

The 23 version-pinned files contain 24 version occurrences because the transcript
overlay golden contains before/after headers. Adding guardian gives **24 stale
snapshot files addressed**. The brief's count of 23 version-pinned files is
correct, but its assertion that all say 0.1.38 is incorrect: 21 say 0.1.38,
history says 0.1.31, and app says 0.1.35. The count is a static inventory, not a
claim that a baseline full-suite run was performed.

## Other header snapshot tests already using fixed fixtures

These six additional goldens were not stale and are not changed:

| Test | Golden suffix | Normalization coverage |
| --- | --- | --- |
| `app::tests::clear_ui_after_long_transcript_snapshots_fresh_header_only` | `clear_ui_after_long_transcript_fresh_header_only` | `assert_app_snapshot!` calls normalizer; existing `<VERSION>` input remains unchanged. |
| `app::tests::ctrl_l_clear_ui_after_long_transcript_reuses_clear_header_snapshot` | same golden | Same shared macro and fixed input. |
| `app::tests::clear_ui_header_shows_fast_status_for_fast_capable_models` | `clear_ui_header_fast_status_fast_capable_models` | Same shared macro and fixed input. |
| `history_cell::tests::session_header_clamps_to_narrow_width` | same test name | `render_lines` calls normalizer; deliberate `test` version remains literal. |
| `history_cell::tests::session_header_indicates_yolo_mode` | same test name | Same helper and fixed `test` version. |
| `history_cell::tests::session_header_aligns_halfwidth_sound_marks` | `session_header_halfwidth_sound_marks` | Direct buffer assertion calls normalizer; fixed `test` remains literal. |
| `history_cell::tests::session_header_truncates_halfwidth_directory` | `session_header_halfwidth_directory` | Direct buffer assertion calls normalizer; fixed `test` remains literal. |

The history rendering helper also covers non-snapshot header checks such as the
availability-tooltip, tooltip-disabled and first-event cases. Those substring
checks do not claim to validate version presence.

## Guardian decision

Keep product code unchanged. `model_with_provider_display_name` in
`chatwidget/status_surfaces.rs` explicitly appends ` via {provider}` for every
non-openai provider. The existing `esc_interrupt_goal_paused_footer` golden also
contains `Ambient GLM 5.2 via ambient standard`. Therefore guardian's old golden
is stale. **Cosmetic observation:** `Ambient GLM 5.2 via ambient` names the
provider twice; intentionally left unchanged.

## Validation

Initial shared-target permission attempt: exit 100; 91 tests run, 87 passed,
1 failed, 3 timed out, 4074 skipped. Each nonpassing case failed/timed out on
both nextest attempts. Exact names:

- `status::tests::status_snapshot_shows_auto_review_permissions` (snapshot failure).
- `app::permission_confirmation::tests::permission_confirmation_native_requests_carry_effective_defaults` (timeout).
- `app::permission_confirmation::tests::permission_confirmation_dispatch_preserves_policy_for_fresh_threads` (timeout).
- `app::tests::app_server_spawn_preserves_active_thread_yolo_permissions` (timeout).

This attempt is **not final-tree qualification**: although Cargo compiled from the
assigned worktree and output contained our normalized placeholder, insta read
the old golden and wrote .snap.new in
`/Volumes/CorbanuDrive/Corbanu/worktrees/acct-activation-20260916/`.
The run inherited shared `CARGO_TARGET_DIR=/Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/target`.
Cross-worktree snapshot lookup is directly observed; the exact shared-artifact or
metadata race is not established. No foreign snapshot was accepted, inspected,
deleted or otherwise edited by this worker. No attribution of the three timeouts
is claimed. See `tui-snapshot-rot-21-permission.log`.

Subsequent guarded runs use a dedicated build directory beneath the assigned QA
path: `qa/initiative-control/pf-80-s01/.cache/tui-snapshot-rot-21-target`.
This changes build-artifact placement only; profile and native-keyring guards
remain in effect.

First dedicated-target permission attempt: exit 100; 91 run, 90 passed,
1 failed, 4074 skipped. Both repaired permission snapshots passed. Exact failure:
`suite::provider_convergence::tmux_permission_reload_preserves_active_runtime_over_saved_default`
failed twice because the fresh target lacked the `codex` executable.
A normal debug `cargo build -p codex-cli --bin codex` was launched in that target
with debug assertions enabled to supply the integration fixture's executable;
this is a build command, not a raw test invocation.

First dedicated-target status attempt: exit 100; 386 run, 373 passed,
13 failed, 3779 skipped. All 21 version-bearing status goldens passed.
Exact failed cases (each failed on both nextest attempts):

- `chatwidget::tests::exec_flow::unified_exec_begin_restores_working_status_snapshot`
- `chatwidget::tests::exec_flow::unified_exec_wait_status_renders_command_in_single_details_row_snapshot`
- `chatwidget::tests::guardian::guardian_parallel_reviews_render_aggregate_status_snapshot`
- `chatwidget::tests::exec_flow::preamble_keeps_working_status_snapshot`
- `chatwidget::tests::status_and_layout::chatwidget_tall`
- `chatwidget::tests::guardian::guardian_cleanup_drops_stale_reviews_and_restores_mcp_status`
- `chatwidget::tests::exec_flow::image_generation_begin_restores_working_status_after_single_line_preamble`
- `chatwidget::tests::status_and_layout::reasoning_delta_restores_recreated_status_indicator_header`
- `chatwidget::tests::status_and_layout::status_widget_active_snapshot`
- `chatwidget::tests::status_surface_previews::status_surface_preview_lines_hardcoded_only_snapshot`
- `chatwidget::tests::status_surface_previews::status_surface_preview_lines_mixed_snapshot`
- `status::snapshot_helpers::tests::status_header_version_normalization_preserves_frame_across_releases`
- `suite::provider_management::tmux_shared_and_custom_catalog_have_management_status_parity`

The new normalizer's width regression test had an incorrect expected space count:
the 40-column field contains a 32-character normalized header, leaving eight
spaces, not nine. Only the expected test string was corrected and formatted;
the normalizer and all 24 edited golden files remained unchanged.

Nine footer snapshot failures show ` via ambient` and the consequent line
truncation. Two preview failures show `tmp` → `my-project` in the status preview,
and `tmp` → `project` in the mixed terminal title. These are explained by
ProjectRoot preview placeholder `my-project` when no project root is found,
and the terminal-title fallback to the basename of /tmp/project. These fixtures
do not seed a fixed project-root cache, unlike the nearby popup snapshot fixture.
Neither preview contains an application version header, so the new normalizer
returns those strings unchanged. No claim of baseline attribution or product
correctness is made; all 11 additional goldens remain unaccepted and their
.snap.new files are preserved. The integration failure in the status gate also
reported the missing `codex` executable.

Final dedicated-target gates, after the corrected regression expectation was
formatted and the debug CLI fixture executable built:

| Guarded command | Exit | Run | Passed | Failed | Timed out | Skipped | Log |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `just test -p codex-tui permission` | 0 | 91 | 91 | 0 | 0 | 4074 | `tui-snapshot-rot-21-permission-final.log` |
| `just test -p codex-tui status` | 100 | 386 | 375 | 11 | 0 | 3779 | `tui-snapshot-rot-21-status-final.log` |
| `just test -p codex-tui usage` | 0 | 91 | 91 | 0 | 0 | 4074 | `tui-snapshot-rot-21-usage.log` |

Permission and usage have no failure names. Usage has one passing test marked
`LEAK` by nextest:
`chatwidget::goal_status::tests::active_goal_usage_reports_time_without_budget`.
This observation is not concealed or attributed to the base.

The 11 final status failures (all failed both nextest attempts):

- `chatwidget::tests::exec_flow::image_generation_begin_restores_working_status_after_single_line_preamble`
- `chatwidget::tests::exec_flow::preamble_keeps_working_status_snapshot`
- `chatwidget::tests::exec_flow::unified_exec_wait_status_renders_command_in_single_details_row_snapshot`
- `chatwidget::tests::status_and_layout::chatwidget_tall`
- `chatwidget::tests::exec_flow::unified_exec_begin_restores_working_status_snapshot`
- `chatwidget::tests::guardian::guardian_cleanup_drops_stale_reviews_and_restores_mcp_status`
- `chatwidget::tests::guardian::guardian_parallel_reviews_render_aggregate_status_snapshot`
- `chatwidget::tests::status_and_layout::reasoning_delta_restores_recreated_status_indicator_header`
- `chatwidget::tests::status_and_layout::status_widget_active_snapshot`
- `chatwidget::tests::status_surface_previews::status_surface_preview_lines_mixed_snapshot`
- `chatwidget::tests::status_surface_previews::status_surface_preview_lines_hardcoded_only_snapshot`

All three normalizer regression tests and all 21 version-bearing status goldens
pass. The final focused header run also passed: **16 run, 16 passed, 0 failed,
0 timed out, 4149 skipped, exit 0**. It verifies both remaining changed header
goldens, all existing fixed-fixture header goldens, and associated header behavior.
Exact guarded filter:

```sh
just test -p codex-tui -E 'test(history_cell::tests::session_) | test(app::tests::model_change_refreshes_committed_session_header_snapshot) | test(app::tests::clear_ui_) | test(app::tests::ctrl_l_clear_ui_)'
```

See `tui-snapshot-rot-21-headers.log`. All final commands used the dedicated
CARGO_TARGET_DIR described above. Both repaired permission cases pass. Thus all
24 intentionally edited goldens pass; **the broad status gate remains red**.

The exact 11 unaccepted snapshot diffs and filenames are preserved in
[tui-snapshot-rot-21-unaccepted-snapshots.md](tui-snapshot-rot-21-unaccepted-snapshots.md),
with the original generated .snap.new files retained beside their goldens. No
unexplained or additional golden changes were accepted. These observations
extend the brief's inventory, but do not establish baseline attribution.

The fixture-created `tui/target/tmux-artifacts` outputs were archived under this
assigned QA directory (`tui-snapshot-rot-21-shared-target-tmux-permission`,
`tui-snapshot-rot-21-final-tmux-permission`, and
`tui-snapshot-rot-21-final-tmux-status`); their now-empty source-tree directories
were removed. No native credential prompt was observed; no live profile was used.
The source tree has not changed since the final test runs.
Read docs/development/test-isolation.md before tests. Only guarded `just test`
is used; no live profile or credential access is authorized.

Formatting: only changed Rust files, with
`rustfmt --edition 2024 --config skip_children=true`; no `just fmt` or `just fix`.
Rustfmt warned that the repository's nightly-only imports_granularity setting
is unavailable on the active stable formatter. No formatter edits escaped scope.

Non-test runtime source lines changed: **0**. The three added lines in status/mod.rs
are a blank line and a cfg(test)-guarded module declaration, exclusively test
wiring. No production display, authorization or accounting code was modified.
Final read-only audit: `git diff --check` passed; exactly 24 tracked golden files
changed; no literal numeric release version remains in the three header snapshot
directories; every modified/untracked final path is inside the assigned writable
scope. New test helpers and evidence are untracked and must be included by the
integrator when collecting this repair. No commit or push performed.
