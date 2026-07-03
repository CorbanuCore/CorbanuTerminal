# PfTerminal Phase A Adversarial Code Review - 2026-07-03

Directive: `/home/pfrpc/repos/orc_directives/gorkul_directive_qa_review_0325.md`
Parent mandate: `/home/pfrpc/repos/orc_directives/overnight_qa_release_mandate_20260703.md`
Evidence dir: `/home/pfrpc/repos/pfterminal_qa_20260703/`
Review target: `/home/pfrpc/repos/PfTerminal-bench`
Reviewed HEAD: `d86721e33` (`origin/main`)
Reviewed commits over base `8efcb8e46`: `ff1c90e43` (#18), `f26d0e314` (#19), `d86721e33` (#20)

No fixes made. No merges made. No API spend.

## Coordination

The functional TUI QA worker wrote `/home/pfrpc/repos/pfterminal_qa_20260703/tui_qa_findings.md`. This report does not duplicate that test pass; it focuses on code review, local test gates, and failure mechanisms. The TUI report independently reproduces live dispatch/report failures, which corroborate the dispatch risks below.

## Findings Table

| ID | Sev | Area | File:line | Scenario | Repro/Test | Gate |
| --- | --- | --- | --- | --- | --- | --- |
| R1 | P1 | OpenRouter stream hardening | `codex-rs/model-provider-info/src/lib.rs:28`, `:29`; `codex-rs/codex-api/src/endpoint/chat_completions.rs:1018`, `:1038`, `:1160`, `:1197`, `:1618` | OpenRouter comment-only keepalives reset the byte idle timer, but still fail at the shorter 180s actionable-silence timer before the 600s transport idle timeout. This preserves the original long-think failure class under a different timeout. | Existing passing test `comment_only_stream_hits_actionable_silence_timeout` asserts this failure mode. | Blocker |
| R2 | P1 | Spawn report delivery | `codex-rs/tui/src/spawn_orchestration.rs:1489`, `:1502`, `:1517`; `codex-rs/tui/src/app/event_dispatch.rs:2338`, `:2367`, `:2393`, `:2434`, `:2447`, `:2485`; `codex-rs/tui/src/app_event_sender.rs:32` | Pending child reports are drained from memory before the parent `turn_start` is accepted. Any attach/read/auth/turn_start/channel failure after that point drops the report and leaves only a UI error/log. | Existing tests assert event emission, not retention/retry after rejected `SubmitSpawnAgentTask`. Add a Phase B repro with app-server returning active-turn/session/auth errors after queue drain. | Blocker |
| R3 | P1 | TUI release gate | `codex-rs/tui/src/app_server_session.rs:115`, `:175`; `codex-rs/tui/tests/manager_dependency_regression.rs:49` | `codex-tui` test suite is red on main. One deterministic failure flags `AuthManager` usage in TUI after `AuthManagerConfig` was imported into `app_server_session.rs`; many snapshot/model-picker/status tests also fail. | `just test -p codex-tui`: `3117 tests run: 3095 passed, 22 failed, 9 skipped`. | Blocker |
| R4 | P1 | `ModelProviderInfo` blast radius | `codex-rs/login/src/auth_env_telemetry.rs:61`; `codex-rs/app-server/src/request_processors/thread_processor_tests.rs:670` | #20 added `ModelProviderInfo.chat_completions_provider`, but test initializers in downstream crates were not updated, so touched crates fail to compile. | `just test -p codex-login` fails with E0063; `just test -p codex-app-server` fails with E0063. | Blocker |
| R5 | P1 | App-server protocol schema gate | `codex-rs/app-server-protocol/src/protocol/v2/thread.rs:55`, `:150`, `:164`; `codex-rs/app-server-protocol/src/protocol/v1.rs:196`, `:200` | Schema export tests fail on a duplicate `ThreadStartParams` definition through `ThreadSpawnAgentParams.thread`; TS fixtures also miss new `GetAuthStatusResponse.hasCodexBackendAuth`. | `just test -p codex-app-server-protocol`: `245 tests run: 241 passed, 4 failed`. | Blocker |
| R6 | P1 | Core release gate | `codex-rs/core/src/agent/role_tests.rs:234`; `codex-rs/core/src/config/config_tests.rs:1323`; `codex-rs/core/src/tools/router_tests.rs:389`; `codex-rs/core/src/tools/spec_plan_tests.rs:135` | Full core suite is massively red after the merged diff set, with failures in role application, config validation, tool routing/spec-plan exposure, guardian assessment, shell/unified-exec, and snapshot suites. Some may be environmental, but this cannot pass release until triaged. | `just test -p codex-core`: `2877 tests, 246 failures`. | Blocker |

## Detail

### R1: Comment keepalives still do not make OpenRouter long-think safe

#20 moved the idle timeout toward the transport layer: `byte_idle_timeout_stream` records byte activity and comment frames (`chat_completions.rs:1197-1217`). That fixes the narrow "no parsed data event" idle reset bug.

The remaining problem is the added actionable-silence timer. Defaults are 600s for byte idle and 180s for actionable silence (`model-provider-info/src/lib.rs:28-29`). Chat-completions options pass the provider actionable timeout through (`core/src/client.rs:1661-1668`). In `process_chat_sse`, the first activity, including a comment-frame byte, starts `actionable_deadline_at` (`chat_completions.rs:1018-1035`). `poll_chat_sse_event` then races that deadline against both byte activity and parsed events (`chat_completions.rs:1160-1180`), and a deadline win emits `ApiError::Stream` (`chat_completions.rs:1038-1051`).

The regression test currently encodes the bad production shape: `comment_only_stream_hits_actionable_silence_timeout` sends repeated `: OPENROUTER PROCESSING` comments and asserts an actionable-silence error (`chat_completions.rs:1618-1658`). For the bench failure class, comments from OpenRouter prove the socket is alive while the upstream model is thinking. Failing at 180s still causes a long turn to abort/retry before the 600s idle window and before a valid late content/reasoning delta.

Phase B should invert or scope this test: comment-only keepalives should not trigger a reconnect/fail before the configured transport idle policy for OpenRouter long-think, unless product policy intentionally sets a much longer user-visible "no useful output" cap that does not silently retry the same request.

### R2: Pending child reports can still be lost after queue drain

`flush_pending_reports_for_thread` drains `spawn_pending_reports_by_thread` (`spawn_orchestration.rs:1489-1502`) and immediately sends `AppEvent::SubmitSpawnAgentTask` (`spawn_orchestration.rs:1516-1520`). The event sender swallows channel-send failures after logging (`app_event_sender.rs:32-42`).

The later handler has multiple failure exits before and during `turn_start`: attach-live-thread failure (`event_dispatch.rs:2356-2372`), `thread_read` plus materialization failure (`event_dispatch.rs:2375-2421`), missing session (`event_dispatch.rs:2426-2433`), provider auth failure (`event_dispatch.rs:2434-2440`), and `turn_start` error (`event_dispatch.rs:2447-2488`). None of those paths requeue the drained report or mark durable delivery failure.

This is separate from, but compatible with, the TUI QA report's live finding that child results were visible in spawn status but not cleanly delivered to Main. The code path still treats "event submitted" as "report delivered"; delivery is not acknowledged at the parent turn boundary.

### R3: TUI tests are red on main

`just test -p codex-tui` failed with `3117 tests run: 3095 passed, 22 failed, 9 skipped`.

Representative failures:

- `manager_dependency_regression tui_runtime_source_does_not_depend_on_manager_escape_hatches` fails at `tui/tests/manager_dependency_regression.rs:49`, reporting `app_server_session.rs contains AuthManager`. The direct new anchor is `use codex_login::AuthManagerConfig` at `app_server_session.rs:115` and use at `:175`.
- `chatwidget::model_popups::tests::model_provider_for_selection_maps_cross_provider_models` fails at `tui/src/chatwidget/model_popups.rs:835`, with `Some("ambient")` vs `Some("openrouter-anthropic")`.
- Numerous snapshot/status/model-picker/side-context/review-mode tests emit `.snap.new` output. Those generated files were deleted after the run so the checkout is clean.

### R4: `ModelProviderInfo` change breaks downstream test builds

`just test -p codex-login` does not compile:

`login/src/auth_env_telemetry.rs:61:24`: missing field `chat_completions_provider` in initializer of `ModelProviderInfo`.

`just test -p codex-app-server` does not compile:

`app-server/src/request_processors/thread_processor_tests.rs:670:32`: missing field `chat_completions_provider` in initializer of `ModelProviderInfo`.

This is direct #20 blast radius from adding a required field without updating all in-tree construction sites.

### R5: App-server protocol schema gate is red

`just test -p codex-app-server-protocol` failed with 4 failures:

- `export::tests::generate_json_filters_experimental_fields_and_methods`
- `export::tests::generate_json_includes_remote_control_methods_with_experimental_api`
- `schema_fixtures json_schema_fixtures_match_generated`
- `schema_fixtures typescript_schema_fixtures_match_generated`

The JSON failures report:

`schema definition collision in namespace v2: ThreadStartParams (existing title: ThreadStartParams, new title: <untitled>)`.

The relevant nested shape is `ThreadSpawnAgentParams.thread: ThreadStartParams` (`protocol/v2/thread.rs:150-164`) alongside the top-level `ThreadStartParams` export (`protocol/v2/thread.rs:55`). The TS fixture failure shows `GetAuthStatusResponse` now includes `hasCodexBackendAuth` (`protocol/v1.rs:196-200`) but the vendored generated fixture was not updated.

### R6: Core suite is red

`just test -p codex-core` completed with `2877 tests, 246 failures`.

Representative failing clusters:

- Role application tests in `core/src/agent/role_tests.rs`.
- Config validation around provider overrides in `core/src/config/config_tests.rs`.
- Tool router/spec-plan exposure tests in `core/src/tools/router_tests.rs` and `core/src/tools/spec_plan_tests.rs`.
- Guardian assessment and broad shell/unified-exec/snapshot failures.

This needs triage before release. The run generated two `.snap.new` files; they were deleted after the run.

## Passing Local Checks

| Command | Result |
| --- | --- |
| `python3 scripts/test_spawn_report_verify.py` | PASS, 8 tests |
| `just test -p codex-api` | PASS, 149 tests |
| `just test -p codex-model-provider-info` | PASS, 40 tests |
| `just test -p codex-config` | PASS, 189 tests |

## Failing Local Checks

| Command | Result |
| --- | --- |
| `just test -p codex-tui` | FAIL, 22 failed |
| `just test -p codex-login` | FAIL, compile E0063 |
| `just test -p codex-core` | FAIL, 246 failed |
| `just test -p codex-app-server-protocol` | FAIL, 4 failed |
| `just test -p codex-app-server` | FAIL, compile E0063 |

## Quality Gate Blockers

P0: none found in code review.

P1 blockers: R1, R2, R3, R4, R5, R6.

Do not release or merge further dependent work until these are resolved or explicitly downgraded by the release owner with written rationale.

## Evidence Hygiene

- PfTerminal checkout was returned to a clean `main` after generated `.snap.new` artifacts were removed.
- Exact-key scan over the evidence directory: no vendor key or bearer-token patterns found after this report was written.
