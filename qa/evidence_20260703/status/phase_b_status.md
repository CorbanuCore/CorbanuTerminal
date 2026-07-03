# Phase B Status — TUI Re-Verification

## 2026-07-03 03:56:52 UTC — tasknodeorc verifier readiness gate

Status: BLOCKED / WAITING FOR FIXER READY MARKER

Verifier directive: `/home/pfrpc/repos/orc_directives/tasknodeorc_directive_qa_verify_0400.md`

Observed state:

- Required coordination file was absent when verifier started this gate.
- Branch checkout present: `/home/pfrpc/repos/PfTerminal-bench`
- Branch: `qa/release-fixes-20260703`
- HEAD: `d86721e33`
- `main` and `origin/main` also resolve to `d86721e33`
- Worktree status: clean
- `qa_tui` session: absent

Gate decision:

- No Phase B TUI re-verification was run.
- Reason: the directive says to re-run failing TUI scenarios for each fix marked ready by gorkul in this status file. No fix is marked ready, and the branch is still the same SHA used for the Phase A failure report.
- Running another live TUI repro now would spend against the Vercel GLM cap without testing a changed build.

Next verifier action:

- When gorkul marks a specific fix ready in this file and the branch checkout advances or contains the named fix, rebuild that branch binary and re-run the matching Phase A failing scenario in a fresh `qa_tui` tmux session.

## 2026-07-03 04:06 UTC — gorkul fix marker R4/R5a

Branch: `qa/release-fixes-20260703`
Commit: `3fefcd6f9` (`Fix provider info test initializers`)

Finding status:

- R4 REGRESSION fixed: added `chat_completions_provider: None` to downstream `ModelProviderInfo` test initializers in `codex-login` and `codex-app-server`.
- R5 partial REGRESSION fixed: updated `GetAuthStatusResponse.ts` for the new `hasCodexBackendAuth` field.
- R5 pre-existing documented: `codex-app-server-protocol` still has the base-red duplicate `ThreadStartParams` JSON schema collision and base-red `v2/ThreadItem.ts` `taskPreview` fixture mismatch. These were already red at `8efcb8e46`.

Tests:

- PASS: `just test -p codex-login` (`158 passed`)
- PASS: `just test -p codex-app-server derive_config_from_params_uses_session_thread_config_model_provider`
- EXPECTED PRE-EXISTING FAIL: `just test -p codex-app-server-protocol typescript_schema_fixtures_match_generated` now reports only the base-red `v2/ThreadItem.ts` mismatch, not `GetAuthStatusResponse.ts`.

Verifier action:

- No TUI re-verification needed for this mechanical compile/schema-fixture fix.

## 2026-07-03 04:10 UTC — gorkul fix marker R3a

Branch: `qa/release-fixes-20260703`
Commit: `fc62a3eab` (`Remove TUI auth manager dependency`)

Finding status:

- R3 REGRESSION fixed: removed direct `AuthManagerConfig` dependency from TUI runtime source. `app_server_session.rs` now uses the resolved `Config::auth_keyring_backend_kind()` helper instead of importing the login manager trait.
- R3 remaining: full TUI suite still needs classification/fixes for snapshot/model-picker/status failures. Base TUI classification was blocked by `ENOSPC` after compiling the detached base worktree; generated base `target/` was deleted to recover disk.

Tests:

- PASS: `just test -p codex-tui tui_runtime_source_does_not_depend_on_manager_escape_hatches`

Verifier action:

- No live TUI re-verification needed for this layering fix alone. Dispatch-cluster fixes will get separate ready markers.

## 2026-07-03 04:37 UTC — gorkul fix marker R2/dispatch

Branch: `qa/release-fixes-20260703`
Commit: `31b591e473b621e17931a6719d2703ace2bc18d4` (`Fix spawn report dispatch retention`)

Finding status:

- R2 REGRESSION fixed: pending child reports are no longer dropped when a queued parent report-processing task is rejected. The app event sender now has a checked send path, queued report flushes keep retry state until the event is accepted, and `SubmitSpawnAgentTask` failure paths requeue report-processing prompts for retry.
- Dispatch live-finding fix: child reports delivered to `codex-main`/the primary thread are recorded and surfaced as reports, but are no longer auto-submitted as Main model turns. This removes the observed phantom `Task sent to Main [default]` follow-up that could leave Main busy after child completions.
- Dispatch live-finding guard: `/spawn status` task action rows now have a regression test that executing `Send task to Burzum [troll]` and `Send task to Snaga [orc]` opens a task prompt for the selected thread only.

Tests:

- PASS: `just test -p codex-tui flushed_child_report_is_requeued_when_parent_submission_fails`
- PASS: `just test -p codex-tui child_report_to_codex_main_is_recorded_without_auto_submitting_main_turn`
- PASS: `just test -p codex-tui child_report` (`9 passed`)
- PASS: `just test -p codex-tui pane_spawn_tree_hides_task_actions`
- PASS: `just test -p codex-tui dispatch` (`37 passed`)

Verifier action:

- tasknodeorc should rebuild `qa/release-fixes-20260703` at `31b591e47` or later and re-run the Phase A live TUI dispatch sequence: create Burzum Troll and Snaga Orc, send exact-response tasks through `/spawn status`, confirm tasks run only in selected child panes, confirm Main receives visible child-report delivery instead of an automatic Main turn, and confirm Main remains responsive afterward.

## 2026-07-03 04:32:49 UTC — tasknodeorc verifier R2/dispatch live TUI result

Status: FAIL

Build verified:

- Checkout: `/home/pfrpc/repos/PfTerminal-bench`
- Branch: `qa/release-fixes-20260703`
- HEAD at test start: `31b591e473b621e17931a6719d2703ace2bc18d4`
- Build command: `cargo build -p codex-cli --bin pfterminal`
- Build result: PASS
- Binary: `/home/pfrpc/repos/PfTerminal-bench/codex-rs/target/debug/pfterminal`
- Binary timestamp/size: `2026-07-03 04:25:36.366489292 +0000`, `1369109040` bytes

Live TUI run:

- tmux session: `qa_tui`
- Scratch dir: `/home/pfrpc/repos/pfterminal_qa_20260703/scratch_phase_b`
- Evidence dir: `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence_phase_b`
- Provider/model: `vercel-anthropic-fast` + `zai/glm-5.2-fast`
- Permissions: YOLO mode

Verifier verdicts:

- Spawn hierarchy creation: PASS.
  - Evidence: `tui_evidence_phase_b/007_troll_spawned.txt`, `tui_evidence_phase_b/013_orc_spawned.txt`, `tui_evidence_phase_b/016_spawn_status_initial.txt`
  - Burzum and Snaga spawned on the intended current `zai/glm-5.2-fast` route, and `/spawn status` showed addressable send-task rows.
- Burzum targeted send isolation: PARTIAL PASS.
  - Evidence: `tui_evidence_phase_b/019_burzum_task_confirmed.txt`
  - The old immediate duplicate `Task sent to Main [default]` line did not appear. The screen showed only `Task sent to Burzum [troll]`.
- Child report delivery to Main: FAIL.
  - Evidence: `tui_evidence_phase_b/020_burzum_report_wait20.txt`, `tui_evidence_phase_b/022_status_after_burzum.txt`
  - After 20 seconds, Main did not show a visible child-report block for Burzum. `/spawn status` showed Burzum `done` with `latest result: QA_TROLL_REPORT_DELIVERED`, so the result existed but was only visible through status.
- Snaga targeted send isolation / correct parent path: FAIL.
  - Evidence: `tui_evidence_phase_b/025_snaga_task_confirmed.txt`, `tui_evidence_phase_b/028_status_after_snaga.txt`
  - After `Task sent to Snaga [orc]`, the exact Snaga task text appeared on Main as a normal prompt, Main produced `QA_ORC_REPORT_DELIVERED`, and the UI emitted `Task sent to Burzum [troll]`.
  - `/spawn status` then showed Snaga done, while Burzum's current task was the child-report review prompt for Snaga. This reproduces the Phase A wrong-parent/report-routing failure.
- Main responsiveness after dispatch stress: FAIL.
  - Evidence: `tui_evidence_phase_b/029_file_edit_wait35.txt`, `tui_evidence_phase_b/031_file_edit_wait70.txt`, `tui_evidence_phase_b/030_qa_loop_check_after35.txt`, `tui_evidence_phase_b/032_qa_loop_check_after70.txt`, `tui_evidence_phase_b/035_main_responsive_wait25.txt`
  - The post-dispatch file-edit task produced no visible output after about 70 seconds, and `qa_loop.txt` was still missing.
  - After Escape then Ctrl-C recovery, a simple `Reply exactly QA_MAIN_RESPONSIVE.` prompt produced no output within 25 seconds.
- P2 slash command/picker behavior: STILL REPRODUCES / P2 ONLY.
  - Evidence: `tui_evidence_phase_b/003_spawn_first_enter.txt`, `tui_evidence_phase_b/008_spawn_for_orc_first_enter.txt`
  - `/spawn` remained in the composer on first Enter and required a second Enter. This was not part of the R2 fix marker, so it remains a separate P2.

Gate result:

- Phase B dispatch gate: FAIL.
- No full release sign-off. The priority dispatch re-verification did not pass, so streaming/rejection child-report variants and the broader final Phase A regression sweep were not run in this attempt.

Cleanup:

- `qa_tui` killed after capture.
- New evidence key scan: PASS, `hit_count=0`.
  - Evidence: `tui_evidence_phase_b/key_scan.txt`
- No verifier fixes or merges performed.

Branch hygiene note:

- Two untracked `.snap.new` files were present before this verification and were not touched by the verifier.
- After the live run, `git status` also showed modified `codex-rs/core/src/agent/role.rs` and `codex-rs/core/src/config/mod.rs`. Those source edits were not made by the verifier and were not part of the committed `31b591e47` binary rebuilt for this run.
- Final housekeeping check also showed modified `codex-rs/tools/src/tool_search.rs` and `codex-rs/tools/src/tool_search_tests.rs`. These appeared after the verifier run and were likewise not made by the verifier or included in the verified binary.

## 2026-07-03 04:47:49 UTC — gorkul fix marker R2/dispatch live findings follow-up

Commit: `b096e82be` (`Fix spawn dispatch report visibility`)

Finding status:

- R2 live report visibility fixed: child reports destined for Codex Main now surface as visible history even when a child native pane is the active thread, while still avoiding automatic Main model turns.
- R2 live operator-pane isolation fixed: creating native spawn workers no longer switches the operator into the new worker pane when a current thread is already active, so later human prompts stay on the operator/root pane unless the operator explicitly selects a child.
- R2 live noise reduced: automatic child-report processing turns no longer emit misleading `Task sent to ...` info toasts; explicit user-dispatched tasks still do.

Tests:

- PASS: `just test -p codex-tui child_report_to_codex_main_is_visible_even_when_child_thread_active child_report_to_codex_main_is_recorded_without_auto_submitting_main_turn flushed_child_report_is_requeued_when_parent_submission_fails child_report_to_idle_parent_triggers_a_processing_turn pane_spawn_tree_hides_task_actions`
- PASS: `just test -p codex-tui child_report dispatch` (`47 passed`)
- PASS: `cargo fmt --check` (rustfmt emitted existing stable-channel warnings for `imports_granularity`, but exited 0)

Verifier action:

- tasknodeorc should rebuild `qa/release-fixes-20260703` at `b096e82be` or later and re-run the Phase B live TUI dispatch sequence. Expected changes from the failed `31b591e47` run: after spawning Burzum/Snaga the active operator pane remains Main/root; Burzum's report is visibly surfaced; Snaga's exact-response task should not appear as a Main prompt; no misleading automatic `Task sent to Burzum` toast should appear for child-report processing; a post-dispatch Main exact-response/file-edit prompt should remain responsive.

## 2026-07-03 04:56:41 UTC — gorkul fix marker R6/core regressions

Branch: `qa/release-fixes-20260703`
Commit: `9013d06f0` (`Fix ambient model and deferred tool regressions`)

Finding status:

- R6 REGRESSION fixed: model-only config no longer gets normalized to the ambient provider default model when no provider is explicitly selected. Explicit override models still win, and ambient reasoning-effort normalization is only applied to ambient defaults or ZAI chat providers.
- R6 REGRESSION fixed: role reload now preserves explicit role model and role reasoning effort after `reload::build_next_config`, and preserves the current model when the role does not set model runtime fields.
- R6 REGRESSION fixed: deferred tool-search entries with curated search text now also include generated namespace/tool/parameter metadata, restoring recall for role-specific `spawn_agent` metadata and model-visible deferred tools.
- R6 PRE-EXISTING documented: `guardian_review_does_not_retry_valid_denial` is red at pre-merge base `8efcb8e46` with the same `request_log.requests().len() == 0` failure. The guardian retry/denial cluster remains outside the Phase B regression gate.

Tests:

- PASS: `just test -p codex-tools custom_search_text_is_augmented_with_spec_metadata`
- PASS: `just test -p codex-core apply_role_ignores_agent_metadata_fields_in_user_role_file apply_role_preserves_unspecified_keys apply_role_takes_precedence_over_existing_session_flags_for_same_key spawn_agent_role_overrides_requested_model_and_reasoning_settings spawn_agent_tool_description_mentions_role_locked_settings load_config_ambient_provider_replaces_stale_openai_model load_config_defaults_to_ambient_provider_and_model load_config_rejects_unsupported_amazon_bedrock_overrides specs_filter_deferred_dynamic_tools extension_tool_executors_are_model_visible_and_dispatchable code_mode_only_exposes_configured_dynamic_namespace_directly excluded_deferred_namespaces_do_not_enable_nested_tool_guidance mcp_and_tool_search_follow_direct_and_deferred_tool_exposure deferred_extension_tools_are_discoverable_with_tool_search environment_count_controls_environment_backed_tools tool_search_cache_rebuilds_when_deferred_sources_change v1_multi_agent_tools_defer_when_tool_search_available`
- PASS: `cargo fmt --check` (rustfmt emitted existing stable-channel warnings for `imports_granularity`, but exited 0)

Verifier action:

- No live TUI verification needed for this core/config/tools regression set. R6 release gate remains tied to the targeted tests above plus final suite/key-scan status.

## 2026-07-03 05:00:24 UTC — gorkul fix marker R1/chat stream keepalive

Branch: `qa/release-fixes-20260703`
Commit: `60014b5f6` (`Keep chat streams alive on SSE comments`)

Finding status:

- R1 REGRESSION fixed: chat-completions SSE comment frames such as `: OPENROUTER PROCESSING` now feed the actionable-silence deadline. A stream that continues receiving comment keepalives no longer reconnects before late content/tool/reasoning deltas arrive.
- R1 guard preserved: truly silent streams still fail through the byte-level idle timeout path.
- R1 belt already present before this marker: built-in OpenRouter defaults use a 600s stream idle timeout, built-in providers accept transport retry/timeout overrides, and the same-request idle guard aborts after the second repeated idle failure instead of silently restarting up to five times.

Tests:

- PASS: `just test -p codex-api comment_frames_keep_idle_timer_alive comment_only_stream_survives_actionable_silence_timeout truly_silent_stream_hits_idle_timeout`
- PASS: `just test -p codex-core same_request_idle_guard_aborts_after_second_idle_failure same_request_idle_guard_resets_on_non_idle_error long_stream_failure_retries_are_capped_to_one normal_stream_failures_keep_provider_retry_count`
- PASS: `just test -p codex-model-provider-info test_built_in_model_providers_include_openrouter configured_built_in_provider_can_override_transport_knobs`
- PASS: `cargo fmt --check` (rustfmt emitted existing stable-channel warnings for `imports_granularity`, but exited 0)

Verifier action:

- No live TUI verification needed for this stream parser unit fix. Release gate still requires final suite/key-scan status and tasknodeorc dispatch sign-off at `b096e82be` or later.

## 2026-07-03 05:04:32 UTC — gorkul Phase B local verification status

Branch: `qa/release-fixes-20260703`
HEAD: `60014b5f6` (`Keep chat streams alive on SSE comments`)
Repo status: clean; branch is 6 commits ahead of `origin/main`

Local verification:

- PASS: `just test -p codex-api` (`149 passed`)
- PASS: `just test -p codex-model-provider-info` (`40 passed`)
- PASS: `just test -p codex-config` (`189 passed`)
- PASS: `just test -p codex-login` (`158 passed`)
- PASS: `just test -p codex-app-server derive_config_from_params_uses_session_thread_config_model_provider`
- EXPECTED PRE-EXISTING FAIL: `just test -p codex-app-server-protocol typescript_schema_fixtures_match_generated` still fails only on the base-red `v2/ThreadItem.ts` `taskPreview` fixture mismatch. The `GetAuthStatusResponse.hasCodexBackendAuth` fixture regression remains fixed.
- PASS: `just test -p codex-tui child_report dispatch` (`47 passed`)
- PASS: R6 targeted core/tools tests listed in the `9013d06f0` marker.
- PASS: R1 stream/retry/provider targeted tests listed in the `60014b5f6` marker.
- PASS: branch-diff and QA-artifact key scan (`diff_secret_hit_count=0`, `qa_secret_hit_count=0`).

Open Phase B gate:

- BLOCKED/PENDING external verifier: tasknodeorc live TUI dispatch sign-off at `b096e82be` or later is still required by the directive. No merge/Phase C action should proceed until that live sign-off appears in this QA directory.

## 2026-07-03 05:16:04 UTC — tasknodeorc verifier Round-2 live TUI result

Status: FAIL — priority dispatch gate still not fully green

Build verified:

- Checkout: `/home/pfrpc/repos/PfTerminal-bench`
- Branch: `qa/release-fixes-20260703`
- HEAD at test start: `60014b5f6`
- Note: branch HEAD is later than requested `9013d06f0` and includes `9013d06f0`, `b096e82be`, and `31b591e47`.
- Build command: `cargo build -p codex-cli --bin pfterminal`
- Build result: PASS
- Binary: `/home/pfrpc/repos/PfTerminal-bench/codex-rs/target/debug/pfterminal`
- Binary timestamp/size: `2026-07-03 05:10:20.046510409 +0000`, `1369119816` bytes

Live TUI run:

- tmux session: `qa_tui`
- Scratch dir: `/home/pfrpc/repos/pfterminal_qa_20260703/scratch_phase_b_round2`
- Evidence dir: `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence_phase_b_round2`
- Provider/model: `vercel-anthropic-fast` + `zai/glm-5.2-fast`
- Permissions: YOLO mode

Verifier verdicts:

- Startup/status: PASS.
  - Evidence: `tui_evidence_phase_b_round2/002_trusted_status.txt`
  - TUI loaded with `zai/glm-5.2-fast xhigh` and YOLO mode.
- P2 slash command stickiness: STILL REPRODUCES.
  - Evidence: `tui_evidence_phase_b_round2/003_spawn_first_enter.txt`
  - `/spawn` remained in the composer on first Enter and required a second Enter.
- Spawn hierarchy / operator-pane isolation: PASS.
  - Evidence: `tui_evidence_phase_b_round2/007_troll_spawned.txt`, `tui_evidence_phase_b_round2/013_orc_spawned.txt`, `tui_evidence_phase_b_round2/015_spawn_status_initial.txt`
  - Burzum and Snaga spawned on the intended current `zai/glm-5.2-fast` route.
  - `/spawn status` showed `Nazgul: Codex - Main (current)`, while Burzum and Snaga were idle/addressable. The operator did not get switched into the child pane after spawn.
- Burzum targeted send + visible child report: PASS.
  - Evidence: `tui_evidence_phase_b_round2/018_burzum_task_sent.txt`
  - The send emitted only `Task sent to Burzum [troll]`.
  - Main visibly showed `Child report delivered. Burzum [troll]; status=done; result=QA_TROLL_REPORT_DELIVERED`.
  - No duplicate `Task sent to Main [default]` line and no automatic Main model turn appeared.
- Snaga targeted send isolation / wrong-Main-prompt regression: PASS.
  - Evidence: `tui_evidence_phase_b_round2/023_snaga_task_sent_wait8.txt`, `tui_evidence_phase_b_round2/024_snaga_report_wait33.txt`, `tui_evidence_phase_b_round2/026_status_after_snaga.txt`
  - The send emitted `Task sent to Snaga [orc]`.
  - The Snaga task text did not appear as a Main prompt and Main did not produce `QA_ORC_REPORT_DELIVERED`.
  - `/spawn status` later showed Snaga `done` with `latest result: QA_ORC_REPORT_DELIVERED`.
  - The child report was routed to Burzum without the previous misleading automatic `Task sent to Burzum [troll]` toast.
- Main responsiveness after dispatch stress: FAIL.
  - Evidence: `tui_evidence_phase_b_round2/027_file_edit_wait35.txt`, `tui_evidence_phase_b_round2/029_file_edit_wait70.txt`, `tui_evidence_phase_b_round2/028_qa_loop_check_after35.txt`, `tui_evidence_phase_b_round2/030_qa_loop_check_after70.txt`, `tui_evidence_phase_b_round2/032_main_responsive_wait25.txt`
  - The post-dispatch file-edit prompt produced no visible output after about 70 seconds.
  - `qa_loop.txt` was still missing after both checks.
  - After Ctrl-C recovery, `Reply exactly QA_MAIN_RESPONSIVE.` produced no visible response within 25 seconds.

Gate result:

- Priority dispatch gate: FAIL because Main responsiveness still fails after dispatch stress.
- Full Phase A regression sweep: NOT RUN. The directive said to continue into the full sweep only if the priority dispatch items passed.
- Release/Phase C sign-off: BLOCKED.

Cleanup:

- `qa_tui` killed after capture.
- New evidence key scan: PASS, `hit_count=0`.
  - Evidence: `tui_evidence_phase_b_round2/key_scan.txt`
- No verifier fixes or merges performed.

## 2026-07-03 05:33:44 UTC — gorkul fix marker Round 3 / Main responsiveness

Branch: `qa/release-fixes-20260703`
Commit: `37698e1d08488781224ad194e68c7c9338d3f9ba` (`Fix stale active-turn routing for idle panes`)

Finding status:

- Round-3 priority FAIL addressed: Main could render a new user prompt as an idle visible turn while the per-thread event store still retained an `active_turn_id` from an earlier turn. The submit path trusted that stale cached id and sent the visible prompt as `turn/steer` instead of `turn/start`, so no fresh Main turn appeared.
- Fix: for the currently displayed native thread, `active_turn_id_for_submission` now treats a cached active turn as stale when the visible chat widget is not task-running, clears it, and lets normal `turn/start` proceed. Inactive/background worker threads still use the event store as the authoritative liveness source, preserving child-report processing and background steering.
- The Round-2 captures match this failure mode: post-stress `qa_loop.txt` and `QA_MAIN_RESPONSIVE` prompts were visible in Main, but produced no output; `/spawn status` still showed Main selected, so the remaining failure was stale routing state rather than wrong-pane selection or frozen rendering.

Tests:

- PASS: `just test -p codex-tui active_turn_submission_clears_stale_visible_idle_turn_id active_turn_submission_keeps_inactive_thread_turn_id` (`2 passed`)
- PASS: `just test -p codex-tui child_report dispatch` (`47 passed`)
- PASS: `cargo fmt --check` (rustfmt emitted existing stable-channel warnings for `imports_granularity`, but exited 0)
- EXPECTED PRE-EXISTING FAIL: full `just test -p codex-tui` still fails the known unrelated snapshot/model-picker/status cluster (`3100 passed`, `21 failed`, `9 skipped`). The new active-turn tests and dispatch/child-report tests passed in that full run.

Verifier action:

- tasknodeorc should rebuild `qa/release-fixes-20260703` at `37698e1d0` or later and re-run only the Round-3 Phase B priority scenario in a fresh `qa_tui` session.
- Expected observations: after the same Burzum/Snaga dispatch stress, Main remains selected and responsive; the file-edit prompt starts a fresh Main turn, creates `qa_loop.txt` containing exactly `QA_FILE_EDIT_OK`, and replies `QA_FILE_EDIT_DONE`; after any Ctrl-C recovery, `Reply exactly QA_MAIN_RESPONSIVE.` produces the exact response instead of hanging.
- Stop gate remains in force: no merge and no Phase A sweep until tasknodeorc records the live Round-3 verifier result.

## 2026-07-03 05:46:41 UTC — tasknodeorc verifier Round-3 live TUI result

Status: FAIL — Main responsiveness still fails after dispatch stress

Build verified:

- Checkout: `/home/pfrpc/repos/PfTerminal-bench`
- Branch: `qa/release-fixes-20260703`
- HEAD at test start: `37698e1d0`
- Build command: `cargo build -p codex-cli --bin pfterminal`
- Build result: PASS
- Binary: `/home/pfrpc/repos/PfTerminal-bench/codex-rs/target/debug/pfterminal`
- Binary timestamp/size: `2026-07-03 05:40:19.013720455 +0000`, `1369128416` bytes

Live TUI run:

- tmux session: `qa_tui`
- Scratch dir: `/home/pfrpc/repos/pfterminal_qa_20260703/scratch_phase_b_round3`
- Evidence dir: `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence_phase_b_round3`
- Provider/model: `vercel-anthropic-fast` + `zai/glm-5.2-fast`
- Permissions: YOLO mode

Verifier verdicts:

- Startup/status: PASS.
  - Evidence: `tui_evidence_phase_b_round3/002_trusted_status.txt`
  - TUI loaded with `zai/glm-5.2-fast xhigh` and YOLO mode.
- Spawn hierarchy / operator-pane isolation: PASS.
  - Evidence: `tui_evidence_phase_b_round3/007_troll_spawned.txt`, `tui_evidence_phase_b_round3/013_orc_spawned.txt`, `tui_evidence_phase_b_round3/015_spawn_status_initial.txt`
  - Burzum and Snaga spawned on the intended current route.
  - `/spawn status` showed `Nazgul: Codex - Main (current)` with both child panes idle/addressable.
- Burzum dispatch/report: PASS.
  - Evidence: `tui_evidence_phase_b_round3/018_burzum_report.txt`
  - Main visibly showed `Child report delivered. Burzum [troll]; status=done; result=QA_TROLL_REPORT_DELIVERED`.
- Snaga dispatch isolation: PASS.
  - Evidence: `tui_evidence_phase_b_round3/023_snaga_task_wait12.txt`, `tui_evidence_phase_b_round3/026_status_after_snaga.txt`
  - Snaga task did not execute as a Main prompt.
  - `/spawn status` showed Snaga `done` with `latest result: QA_ORC_REPORT_DELIVERED`; Main remained current.
- Round-3 Main responsiveness target: FAIL.
  - Evidence: `tui_evidence_phase_b_round3/027_file_edit_wait45.txt`, `tui_evidence_phase_b_round3/028_qa_loop_check_after45.txt`, `tui_evidence_phase_b_round3/029_file_edit_wait90.txt`, `tui_evidence_phase_b_round3/030_qa_loop_check_after90.txt`
  - The post-dispatch file-edit prompt was visible on Main but produced no visible output after about 90 seconds.
  - `qa_loop.txt` was still missing at both filesystem checks.
- Ctrl-C recovery exact-response check: FAIL.
  - Evidence: `tui_evidence_phase_b_round3/031_file_edit_ctrlc.txt`, `tui_evidence_phase_b_round3/032_main_responsive_wait25.txt`
  - After Ctrl-C recovery, `Reply exactly QA_MAIN_RESPONSIVE.` was visible on Main but produced no visible response within 25 seconds.

Gate result:

- Round-3 priority gate: FAIL.
- Full Phase A regression sweep: NOT RUN. The user/directive required the full sweep only on PASS.
- Release/Phase C sign-off: BLOCKED.

Cleanup:

- `qa_tui` killed after capture.
- New evidence key scan: PASS, `hit_count=0`.
  - Evidence: `tui_evidence_phase_b_round3/key_scan.txt`
- Repo status after verifier run: clean on `qa/release-fixes-20260703` at `37698e1d0`.
- No verifier fixes or merges performed.

## 2026-07-03 06:11 UTC — gorkul Round-4 live repro stop gate

Status: STOP — target Main-responsiveness failure did not reproduce; no code changed.

Build/route:

- Checkout: `/home/pfrpc/repos/PfTerminal-bench`
- Branch: `qa/release-fixes-20260703`
- HEAD: `37698e1d0848`
- Binary build: PASS (`cargo build -p codex-cli --bin pfterminal`)
- Provider/model: `vercel-anthropic-fast` + `zai/glm-5.2-fast`

Live findings:

- PASS: manual live sequence spawned Burzum and Snaga, dispatched exact-response work, then Main created `qa_loop.txt` and replied `QA_FILE_EDIT_DONE` within the 45-second checkpoint.
- PASS: queued-slash timing attempt also left Main responsive; session log recorded a fresh `AppCommand::UserTurn` for the file-edit prompt and the file was created.
- PASS: controlled active Ctrl-C recovery check; after interrupting a `sleep 120` turn, Main replied `QA_MAIN_RESPONSIVE` within 25 seconds.
- OBSERVED: under queued-slash timing, Snaga reached `done` with `QA_ORC_REPORT_DELIVERED`, but the final bubbled Burzum report did not visibly render on Main before status inspection. Main still remained responsive afterward, so this is not the Round-3 Main hang.

Evidence:

- Dossier: `/home/pfrpc/repos/pfterminal_qa_20260703/main_responsiveness_dossier.md`
- Primary evidence: `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence_phase_b_round4_gorkul`
- Queued-slash timing evidence: `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence_phase_b_round4_gorkul_attempt3`
- Key scan over new Round-4 evidence: PASS.

Gate:

- Release remains held pending tasknodeorc verifier PASS or method-level logs from the verifier that reproduced the Round-3 FAIL.
- No merge, no further blind fix.

## 2026-07-03 06:36:04 UTC — tasknodeorc Round-4 arbitration attempt 1

Status: FAIL — reproduced tasknodeorc Main-responsiveness failure with method-level logging; stopped per arbitration rule.

Build verified:

- Checkout: `/home/pfrpc/repos/PfTerminal-bench`
- Branch: `qa/release-fixes-20260703`
- HEAD: `37698e1d0`
- Binary: `/home/pfrpc/repos/PfTerminal-bench/codex-rs/target/debug/pfterminal`
- Binary timestamp/size: `2026-07-03 05:40:19.013720455 +0000`, `1369128416` bytes

Run setup:

- Counted attempt evidence: `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence_phase_b_round4_tasknode_attempt1_method`
- Scratch dir: `/home/pfrpc/repos/pfterminal_qa_20260703/scratch_phase_b_round4_tasknode_attempt1_method`
- Provider/model: `vercel-anthropic-fast` + `zai/glm-5.2-fast`
- Session logging: `CODEX_TUI_RECORD_SESSION=1`, `CODEX_TUI_SESSION_LOG_PATH=.../session_log.jsonl`
- Input timing log: `tui_evidence_phase_b_round4_tasknode_attempt1_method/input_timing.log`
- Focused method extract: `tui_evidence_phase_b_round4_tasknode_attempt1_method/method_log_extract.txt`

Arbitration note:

- An earlier diagnostic launch without `CODEX_TUI_RECORD_SESSION=1` also reproduced the file-edit hang, but it lacked the required session log and is not counted as one of the three arbitration attempts.
- Once the counted attempt reproduced the failure, attempts 2 and 3 were not run, per the instruction: "If ANY run FAILS: write the failing run method-level logs + captures for gorkul and stop."

Sequence result:

- Spawn hierarchy / operator-pane isolation: PASS.
  - Evidence: `015_spawn_status_initial.txt`, `026_status_after_snaga.txt`
  - Status showed `Nazgul: Codex - Main (current)` and both Burzum/Snaga completed.
- Burzum dispatch/report: PASS.
  - Evidence: `018_burzum_report.txt`
  - Main visibly showed `Child report delivered. Burzum [troll]; status=done; result=QA_TROLL_REPORT_DELIVERED`.
- Snaga dispatch isolation: PASS.
  - Evidence: `023_snaga_task_wait15.txt`, `024_snaga_task_wait30.txt`, `026_status_after_snaga.txt`
  - Snaga task did not execute as a Main prompt; `/spawn status` showed Snaga `done` with `latest result: QA_ORC_REPORT_DELIVERED`.
- Main file-edit responsiveness: FAIL.
  - Evidence: `028_file_edit_wait45.txt`, `029_qa_loop_check_after45.txt`, `030_file_edit_wait90.txt`, `031_qa_loop_check_after90.txt`
  - The file-edit prompt was visible on Main, but produced no output after about 90 seconds.
  - `qa_loop.txt` was missing at both filesystem checks.
- Post-interrupt exact-response recovery: FAIL / no fresh Main submission.
  - Evidence: `032_sleep_turn_wait8.txt`, `033_sleep_ctrlc.txt`, `034_main_responsive_after_ctrlc.txt`, `method_log_extract.txt`
  - The visible exact-response prompt did not produce `QA_MAIN_RESPONSIVE`.

Method-level finding:

- `session_log.jsonl` recorded the child task submissions:
  - `SubmitSpawnAgentTask` for Burzum at `2026-07-03T06:32:20.070Z`
  - `SubmitSpawnAgentTask` for Snaga at `2026-07-03T06:32:35.142Z`
  - child-report task to Burzum at `2026-07-03T06:32:36.479Z`
- After the Main file-edit prompt was typed at `2026-07-03T06:33:09Z`, the session log recorded only a visible history append at `2026-07-03T06:34:47.246Z`.
- There was no subsequent `from_tui` `UserTurn` for the Main file-edit prompt.
- There was no `from_tui` `UserTurn` for the post-interrupt `Reply exactly QA_MAIN_RESPONSIVE.` prompt.
- Therefore this reproduction did not reach a lower app-server `turn/start` or `turn/steer` decision. The failure boundary is earlier: Main prompt text becomes visible/history-appended but no outbound Main `UserTurn` is emitted from the TUI.
- The session log also shows the later controlled sleep prompt was concatenated into the same visible history append as the stuck file-edit prompt, consistent with a composer/submission state problem after the dispatch sequence.

Discrepancy with gorkul Round-4:

- gorkul's Round-4 evidence did not reproduce the Main hang and showed file-edit + Ctrl-C recovery PASS.
- tasknodeorc's counted Round-4 attempt did reproduce the hang at the same commit, with method logging enabled.
- gorkul's unrendered/bubbled Snaga report observation remains P2 relative to this blocking Main-submission failure.

Cleanup:

- `qa_tui` killed after capture.
- Key scan over tasknodeorc Round-4 arbitration evidence: PASS, `hit_count=0`.
  - Evidence: `tui_evidence_phase_b_round4_tasknode_attempt1_method/key_scan.txt`
- Repo status after verifier run: clean on `qa/release-fixes-20260703` at `37698e1d0`.
- No verifier fixes or merges performed.

## 2026-07-03 06:48:48 UTC — gorkul fix marker Round-5 composer/submission swallow

Status: READY FOR TASKNODEORC ROUND-5 VERIFIER

Branch: `qa/release-fixes-20260703`
Commit: `f76b2de867ff13197761fc761b3cd36e0d8b1699` (`Fix long single-line paste burst submission`)

Log analysis / root cause:

- The Round-4 method evidence showed the file-edit prompt typed at `2026-07-03T06:33:09Z`, then no `from_tui` `UserTurn` for that prompt.
- The only later record was `AppendMessageHistoryEntry` at `2026-07-03T06:34:47.246Z`, and it contained the file-edit prompt concatenated with the later sleep prompt.
- That signature is not a lower app-server `turn/start` or `turn/steer` loss. It is the TUI composer path keeping the text as a draft, then Ctrl-C clearing that draft into visible history without dispatching an outbound turn.
- The specific boundary was fast terminal-injected long single-line input: paste-burst handling treated the real Enter key as a pasted newline, so the prompt never submitted. Short/multiline paste still keeps the existing Enter-as-newline behavior.

Fix:

- `PasteBurst` now exposes a long-single-line burst predicate.
- `ChatComposer::handle_submission_with_time` flushes any buffered burst chars and submits when Enter follows a long single-line burst outside slash-command context.
- Existing short/multiline paste suppression remains guarded by the old path.

Tests:

- PASS: `just test -p codex-tui long_single_line_burst_enter_submits ascii_burst_treats_enter_as_newline queued_submission_flushes_ascii_burst_instead_of_inserting_newline child_report dispatch active_turn_submission_clears_stale_visible_idle_turn_id active_turn_submission_keeps_inactive_thread_turn_id` (`52 passed`)
- PASS: `cargo fmt --check` (rustfmt emitted the existing stable-channel `imports_granularity` warnings, exit 0)
- PASS: `cargo build -p codex-cli --bin pfterminal`
- PASS: `git diff --check`

Live self-verification:

- Evidence: `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence_phase_b_round5_gorkul_fixed_live`
- Scratch: `/home/pfrpc/repos/pfterminal_qa_20260703/scratch_phase_b_round5_gorkul_fixed_live`
- Provider/model: `vercel-anthropic-fast` + `zai/glm-5.2-fast`
- Binary: `/home/pfrpc/repos/PfTerminal-bench/codex-rs/target/debug/pfterminal`
- Fast injected prompt: `Create file qa_loop.txt containing exactly QA_FILE_EDIT_OK and nothing else. Then reply QA_FILE_EDIT_DONE.`
- Session log result: `AppendMessageHistoryEntry` and `from_tui` `UserTurn` both recorded at `2026-07-03T06:48:05.702Z` for the exact prompt.
- Model result: `QA_FILE_EDIT_DONE`; file `qa_loop.txt` was created with `QA_FILE_EDIT_OK`.
- Key scan over new live evidence: PASS, no secret-shaped hits.

Verifier action:

- tasknodeorc should rebuild `qa/release-fixes-20260703` at `f76b2de86` or later and run the full Phase B dispatch/main-responsiveness sequence 3 times. A single PASS is not sufficient because the original failure is timing-dependent.
- Expected observation: after Burzum/Snaga dispatch and child-report traffic, a fast Main file-edit prompt must emit a `from_tui` `UserTurn` immediately, must not be concatenated with later prompts, and must not only appear as a delayed Ctrl-C history append.
- No merge/Phase C action should proceed until tasknodeorc records the required 3x live verifier PASS or returns new method-level failure evidence.

## 2026-07-03 07:12:11 UTC — tasknodeorc Round-5 verifier + final Phase A sweep

Status: PASS — Phase B TUI dispatch/Main-responsiveness gate is green at Round 5.

Build verified:

- Checkout: `/home/pfrpc/repos/PfTerminal-bench`
- Branch: `qa/release-fixes-20260703`
- HEAD: `f76b2de86` (`Fix long single-line paste burst submission`)
- Build command previously run for this verifier: `cargo build -p codex-cli --bin pfterminal`
- Binary: `/home/pfrpc/repos/PfTerminal-bench/codex-rs/target/debug/pfterminal`
- Binary timestamp/size: `2026-07-03 06:47:12.338498638 +0000`, `1369127560` bytes

Required Round-5 3x priority run:

- Aggregate result: PASS 3/3.
- Aggregate file: `/home/pfrpc/repos/pfterminal_qa_20260703/round5_tasknode_3x_result.env`
- Attempt evidence:
  - `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence_phase_b_round5_tasknode_attempt1`
  - `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence_phase_b_round5_tasknode_attempt2`
  - `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence_phase_b_round5_tasknode_attempt3`
- Each attempt spawned Burzum and Snaga, dispatched exact-response child tasks, then submitted the fast Main file-edit prompt.
- Method log result in all three attempts: file-edit `AppendMessageHistoryEntry` and outbound `from_tui` `UserTurn` were immediate (`delta_seconds=0.0`); post-Ctrl-C exact-response `UserTurn` was also immediate (`delta_seconds=0.001`); `concat_event_count=0`; `pass_all=True`.

Final Phase A regression sweep:

- Evidence: `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence_phase_b_final_sweep_tasknode`
- Scratch: `/home/pfrpc/repos/pfterminal_qa_20260703/scratch_phase_b_final_sweep_tasknode`
- Provider/model: `vercel-anthropic-fast` + `zai/glm-5.2-fast`
- Permissions: YOLO mode.
- PASS: startup loaded `zai/glm-5.2-fast xhigh` in YOLO mode.
- PASS: `/spawn` surface opened; Orc-before-Troll guard still blocks with `Spawn a Troll before creating Orc panes, then choose that Troll as supervisor.`
- PASS: Burzum Troll and Snaga Orc spawned on the intended current model route; `/spawn status` showed Main current with both child panes addressable.
- PASS: Burzum targeted dispatch produced visible `Child report delivered. Burzum [troll]; status=done; result=QA_TROLL_REPORT_DELIVERED`.
- PASS: Snaga targeted dispatch did not execute as a Main prompt; status showed Snaga done with `latest result: QA_ORC_REPORT_DELIVERED`.
- PASS: Main file-edit prompt emitted immediate `AppendMessageHistoryEntry` and `from_tui UserTurn` at `2026-07-03T07:09:23.926Z`; no concatenation with the later sleep prompt.
- PASS: Ctrl-C recovery prompt emitted immediate `AppendMessageHistoryEntry` and `from_tui UserTurn` at `2026-07-03T07:10:20.977Z` and produced `QA_MAIN_RESPONSIVE`.
- PASS: `/model` picker opened and showed `zai/glm-5.2-fast (current)`.
- PASS: `pfterminal exec` with closed stdin exited quickly with status `1` and `No prompt provided via stdin` instead of hanging.
- PASS: final sweep key scan, `hit_count=0`.

Notes / non-blocking residuals:

- P2 slash command stickiness still reproduces: first `/spawn` Enter leaves `/spawn` in the composer and requires a second Enter. Evidence: `003_spawn_orc_before_troll_first_enter.txt`.
- Final sweep strict file-content checker is intentionally recorded as `strict_pass_all=0` because the model wrote `qa_loop.txt` as `QA_FILE_EDIT_OK\n`. This is classified as model output variance, not the Round-5 TUI submission bug: the UI emitted the Main `UserTurn` immediately, the file-edit turn completed, and no delayed Ctrl-C append or prompt concatenation occurred. Evidence: `final_sweep_verdict.env`.

Cleanup:

- `qa_tui` killed after capture.
- No verifier code changes or merges performed.
- Repo status after verifier run: clean on `qa/release-fixes-20260703` at `f76b2de86`.

Gate decision:

- Phase B TUI dispatch/Main-responsiveness verifier: PASS.
- Release/Phase C remains subject to reviewer/owner decision and the existing non-TUI gates already recorded above; this verifier no longer blocks on the live TUI dispatch/Main-responsiveness path.
