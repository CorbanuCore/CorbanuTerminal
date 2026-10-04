# Source findings and candidate comparison

All observations below are from source at the recorded base, not the running package. The parent reports `/permissions` accepts Full Access with `approval: never` and a disabled profile, followed by escalation on the same running turn. That live reproduction was not repeated here.

## Observed control flow

| Source / symbol | Observation |
| --- | --- |
| `codex-rs/tui/src/app/config_persistence.rs::apply_permission_profile_selection` (173–285) | Rebuilds and validates a candidate, then changes app/widget state, runtime overrides and cached session before dispatch; queues “Permissions updated” immediately. |
| `codex-rs/tui/src/app/thread_settings.rs::send_thread_settings_update` (175) | Reports RPC submission errors; does not itself wait for authoritative effective-state confirmation. |
| `codex-rs/app-server/src/request_processors/turn_processor.rs::thread_settings_update_inner` (890) | Submits `Op::ThreadSettings`, then returns an empty response. Submission success is not proof the settings have become effective. |
| `codex-rs/core/src/session/handlers.rs::update_thread_settings` (133) | Calls `update_settings`, then emits `ThreadSettingsApplied` from the current session snapshot. Error path emits BadRequest. |
| `codex-rs/core/src/session/session.rs::SessionConfiguration::apply` (247) | Builds a clone and validates constrained fields. Preserve this validation and all-or-nothing behavior. |
| `codex-rs/core/src/session/mod.rs::update_settings` (1601) | Publishes new session configuration under its state mutex; outside that lock refreshes shared network policy and schedules MCP prewarm. No replacement of the current turn snapshot. |
| `codex-rs/core/src/session/turn_context.rs::make_turn_context` (753–804) | Stores an `Arc<Config>`, cloned approval policy, effective permission profile, Windows sandbox level and network handle in the turn. |
| `codex-rs/core/src/unified_exec/process_manager.rs` (1197, 1250) | Approval and orchestration read `context.turn`, not the newly updated session settings. |
| `codex-rs/core/src/tools/orchestrator.rs::run` (143 onward) | Sandbox profile comes from turn config; approval can suspend before execution/retry. Changing only one permission field cannot make this coherent. |
| `codex-rs/core/src/session/turn_context.rs::new_turn_with_sub_id` (824) | A second settings mutation path used for user-input overrides. It must obey the same boundary; fixing only `Op::ThreadSettings` is insufficient. |
| `codex-rs/core/src/tasks/mod.rs::spawn_task`, `start_task_with_execution_guard`, `abort_all_tasks` | Context creation, task admission, removal and cancellation cleanup are separate operations. An empty `active_turn` observation alone cannot prove quiescence. |
| `codex-rs/core/src/unified_exec/process_manager.rs` (472 onward; 1471 onward) | Stores live processes before yielding specifically so turn interruption does not terminate them. Process reservations/startup and retained background terminals must participate in the boundary. |
| `codex-rs/core/src/session/tests.rs::refreshed_mcp_binding_captures_current_approval_authority` | Existing test explicitly expects updated MCP authority and unchanged old-turn approval policy. Source evidence, not a test run. |
| `codex-rs/core/src/session/tests.rs::mcp_elicitation_reviewer_uses_latest_runtime_authority` | Existing test changes authority during an active task. A boundary correction will require deliberate compatibility treatment, not deleting or weakening this test to get green. |

The operation dispatcher serializes submitted operations, but tools/tasks and refresh callbacks are asynchronous. Serialization of settings submissions alone does not cover tool startup, old contexts waiting for admission, teardown, background processes, or publication ordering. `thread_settings_applied_event` takes a fresh snapshot after the mutation call; a robust acknowledgement must identify the exact committed request/revision rather than an unrelated later update.

## Candidate assessment

1. UI wording only: lowest footprint, useful honesty repair, but does not fix stale execution authority or mixed shared-service state. Not selected as a complete correction.
2. Fetch current approval policy at each exec: rejected. Splits approval from sandbox/network/config, can convert a pending decision into unintended execution, and leaves other tools and in-flight races unresolved.
3. Hot-swap all active authority with generations/revocation: possible larger initiative, but requires admission fences, post-approval revalidation, process/channel revocation semantics and cross-tool coverage. Not the smallest initial correction.
4. Explicit quiescent application boundary with atomic backend refusal while busy: recommended for allocation. Both authority increases and decreases use the same rule. No automatic replay, implicit approval, process kill, or turn restart. UI exposes an attempted change as pending only until an authoritative result; a refusal says the existing level remains effective and instructs the user to finish or explicitly stop relevant work, then retry.

Candidate 4 still needs implementation proof that all holders of old authority are accounted for. If a complete boundary cannot be established within the allocated files/size, stop and return that evidence. Do not ship a TUI-only busy flag or substitute a new turn for confirmed process/tool quiescence.

## Reproduction to carry into isolated regression work

The parent-reported sequence is: start a restricted turn, select Full Access, observe settings-applied, then attempt another escalation-sensitive command in that original turn. Source predicts the old turn approval remains in use. The symmetric decrease is more safety-critical: a session selection can become restrictive while an old context still carries broader authority.

No new executable reproduction has been run. After allocation, implementer-owned synthetic regression tests should use native Core/test support and synchronization barriers to exercise that sequence, both mutation paths, queued admission, outstanding approval resolution, startup reservations, background terminals, cancelled/failed updates and delayed acknowledgements. These tests support the frozen independent F01–F11; they do not replace those expectations or acceptance execution.
