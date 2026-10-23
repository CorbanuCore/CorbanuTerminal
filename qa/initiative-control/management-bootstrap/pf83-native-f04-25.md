# RETURN — pf83-native-f04-25

- Allocation digest: `b67f8ffa80d2a1b30117fe8a2e5387756064d54d4b847957abd2703ee42652c4`; claim: `b430af97-f7cf-4bea-b362-c2843b13961d`.
- Runtime: gpt-6-astra, high. Verified brief SHA-256: `7e597dbfbff3b988075fe51b5ffc0e3ac512d72d4572a1ff750f09ed0c3ec122`.
- Base: `44729e28937c3e75a6c786e8091ed64c9f486277`; worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/pf83-def015-20260916`; initially clean.
- Classification: routine test/evidence revision. Production/non-test Rust changes: **0 lines**. All existing tests retained; the MCP test is renamed to narrow its stated claim.
- Product citation: **Permission selection confirmation — TO BUILD**: “requested, applied-to-next-turn, failed and uncertain”; “Existing active-turn approval/sandbox snapshots and pending approvals are not retroactively changed.”
- This allocation changes no authorization policy. The F04 reproduction asserts a desired post-Applied steering contract awaiting a product decision; active-turn execution under its captured snapshot is authorized by the current contract. This is evidence of the expectation/message mismatch and missing re-bind rule, not a current-contract regression.
- Independent functional execution/review, true-TUI F04 qualification, live repositories, release, benchmarks and human acceptance are outside this routine revision. No human-test readiness claim; the applicable independent PF-83 functional gate remains open.

## Corrections

Exact ignore attribute:

```rust
#[ignore = "PF-83 F04: reproduces an unauthorized-behaviour expectation; un-ignore with the product decision"]
```

Original assertion text retained verbatim inside the new diagnostic:

> F04: work steered into the running turn after Applied must require approval and must not write after decline

The complete new hard-assertion message is:

> DESIRED post-Applied steering contract (pending product decision, not the current next-turn snapshot contract): F04: work steered into the running turn after Applied must require approval and must not write after decline; refusing or deferring admission into this turn with no write is also permitted

The hard assertion is `assert!(!wrote, ...)`. The additional predicate is `(admitted && approvals > 0) || (!model_received_steer && approvals == 0)`, where admitted requires same-turn acknowledgment and the steered text reaching model inference. All approvals are declined. Both restricted baseline and independent fresh-turn controls still require one approval and no marker.

A steer error or deferred acknowledgment can now take the non-admission path. Scripted inference emits the post-Applied write only when the steered text reaches its request. Resetting the mock sequence before the independent fresh-turn control prevents refusal/deferment from shifting that control's responses.

The pending-approval test reads buffered and arriving notifications using `read_stream_until_matching_notification` under a **250 ms** timeout after each confirmation. The outer timeout must expire; a resolution/completion notification or stream error fails the assertion. Marker absence and the original request's successful decline remain asserted.

The renamed `thread_settings_confirmation_leaves_mcp_status_responses_unaffected` preserves both isolated lanes, before/after tool-inventory checks, and complete status-response equality. Its claim is **MCP status responses are unaffected**. It explicitly does not prove equivalence of live-binding refresh, tool execution or authorization semantics.

Added `pf83-native-f04-23-tui-artifacts/README.md`, the brief's permitted alternative to a rename. It identifies those files as incidental PF-55 model-picker / active-runtime permission prerequisite replay artifacts, not F04 evidence. Historical artifacts/results are preserved.

Optional deadline correction completed: the barrier exec timeout is `DEFAULT_TIMEOUT * 6` (60 s), with the Python deadline derived as exec timeout minus `DEFAULT_TIMEOUT` (50 s). The 10 s readiness, confirmation, steering and completion waits remain bounded. This does not cure or claim to cure separate initialization failures.

## Brief inconsistencies and limits

1. “Keep the assertion text VERBATIM” conflicts with the subsequent requirements to change its message and predicate. The original message remains verbatim as a constituent string in the new desired-contract diagnostic; the old equality predicate is intentionally replaced as requested.
2. The MCP cross-lane comparison includes whole responses, whereas the per-lane assertions constrain only server count and tool names. Thus it is not strictly impossible for the comparison to fail independently (other response fields can differ). Nonetheless, equality of status responses does not establish live refresh semantics; that stronger claim is removed.
3. No product decision, regression attribution or approval is fabricated. Remaining gate failures will be listed without calling them pre-existing.

## Validation

All commands below ran from `codex-rs`, with `INSTA_UPDATE=no`, the checkout's guarded `just test`, disposable profiles and debug native-keyring denial. No native credential prompt was observed. No retry, snapshot update, production fix, commit or push was performed.

| Gate | Exact command | Exit | Results |
| --- | --- | --- | --- |
| App-server, F04 ignored | `just test -p codex-app-server --locked --offline --retries 0 --test-threads 2` | 100 | 1,093 run: **1,072 passed, 20 failed, 1 timed out, 2 skipped** |
| TUI permission | `just test -p codex-tui permission --locked --offline --retries 0 --test-threads 2` | 100 | 91 run: **89 passed, 2 failed, 4,071 skipped** |

Logs: `pf83-native-f04-25-app-server.log` and `pf83-native-f04-25-tui.log`, adjacent to this receipt. The TUI failure-artifact environment override pointed into this allocated evidence directory. Its passing PF-55 replay nevertheless writes success artifacts to a hard-coded `codex-rs/tui/target/tmux-artifacts/pf55-activeruntimepermissions` path; these four generated files were relocated unchanged into `pf83-native-f04-25-incidental-pf55-tui-artifacts/` and the empty generated directories removed. Its README identifies the scope correctly.

All four confirmation lifecycle tests and all four enabled `thread_settings_confirmation_*` tests passed. This includes the revised bounded-drain and MCP-status tests. The app-server suite is **not green**: 21 nonpassing cases remain in untouched tests. No remaining gate failure is in a test changed by this revision; causal/base attribution remains the manager's task, and these are not labeled pre-existing.

### Exact remaining app-server failure names

Each failed test below belongs to binary `codex-app-server::all`:

```text
suite::v2::account::login_account_chatgpt_redirects_to_hosted_success_page
suite::v2::executor_skills::restricted_executor_skill_is_listed_only_when_permitted
suite::v2::executor_skills::restricted_executor_skill_rejects_reference_until_permission_approved
suite::v2::git_attribution::git_attribution_follows_authenticated_workspace_policy
suite::v2::host_skills::host_skill_catalog_refreshes_once_when_skills_change
suite::v2::initialize::turn_start_notify_payload_includes_initialize_client_name
suite::v2::mcp_resource::orchestrator_skill_can_read_referenced_resource_without_an_executor
suite::v2::realtime_conversation::websocket_v2_assistant_output_without_handoff_reaches_realtime_context
suite::v2::realtime_conversation::websocket_v2_background_agent_progress_is_sent_before_function_output
suite::v2::realtime_conversation::websocket_v2_background_agent_returns_function_output
suite::v2::realtime_conversation::websocket_v2_background_agent_steering_ack_requests_response_create
suite::v2::realtime_conversation::websocket_v2_forwards_audio_and_text_between_client_and_sideband
suite::v2::realtime_conversation::websocket_v2_tool_call_delegated_turn_can_execute_shell_tool
suite::v2::realtime_conversation::websocket_v2_tool_call_does_not_block_sideband_audio
suite::v2::thread_delete::thread_delete_rejects_paginated_writer_owned_by_another_process
suite::v2::thread_read::paginated_history_lists_use_projected_turns_and_items
suite::v2::thread_read::paginated_thread_name_set_is_reflected_in_read_list_and_metadata_resume
suite::v2::thread_read::thread_search_occurrences_reads_paginated_projection
suite::v2::thread_resume::thread_resume_paginated_model_context_preserves_original_metadata
suite::v2::thread_resume::thread_resume_rejects_paginated_writer_owned_by_another_process
```

The timed-out test belongs to binary `codex-app-server`:

```text
in_process::tests::in_process_start_uses_requested_session_source_for_thread_start
```

### Exact remaining TUI failure names

Both belong to binary `codex-tui`:

```text
chatwidget::tests::guardian::guardian_approved_request_permissions_renders_request_summary
status::tests::status_snapshot_shows_auto_review_permissions
```

The incidental PF-55 TUI case `suite::provider_convergence::tmux_permission_reload_preserves_active_runtime_over_saved_default` passed in binary `codex-tui::all`; it is not F04 qualification.

### On-demand F04 and final checks

On-demand ignored reproduction: `just test -p codex-app-server --locked --offline --retries 0 --test-threads 1 --run-ignored only -E 'test(thread_settings_confirmation_f04_restricts_work_steered_after_applied)'`. **Exit 100; 1 run, 0 passed, 1 failed, 1,094 skipped.** Separate log: `pf83-native-f04-25-f04-on-demand.log`. It fails at the intended `assert!(!wrote, ...)` with the desired-contract message above, after both restricted controls pass. The wire log contains exactly two command approval requests: one for `baseline`, one for `next-turn`, neither for `after-applied`. The after-Applied command completed with exit 0 in the steered active turn and the filesystem marker existed (`wrote == true`). Thus it still fails today for the actual write, not for setup, mandatory admission or an exact approval-count assumption. The alternative refusal/deferment branch is allowed by the fixture but cannot be exercised against today's implementation without changing product behavior; no such qualification is claimed.

Formatting was restricted to `rustfmt --edition 2024 app-server/tests/suite/v2/thread_settings_update.rs` before all final tests. It succeeded with the existing stable-toolchain `imports_granularity` warning. No workspace formatter or fix command was used. Final `git diff --check` passes. Final `git status --short` shows only the allocated integration-test file and allocated QA evidence; generated TUI success files were relocated as described above. Rust diff: +112/−29 in one test file; the allocated unit-test file needed no revision. **Zero non-test Rust lines changed.**
