# Historical complete-Core failure triage and final disposition

This document preserves the failed repair-input run below. Its gate was closed
on 2026-08-29 after the sprint's scope was amended to enumerate every affected
runtime/test path. Final run `fd5920a2-8b87-4e14-a2b8-a7201aed6304` passed
3,411/3,411 tests with 19 platform-filtered skips and no retries or flaky
classification. The exact JUnit is
`repair-core-final-macos-junit.xml.gz` (SHA-256
`9eb1c35509c4cd4480f8491ed218b2b59a8e765d39c8fd71fdb8f7381f1f1a7e`).
The original failure evidence and reasoning remain below rather than being
rewritten as a pass.

Candidate Rust tree: `a7ae94e4c9c01924c896d9f10b1f588f1727fc67`, macOS arm64,
Rust 1.95.0, nextest 0.9.143, default features, no test exclusions added.
Command: `just test -p codex-core --test-threads 4`, with LLVM lld,
`CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`.
All companion binaries (`codex`, `corbanu`, `codex-code-mode-host`,
`test_stdio_server`) were built before running. Their presence is prerequisite
repair, not a change to assertions or a waiver of the failing suite.

Run `694ae0c9-b530-44df-99f5-853ed5f7f3ad`, 2026-08-28 06:22:54 UTC,
225.692 seconds: **3,407 executed, 3,388 passed, 19 failed**. The runner also
reported 19 pre-existing skipped tests outside execution. Unit binary: 2,335
tests / 12 failures; integration `all`: 1,068 / 7; response headers: 4 / 0.
Every one of the 13 credential-named tests passed, including the newly
secret-bearing callback panic in the unique-canary test.

Artifact: `repair-core-macos-junit.xml.gz`, SHA-256
`5a2dc969824f0df5773c419cd93e81c7e1a304d9989b9fee14bf8cdfa51f9b09`.
Compared with the historical 135-failure report, 117 former failing tests now
pass, 18 repeat, and one additional prompt-caching test fails. This comparison
does not attribute every improvement solely to companion binaries; the earlier
report predates merged source changes as well.

## Remaining failure groups

| Group | Tests | Evidence and next action |
| --- | --- | --- |
| Mac Bash compatibility | `bash_snapshot_filters_secret_and_invalid_exports`, `bash_snapshot_preserves_multiline_exports` | Confirmed production script uses `${name^^}` but `/bin/bash` is 3.2.57. An isolated, secret-free invocation reproduces `bad substitution`. A portable uppercase conversion must retain the secret-export exclusion checks; installing another Bash does not fix support for the system shell. |
| Agent lifecycle / authority / delivery | `resume_agent_from_rollout_reads_archived_rollout_path`, `crew_child_terminal_result_uses_one_triggering_native_mailbox_message`, `direct_spawn_troll_can_followup_task_two_named_orc_children`, `spawn_agent_allows_depth_up_to_configured_max_depth`, `cold_root_resume_restores_agent_identity_and_role_on_followup` | Duplicate/cyclic actor chain, absent live-parent security binding, mailbox/follow-up expectation mismatch and worker timeout. Needs native-agent/authority tracing and a separate execution mandate before changing lifecycle or policy composition. |
| MCP authority refresh | `user_turn_updates_approvals_reviewer` | Fails the required MCP-state refresh assertion when elicitation authority changes. Preserve the authority-refresh requirement while diagnosing the runtime transition. |
| Stale encrypted-message wording | `encrypted_openai_message_requires_plaintext_adapter_for_external_target` | Runtime correctly denies cross-provider encrypted payload before admission, but the exact expected guidance text is stale. A focused expectation update must retain the denial/no-turn-start assertions. |
| Tool registration / approval contracts | `environment_count_controls_environment_backed_tools`, `environment_tools_follow_the_step_context`, `multi_agent_feature_selects_one_agent_tool_family`, `hosted_web_search_and_standalone_image_generation_follow_runtime_gates`, `remote_model_override_uses_catalog_model_for_strict_auto_review`, `code_mode_holds_yielded_result_during_patch_approval` | Missing expected tool/parameter, missing Guardian apply-patch request, or patch-approval timeout. Establish the intended native tool/model gates before modifying fixtures or registration. Do not re-enable a tool just to satisfy a stale test. |
| Prompt/serialization expectations | `skills_use_aliases_in_developer_message_under_budget_pressure`, `multiple_auto_compact_per_task_runs_after_token_limit_hit` | Alias budget assertion and an extra `content: null` in reasoning serialization. Compare current model-visible contracts before updating expectations. |
| Prompt-cache context transition | `overrides_turn_context_but_keeps_cached_prefix_and_key_constant` | One expected request was absent in the full run. An isolated rerun passed in 0.316 seconds (run `71836555-03e0-4433-bd60-dac76e371afc`; `repair-core-cache-isolated-junit.xml`). Treat as intermittent pending transition/concurrency tracing, not a repaired full-suite failure. |
| Shell parallelism | `shell_tools_start_before_response_completed_when_stream_delayed` | Deadline exceeded. Verify event ordering and sandbox/executor startup before adjusting timing. |

These failures lie outside the accepted credential callback/header/harness
repair boundaries. No assertion was weakened or unrelated runtime change made.
This is triage, not a clean-Core result or permission to archive PF-13-S05.
The final integration/release gate remains blocked until scoped follow-up
repairs pass a fresh complete run. Native PF-23 composition remains separate.
