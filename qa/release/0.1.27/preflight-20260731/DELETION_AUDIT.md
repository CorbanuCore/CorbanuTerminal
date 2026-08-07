# PF Terminal 0.1.27 deletion audit

The path-level inventory is `DISPOSITION.csv`. This audit explains every
deleted-file family so deletion is never treated as proof that a released
feature was intentionally retired.

## Core runtime

| Deleted family | Disposition | Retained/replacement boundary |
| --- | --- | --- |
| `core/src/agent/builtins/{nazgul,orc,troll,standard}*` | Replace | The hard-coded fantasy-role stack is replaced by upstream agent roles plus the typed PF model catalogue. Selection is based on route eligibility, capability, vision, billing, and task fit rather than role-name model constants. |
| `core/src/agent/control/mailbox.rs` | Replace | Durable admission, deduplication, attempts, completion, and recovery live in `state/src/runtime/agent_mailbox.rs`; current-turn delivery lives in `core/src/state/turn.rs`, `core/src/codex_thread.rs`, `core/src/tasks/`, and `core/src/stream_events_utils.rs`. |
| `context/subagent_turn_budget_finalization.rs`, `context/turn_completion_continuation.rs`, and `session/turn_completion*` | Replace | Current upstream turn lifecycle and provider completion semantics live in `session/turn.rs`, `tasks/`, and `stream_events_utils.rs`. PF-specific regex completion, output caps, and hard continuation limits are deliberately removed. |
| `session/turn_context_tests.rs` | Relocate | Turn-context behavior is covered beside the current session, route-lock, model-switching, compaction, and orchestration implementations. |
| `tools/handlers/structured_edit.rs` | Replace | Native and wrapped edits use `core/src/apply_patch.rs`, `tools/handlers/apply_patch.rs`, and `tools/runtimes/apply_patch.rs`, with hook and permission coverage. |
| `tests/suite/anthropic_payload_limit.rs` | Relocate | Payload projection, image preservation, bounded 413 recovery, signed thinking, and output behavior are covered in current Anthropic/client/compaction suites. |
| `tests/suite/chat_provider_turn_lifecycle.rs` | Relocate | Provider-neutral Chat continuation and Kimi/GLM reasoning behavior are covered in API adapter and current core client suites. |
| `tests/suite/vercel_server_state.rs` | Relocate | Vercel route pinning and server-state behavior are covered through the shared Responses/gateway lifecycle tests. |

## TUI

| Deleted family | Disposition | Retained/replacement boundary |
| --- | --- | --- |
| `claude_panes/**`, `external_plan_agent_adapter.rs` | Replace | Plan models are ordinary typed provider routes. The TUI no longer maintains a second external-manager control plane that can collide with native Codex orchestration. |
| `crew_*`, `spawn_crew.rs`, `custom_spawn_crew.rs`, `spawn_orchestration.rs`, `orchestrate.rs`, `dispatch_queue*` | Replace | `/agent`, the agent picker/navigation layer, and core collaboration tools own one agent graph and mailbox. Explicit route requests are catalogue-validated and persisted. |
| old model/spawn popup snapshots and `model_catalog_tests.rs` | Replace | Provider/model picker, transactional switching, model-catalog, and agent-picker coverage live beside the current app/chatwidget implementation and generated snapshots. |
| `app/tests/dispatch_integration.rs` | Relocate | Dispatch/session behavior is exercised through current session-lifecycle, event-dispatch, thread-routing, and orchestration integration coverage. |
| `tps.rs` | Retire display implementation only | Route, elapsed time, token accounting, and settled cost remain product telemetry. The deleted standalone TPS widget is not a routing or accounting source of truth. |
| `mkdocs_overlay.rs`, `mkdocs_viewer*` | Remove unreleased overlay | This viewer is outside the mandatory released PF command surface and does not own session/provider state. Repository documentation remains available as files. |
| `tests/spawn_report_delivery_scope.md` | Replace | Direct-parent, single-terminal-result, mailbox-envelope, close/resume, and no-unsolicited-inference behavior is enforced in executable core tests rather than a prose-only fixture. |

## Release decision

No mandatory PF feature is intentionally removed. `/providers`, `/vault`,
`/wallet`, `/model`, `/agent`, `/goal`, `/gpu`, Telegram, multi-provider
routing, model-aware orchestration, local resumable state, and accounting must
all pass against the packaged RC. Any missing surface is a release blocker,
regardless of this audit.
