# Multi-Model Native Orchestration Implementation Journal

Mandate: `/home/pfrpc/repos/pfterminal_multimodel_native_orchestration_spec_20260725.md`  
Branch: `feat/multimodel-native-orchestration`  
Base: `origin/main` at `08a8d9367`

## Phase 0 — Baseline

- Reused the clean `PfTerminal-telegram-hardening` worktree; no new large worktree was created.
- Reclaimed 115.4 GiB with `cargo clean --profile dev`.
- Free disk after cleanup: 164 GiB.
- Existing V2 app-server `thread/spawnAgent` already carries explicit `ThreadStartParams.model_provider` and `model`.
- Model-facing V2 `spawn_agent` exposed `model` but not `model_provider`, so it could not request an exact cross-provider runtime.
- The TUI standard crew already starts provider-specific native app-server threads, but persistent crew configuration and dispatch remain TUI-owned.

## Phase 1 — Explicit native runtime

Status: implemented; scoped tests pass.

Changes:

- V2 `spawn_agent` accepts `model_provider` together with `model`.
- Explicit runtime pairs are validated against configured providers and provider/model compatibility before child creation.
- Role configuration is applied before the explicit runtime, so a role cannot silently replace a caller's exact provider/model selection.
- Full-history forks reject provider overrides alongside role/model/effort overrides.
- V1 retains its original model-only contract and guidance.

Required runtime pair coverage:

- `claude-plan` / `claude-opus-5-plan`
- `claude-plan` / `claude-fable-5-plan`
- `openrouter` / `x-ai/grok-4.5`
- `kimi-code` / `k3`

Passing evidence:

- `spawn_agent_explicit_runtime_supports_required_multimodel_pairs`
- `spawn_agent_explicit_runtime_rejects_incomplete_or_invalid_pairs`
- `multi_agent_v2_spawn_explicit_runtime_wins_over_role_runtime`
- `spawn_agent_tool_v2_requires_task_name_and_lists_visible_models`
- `spawn_agent_tool_v1_keeps_legacy_fork_context_field`
- `multi_agent_v2_spawn_defaults_to_full_fork_and_rejects_child_model_overrides`
- `just fix -p codex-core`

Broader baseline:

`just test -p codex-core multi_agents` ran 108 tests: 97 passed and 11 failed. The new runtime tests all passed. The failures are retained as unresolved evidence, not called green:

- role/depth fixture failures in legacy and V2 hierarchy tests;
- encrypted task-preview expectation failures;
- follow-up encrypted-payload expectation failures; and
- shared thread-manager lifecycle failures.

These overlap the control-plane and role-graph work in later phases and must be reconciled before final qualification.

## Phase 2A — Versioned crew definition

Status: behavior-preserving extraction implemented; persistence gate remains open.

Changes:

- Added versioned `CrewSpec`, `CrewMemberSpec`, `CrewPolicy`, `RuntimeRequest`,
  `AgentRuntimeSpec`, `AgentClass`, and retention/transport types in `codex-protocol`.
- Added validation for schema version, stable member IDs, one root, topological parentage,
  exact runtime completeness, selectors, and provider policy.
- Expressed the existing standard Nazgul/Troll/three-Orc crew as `standard-v1` data.
- Changed `/spawn` standard-crew creation to iterate that validated definition. The provider,
  model, reasoning effort, hierarchy, nickname assignment, and pane behavior remain unchanged.
- Kept an explicit test fixture covering Claude Plan Fable 5, Claude Plan Opus 5,
  OpenRouter Grok 4.5, and Kimi Code K3.

Passing evidence:

- `just test -p codex-protocol crew`: 2 passed.
- `just test -p codex-tui crew_presets`: 2 passed.
- `standard_crew_quick_start_uses_the_expected_role_picker_label`: passed.

Open gate:

- The existing pane layout persists native thread IDs, parent edges, and resolved runtimes, but
  it does not yet persist `CrewSpec` or stable logical-member mappings. Phase 2 is therefore not
  complete until restart tests prove the same crew identity and runtime mapping.

### Phase 2B — Durable crew identity

Status: durable metadata implemented; live restart reconciliation remains for Phase 5.

Changes:

- Added a persisted `CrewInstanceState` containing the versioned spec, stable logical-member to
  native-node mapping, and explicit `creating`, `ready`, or `incomplete` state.
- `/spawn` now writes the crew intent before starting the first native thread and checkpoints each
  successful member mapping.
- Re-entering creation reuses only an identical crew definition and validates the existing native
  endpoint, parent edge, provider, model, and effort. It stops instead of duplicating, reparenting,
  or silently changing a runtime.
- A provider/start failure leaves an incomplete durable intent for recovery.
- Pane-layout persistence now round-trips the crew state alongside the existing native thread
  edges, endpoints, and runtime records.

Passing evidence:

- `crew_instance_round_trip_preserves_logical_to_native_identity` covers both the standard crew
  and a custom Fable/Opus/Grok/Kimi crew.
- `crew_instance_rejects_identity_reassignment_and_incomplete_ready_state`.
- `pane_layout_persistence_round_trips_root_binding_and_parent_map` now asserts full crew-state
  equality after disk persistence.
- Standard `/spawn` quick-start regression remains green.

Remaining distinction:

- This proves durable crew metadata and idempotent creation decisions. Kill-and-resume against
  live native threads, strict stale-layout rejection, and legacy read-only fallback remain Phase 5
  requirements and are not claimed complete here.
## Phase 3A — Target-runtime-safe native reload

- Fixed `AgentControl::ensure_v2_agent_loaded` so an unloaded agent is resumed with the provider,
  model, and reasoning effort stored on that target thread. The caller's current runtime is used
  only as the base configuration and can no longer overwrite a heterogeneous target.
- Added a regression that spawns an OpenRouter Grok 4.5 child, unloads it, asks an Ambient parent
  configuration to reload it, and proves the restored tuple remains
  `openrouter / x-ai/grok-4.5`.
- Verification:
  - `just fmt`
  - `just test -p codex-core ensure_v2_agent_loaded_reloads_registered_unloaded_agent`
    — 1 passed.

## Phase 3B — Canonical durable native mailbox

Status: native-capable delivery path implemented and qualified; the external-plan adapter and
legacy TUI queue cutover remain Phase 4/6 work.

Commits:

- `969dd0755 feat: add durable native agent mailbox state`
- `ddaf1ff9c feat: route native agents through durable mailbox`

Changes:

- Added stable message IDs, optional assignment IDs, typed message kinds, timestamps, and one
  32 KiB provider-neutral body bound to `InterAgentCommunication`.
- Added SQLite mailbox admission and explicit lifecycle states:
  `admitted`, `ready`, `submitting`, `submitted`, `provider_running`, `retryable_failure`,
  `unknown_outcome`, `applied`, `cancelled`, and `terminal_failure`.
- Admission is idempotent by stable message ID and rejects reuse of an ID for different content
  or recipients.
- Native V2 spawn assignments, messages, follow-ups, control requests, and terminal results now
  use one plaintext provider-neutral envelope. OpenAI encrypted tool arguments are no longer the
  V2 internal transport because other providers cannot consume that opaque value.
- Initial V2 spawn assignments now enter the same durable mailbox as later follow-ups. They no
  longer bypass admission or reject before queueing solely because all worker turns are occupied.
- Active-turn capacity uses an atomic reservation. Saturated mailbox work remains accepted and is
  woken by capacity release rather than a retry timer.
- Unloaded heterogeneous targets resume with their own persisted provider/model/effort, never the
  sender's runtime.
- Restart recovery requeues deterministically safe local states and quarantines
  `submitting`/`provider_running` as `unknown_outcome`; those states are not automatically replayed.
- Local rollout application is deduplicated by stable message ID, including the crash seam between
  rollout flush and mailbox acknowledgement.
- Terminal result IDs are deterministic per child thread and turn, so repeated transport applies
  once locally.
- Status previews omit full task bodies; credentials and full task text are not added to logs.

Passing evidence:

- `just test -p codex-protocol`: 243 passed.
- `just test -p codex-state`: 175 passed.
- `durable_agent_mailbox_deduplicates_and_marks_applied_after_rollout_flush`.
- `ensure_v2_agent_loaded_reloads_registered_unloaded_agent`.
- `direct_spawn_troll_can_followup_task_two_named_orc_children`.
- `capacity_waiter_unblocks_after_atomic_worker_reservation_is_released`.
- `message_content_rejects_empty_and_oversized_messages`.
- `provider_neutral_multi_agent_v2_spawn_sends_agent_message_to_child`: 12 consecutive
  no-retry passes after replacing a substring matcher that could capture unrelated requests.
- `just fix -p codex-protocol`, `just fix -p codex-state`, and `just fix -p codex-core`.

Broader V2 result:

- `just test -p codex-core multi_agent_v2`: 71 passed, 3 failed.
- The remaining failures are unchanged baseline fixtures: configured thread-cap expectation
  `3` versus current default `5`, a Troll fixture attempting to spawn a non-Orc, and a depth-2
  fixture whose parent is depth 0. They are retained as explicit Phase 7 role/depth cleanup work;
  no mailbox or provider-neutral delivery test remains failing.

## Phase 4A — External-plan compatibility boundary

Status: compatibility decoder isolated; `/spawn` native-delivery cutover remains in progress.

Changes:

- Moved Claude Plan and other text-only runtime envelope decoding out of the 12k-line
  `spawn_orchestration.rs` into the focused 290-line `external_plan_agent_adapter.rs`.
- Kept the text envelopes strictly at the provider edge. Native agents and the canonical mailbox
  do not use XML-ish or fenced transport internally.
- Decode order now matches source order across mixed legacy XML-ish and fenced messages.
- Enforced the canonical 32 KiB agent-message bound before edge messages can enter routing.
- Malformed and oversized envelopes remain visible to the operator and carry an explicit decode
  failure. They are never silently stripped or treated as successful dispatches.
- Preserved the existing extraction API temporarily so the `/spawn` pane UX and routing behavior
  remain unchanged while native delivery is cut over beneath it.

Passing evidence:

- `external_plan_agent_adapter`: 2 passed.
- `spawn_task_dispatch`: 4 passed.
- `cargo check -p codex-tui`: passed.

Product boundary:

- PfTerminal `/spawn` remains the persistent named mixed-model crew product.
- Codex-native agents provide its lifecycle, identity, capacity, and message substrate.
- The standard `/spawn` crew, pane navigation, provider choices, resumable identities, and role
  presentation are not replaced by Codex's ephemeral OpenAI-only delegation UX.

## Phase 4B — `/spawn` native mailbox cutover

Status: native `/spawn` delivery cut over; legacy Claude-pane delivery remains behind the
compatibility adapter.

Changes:

- Kept `/spawn` as the persistent named mixed-model crew and pane UX.
- Replaced the native TUI `turn/start` versus `turn/steer` delivery decision with
  `thread/sendAgentMessage`, backed by the canonical durable native mailbox.
- Native targets accept durable work while already running or blocked in `wait_agent`; the mailbox
  applies it at a valid model boundary without consuming another TUI execution slot.
- TUI crash reconciliation now resends the same stable mailbox ID. Canonical admission
  deduplicates it; native delivery no longer scans thread history for an inferred client-message
  witness.
- Froze the current `/spawn` roster context into the durable task before stable-ID assignment, so
  retries remain byte-identical while Trolls retain their named Orc roster and report context.
- Deleted the obsolete native steer/start/reconciliation events, workers, fault flags, and helper
  paths: 676 lines removed versus 236 added across the cutover and strengthened tests.
- Preserved the separate Claude compatibility-pane pump. It is not native internal transport and
  remains quarantined on uncertain outcomes.

Passing evidence:

- Real event-path mailbox integration: 6/6 passed.
  - arbitrary task FIFO across valid mailbox coalescing;
  - completed-source replay does not re-enqueue;
  - three active child turns enforce capacity and release one follow-up;
  - a waiting Troll wakes through the mailbox without `turn/start` fallback;
  - oversized tasks reject before provider submission;
  - compaction preserves subsequent dispatch.
- `spawn_orchestration`: 26/26 passed.
- Seeded dispatch qualification: passed.
- Standard `/spawn` quick-start regression: passed.
- App-server stable-ID mailbox integration: passed.
- `cargo check -p codex-tui`: passed without new warnings.

## Phase 7A — Role profiles are not core authorization

Status: role-specific core depth and capacity rules removed; `/spawn` crew shape remains data in
the versioned `CrewSpec`.

Changes:

- Removed the Nazgul/Troll/Orc role graph validator from model-facing spawn, direct app-server
  spawn, and the central native spawn boundary.
- Kept the provider-neutral structural depth invariant: a child still must be exactly one native
  level below its actual parent and remain within `agents.max_depth`.
- Removed the special Nazgul execution-slot bypass. A role label can no longer forge control-plane
  capacity. Mailbox admission remains available under saturation; execution is scheduled by the
  same native limiter as every other sub-agent.
- Kept the named role profiles and their instructions for `/spawn` presentation. This change does
  not remove the standard crew, pane navigation, or mixed-model runtime selection.

Passing evidence:

- `execution_guards_do_not_derive_capacity_policy_from_role_names`.
- `spawn_agent_internal_treats_roles_as_profiles_and_enforces_structural_depth` with
  `RUST_MIN_STACK=33554432` (the repository's known test-harness stack requirement).
- `cargo check -p codex-core -p codex-app-server`.

## Phase 5/6 — Strict recovery and direct `/spawn` mailbox admission

Status: new native crews use the canonical mailbox directly; legacy identities fail closed.

Changes:

- Native `/spawn` assignments no longer enter the TUI dispatch queue or pump. The TUI derives one
  stable message/assignment ID, calls V2 `thread/sendAgentMessage`, and reports “queued” only after
  the canonical mailbox durably acknowledges admission.
- A native admission failure is visible and is not automatically replayed. The stable source
  origin is recorded only after canonical admission, so a failed request is not falsely tombstoned.
- The legacy pump can select only external Claude compatibility panes. It has no native delivery
  variant and cannot become a second native retry path.
- Removed obsolete native-pump qualification tests and replaced their assertions with real
  app-server/mailbox event-path coverage.
- Restored pre-CrewSpec hierarchies are inspectable but read-only. They cannot dispatch, replay,
  or silently enter the new control plane.
- Modern restored crews are validated against `CrewSpec`: every logical member must resolve to one
  available native thread, the parent edge must match, and provider/model/effort must equal the
  persisted exact runtime. Drift marks the crew incomplete and blocks mutation.

Passing evidence:

- Real dispatch integration: 6/6 (FIFO, replay dedup, saturation, waiting-target wake, 32 KiB
  bound, and compaction).
- `spawn_orchestration`: 26/26.
- `restored_legacy_spawn_hierarchy_is_inspectable_but_rejects_mutation`.
- `restored_crew_validation_rejects_runtime_drift`.
- `spawn_roster_lines_carry_dispatch_and_report_seq`.
- `bound_nazgul_freeform_dispatch_routes_without_protocol_headers`.
- `cargo test -p codex-tui --lib --no-run` with the repository stack setting.
- `cargo check -p codex-tui`.

## Phase 7B — Native orchestration remains role-neutral

Status: the complete native V2 handler suite passes after removing the last role-name
authorization check.

Changes:

- Removed the core rule that denied manager tools solely because an agent's display role was
  `orc`. Role names remain behavioral profiles selected by `/spawn`; they are not capabilities.
- Deleted tests that encoded Nazgul/Troll/Orc as a second authorization graph inside Codex.
- Repaired adjacent V2 test fixtures so their parent/depth tuples describe real identity trees.
- Corrected the rejected `agents.max_threads` fixture: V2 rejects the incompatible legacy key and
  retains the configured V2 default of five, rather than silently honoring three.

Passing evidence:

- `RUST_MIN_STACK=33554432 cargo test -p codex-core --lib multi_agent_v2 --
  --test-threads=1`: 66 passed, 0 failed.

Product boundary:

- `/spawn` still owns crew membership, names, role instructions, pane presentation, and exact
  provider/model/effort selection.
- Native Codex owns provider-neutral identity, lifecycle, capacity, and mailbox mechanics.

## Phase 7C — Delete the second scheduler and native text transport

Status: structural deletion gates for the TUI dispatch path pass.

Changes:

- Removed `PumpSpawnDispatches`, its scheduled flag, in-flight target set, round-robin cursor,
  capacity deferral, completion callback, and active queue footer.
- Removed the active TUI-owned `spawn_pending_dispatches` state. Versioned layout fields remain
  deserialize-only compatibility input; new writers emit them empty, and a restored layout that
  contains queued legacy work is read-only and never replayed.
- Removed the external-pane automatic retry path. A legacy external Claude pane accepts one direct
  task only while idle; busy, oversized, or failed admission is explicit and non-retrying.
- Native crew members no longer receive or emit `pfterminal_send_task` as transport. Their live
  `/spawn` context instructs `send_message` and `followup_task` against canonical crew paths.
- Native assistant text is never scanned for dispatch tags. The text-envelope decoder and its
  mechanical contract are isolated in `external_plan_agent_adapter.rs` for genuinely external
  text-only panes.
- Removed the native malformed-tag correction loop and its per-thread retry counters.
- Preserved `/spawn` crew creation, named roles, exact provider/model/effort runtimes, pane
  navigation, roster injection, report rendering, and direct user task actions.

Structural evidence:

- Code search: 0 `PumpSpawnDispatches` / `SpawnDispatchPump` / pump-state references.
- Code search: 0 native text-dispatch or dispatch-correction references.
- Diff for this phase: 196 insertions and 1,613 deletions before journal updates.

Passing evidence:

- `cargo check -p codex-tui`.
- TUI test build with `RUST_MIN_STACK=33554432`.
- Real dispatch integration: 6/6, including FIFO, capacity saturation, wait wake, compaction,
  oversize rejection, and proof that replayed native assistant tags produce zero mailbox work.
- `spawn_orchestration`: 25/25.
- `native_agent_text_is_never_used_as_dispatch_transport`.
- `codex_main_bound_nazgul_turn_receives_domain_neutral_hierarchy_context`.
- `troll_spawn_task_submission_names_existing_orc_panes`.
- `/spawn` app regressions: 6 passed, 1 live-auth test intentionally ignored.

## Phase 5B — Provider-boundary crash recovery

Status: safe local work resumes; remotely ambiguous work is quarantined.

Timestamp: 2026-07-25T05:15:23Z

Changes:

- Added a distinct attempt ID written with the `submitting` transition before delivery. Retries
  retain the stable message and assignment IDs while receiving a new attempt identity.
- Wired mailbox application into the recipient turn lifecycle:
  `submitted -> provider_running -> completed`.
- A turn that aborts or fails after consuming an assignment moves that message from
  `provider_running` to `unknown_outcome`.
- Recovery distinguishes an unapplied `submitted` row (safe local queue replay) from a submitted
  row already present in the recipient rollout (provider outcome may be ambiguous). The latter is
  never replayed automatically.
- `submitting` and `provider_running` rows become `unknown_outcome` across process recovery and
  retain their attempt IDs for operator reconciliation.
- Completed rows remain queryable for audit but are excluded from recovery.

Passing evidence:

- `codex-state agent_mailbox`: 4 passed.
- `durable_agent_mailbox_deduplicates_and_completes_after_rollout_flush`.
- `ensure_v2_agent_loaded_reloads_registered_unloaded_agent`, expanded to cover admitted,
  submitted, retryable-failure, submitting, provider-running, completed/applied, and
  unknown-outcome crash seams.
- app-server V2 `thread_agent_message_uses_native_mailbox_and_deduplicates_stable_id`.
- complete native V2 handler filter: 66 passed, 0 failed.

Product boundary:

- These states belong to the native provider-neutral substrate. `/spawn` continues to own crew
  membership, names, hierarchy presentation, exact runtime selection, panes, and direct
  human navigation.

## Phase 6B — Preserve manual `/spawn` crews and separate them from task agents

Status: the native cutover retains the original `/spawn` product boundary.

Timestamp: 2026-07-25T05:33:45Z

Changes:

- Manually assembled `/spawn` crews now create and persist a custom `CrewSpec`, including the
  bound Nazgul root, every Troll/Orc native identity, parent edge, display name, and exact
  provider/model/effort tuple.
- Adding a member to a ready crew is transactional: validation and identity mapping complete on a
  clone before the live crew state changes. Customization clears the preset identifier without
  changing existing member identities.
- External Claude-plan panes enter the same durable crew mapping at the adapter boundary.
- Crew membership is determined by `CrewSpec` identity mapping, never by a role label. A normal
  native Codex task agent remains outside `/spawn` even if its descriptive role is `orc`,
  `troll`, or `nazgul`.
- Restored pre-CrewSpec layouts remain inspectable and read-only. They do not become writable
  merely because their old role labels resemble a crew.
- Focused custom-crew logic lives in `custom_spawn_crew.rs`; `spawn_crew.rs` remains 375 lines.

Passing evidence:

- `cargo check -p codex-tui`.
- TUI `/spawn` filter: 72 passed, 1 live-auth integration test intentionally ignored.
- `manually_assembled_multimodel_spawn_crew_is_crewspec_backed`.
- `native_task_agent_role_does_not_make_it_a_persistent_spawn_crew_member`.
- Crew state tests: exact identity round-trip, heterogeneous member addition, duplicate mapping
  rejection, and incomplete-state rejection.

Product boundary:

- `/spawn` is not deprecated or replaced. It owns persistent named crews, presets, hierarchy,
  provider/model/effort selection, panes, navigation, and direct human control.
- Native Codex spawning remains the mechanism for bounded task agents. Both products share native
  identity, mailbox, execution, interruption, and recovery primitives without sharing membership
  or retention semantics.

## Phase 6C — Live-discovered exact-runtime recovery repair

Status: same-session process restart preserves the `/spawn` crew and its selected runtimes.

Timestamp: 2026-07-25T06:04:24Z

Discovery:

- A fresh standard crew correctly created Fable, GPT-5.6-Sol, GPT-5.6-Luna, GPT-5.6-Terra, and
  Grok 4.5 members.
- Resuming the same root session in a new process initially marked the crew read-only. Recovery
  had resumed every child using the current parent pane's Opus runtime, then overwritten the
  crew-owned saved runtime with the resume response.
- A `None` reasoning effort in `CrewSpec` means use the selected provider/model's resolved
  default. Recovery incorrectly treated the resulting concrete effort as drift.

Repair:

- Restored native `/spawn` panes now resume with the provider, model, and resolved reasoning
  effort already persisted for that logical crew node.
- The returned runtime is checked before the restored session is attached. Provider/model drift,
  or drift from an explicitly requested reasoning effort, fails closed without mutating the
  saved crew runtime.
- Provider-resolved effort is accepted when the `CrewSpec` intentionally leaves effort
  unspecified.
- Starting a different root session still shows no old crew; crew state remains session-scoped.

Passing evidence:

- `restored_spawn_resume_uses_saved_runtime_instead_of_parent_runtime`.
- `restored_crew_validation_rejects_runtime_drift`, expanded to prove a provider-resolved effort
  is accepted when the specification leaves effort unspecified.
- `thread_resume_params_forward_explicit_model_override`.
- `/spawn` filter with `RUST_MIN_STACK=33554432`: 73 passed, 0 failed, 1 credentialed live test
  ignored.
- Live restart of root `019f97d5-ce22-7b31-aa22-f04656bbe782` restored Angmar, Burzum, Snaga,
  Ghash, and Krimp with no read-only recovery error.

## Phase 6D — Claude Plan tool-result replay repair

Status: the live Fable member continues across the request shape that previously killed its turn.

Timestamp: 2026-07-25T06:25:59Z

Discovery:

- During the real isometric-game objective, Fable twice failed immediately after a tool call with
  `This model does not support assistant message prefill. The conversation must end with a user
  message.`
- Capturing the actual outbound request proved that it already ended in a user message. The
  rejected shape was narrower:
  `assistant[thinking, tool_use, text] -> user[tool_result]`.
- Ordinary tool continuations in the same session ended
  `assistant[thinking, tool_use] -> user[tool_result]` and were accepted.

Repair:

- Claude Plan request construction now appends request-local `Continue.` text only when the
  terminal user message contains tool results exclusively and the preceding assistant message has
  non-empty text after its final tool call.
- Durable rollout history, signed thinking blocks, ordinary tool-result turns, Anthropic API-key
  traffic, and every non-Claude provider remain unchanged.

Automated evidence:

- Anthropic-focused core filter: 21 passed, 0 failed.
- `anthropic_request_normalizes_tool_result_after_trailing_assistant_text`.
- `anthropic_request_leaves_normal_tool_result_continuations_unchanged`.
- `anthropic_api_request_does_not_apply_claude_plan_tool_result_repair`.
- `claude_plan_request_normalizes_child_mail_after_completed_assistant_turn`.
- `cargo check -p codex-core`.
- `just fix -p codex-core`; only the pre-existing `responses_retry` argument-count warning remains.

Live evidence:

- Exact binary:
  `/home/pfrpc/repos/PfTerminal-telegram-hardening/codex-rs/target/debug/pfterminal`.
- Root `019f97d5-ce22-7b31-aa22-f04656bbe782`; Fable member
  `019f97d6-2bef-7643-a554-fc33ddd64ead`; tmux session `native_orch_qual_1`.
- Request SHA-256 `bfed0e0f6b2478a3fc1c7ea1f04c96b55caa3832141da214fa6f53b64b74e0c1`
  replayed `assistant[thinking, tool_use, text] -> user[tool_result, text("Continue.")]`;
  Fable accepted the tool result and continued.
- Request SHA-256 `8ea24633ebbf77abdb713650d345bcf882ef9fcd6d76011729d8b7c41bcd78d2`
  independently repeated the same repaired shape; Fable continued again with no 400.
- The live turn subsequently performed additional shell, image, and search calls while preserving
  the same standard `/spawn` crew and exact runtime.

## Phase 6E — Rejoin restored crew members to one native control plane

Status: a restored `/spawn` hierarchy is one live native agent tree, not merely a set of panes.

Timestamp: 2026-07-25T06:36:19Z

Discovery:

- After process recovery, `/spawn` showed the original Burzum, Snaga, Ghash, and Krimp threads, but
  Fable's native collaboration registry could not resolve the existing Burzum path.
- Fable's failed follow-up led it to create a replacement at the same canonical path. This proved
  that TUI crew restoration and native collaboration restoration were not yet the same operation.
- Fresh child creation already shared the parent's `AgentControl`. The split occurred only during
  generic `thread/resume`, which allocated a new control for every restored thread.

Repair:

- Generic thread resume now reads the persisted session source first. A thread-spawn child rejoins
  the loaded parent's `AgentControl`; a root thread still receives a new tree-scoped control.
- Existing registration on successful resume restores the child's canonical path and metadata in
  that shared registry. Native path reservation therefore rejects a replacement at an already-live
  crew path.
- `/spawn` continues to own crew membership and presentation. This change repairs the native
  lifecycle substrate beneath it.

Automated evidence:

- `resumed_subagent_rejoins_loaded_parent_control_plane` passes. It restarts a parent and named
  child from persisted rollouts, proves both sessions share one control-plane identity, proves the
  child path resolves from each side, and proves a duplicate `/root/troll_burzum` spawn is rejected.
- `cargo check -p codex-core` passes.
- The known pre-existing
  `resume_thread_subagent_restores_stored_metadata_and_effective_multi_agent_mode` timing race
  failed once and passed twice without code changes; it does not exercise generic
  `ThreadManager::resume_thread_with_history`.

Live evidence:

- Exact debug binary was rebuilt, then root `019f97d5-ce22-7b31-aa22-f04656bbe782` and its crew were
  resumed in tmux session `native_orch_registry_probe`.
- Fable's `list_agents` returned:
  `/root/nazgul_angmar`,
  `/root/nazgul_angmar/troll_burzum`,
  `/root/nazgul_angmar/troll_burzum/orc_snaga`,
  `/root/nazgul_angmar/troll_burzum/orc_ghash`, and
  `/root/nazgul_angmar/troll_burzum/orc_krimp`.
- Fable used `followup_task` on the existing Burzum path. The original thread
  `019f97d6-2cae-7ab0-a008-5ece5f704fcf` completed the turn and returned
  `provider=openai model=gpt-5.6-sol`.
- Fable explicitly reported `No files edited, no agents spawned`; the live tree contained one
  Burzum path.

## Phase 6F — Preserve active native descendants across process restart

Status: repair passes automated qualification and the exact live failure replay.

Timestamp: 2026-07-25T07:21:00Z

Discovery:

- Fresh qualification session 1 used root
  `019f9802-e35a-7802-99c1-ffa60a97fa71`, Fable Nazgul
  `019f9803-64f7-77f2-b9f7-f618f084737e`, Sol Troll
  `019f9803-65aa-7ad1-b38b-897338d8928c`, Luna/Terra/Grok Orcs, and the Troll-created
  ephemeral verifier Godel `019f980d-0ffd-7150-abc9-1906f81fe372`.
- The session exercised rapid native follow-ups, four-slot saturation, direct human input to a
  busy Troll, an interrupted Opus turn, Fable compaction, and a process restart with an injected
  bounded `turn/start` failure.
- CrewSpec restored every persistent `/spawn` member with its original runtime, but Godel
  disappeared from `list_agents` after restart. The open thread-spawn edge and rollout still
  existed; only the new process's in-memory registry had forgotten the accepted native work.
- Qualification session 1 is invalid and the three-session count is reset. Its source tree is
  disposable evidence, not a passing qualification result.

Repair:

- Persisted open descendants are reconstructed into the resumed tree's shared `AgentControl`
  without eagerly opening a provider process.
- A persisted, addressable, non-resident agent has the explicit non-final status `Unloaded`.
  It remains visible in native listings and JSON/app-server/TUI representations.
- Sending work to an unloaded agent uses the existing V2 lazy-residency path, which reloads the
  exact persisted provider, model, reasoning effort, identity, and history before delivery.
- Restored paths remain reserved, restored capacity is counted once, and later loading the same
  thread does not double-count it.
- A stale descendant record is logged and skipped without preventing the user's root session from
  opening.
- `/spawn` membership is unchanged: CrewSpec still owns the named persistent crew; generic native
  descendants remain native task agents and do not become crew members.

Automated evidence:

- Workspace `cargo check --workspace` passes.
- Core registry filter: 17 passed, including
  `restored_threads_are_counted_once_and_keep_their_path_reserved`.
- Core AgentControl filter: 55 passed, including
  `resume_agent_from_rollout_does_not_reopen_v2_descendants` and the missing-rollout boundary.
- `resumed_root_restores_open_descendants_as_unloaded_with_exact_runtime` passes. It resumes only
  the root, proves child and grandchild are visible but not resident, then addresses the
  grandchild and proves it reloads as `openrouter` / `x-ai/grok-4.5`.
- App-server protocol library: 246 passed; the `unloaded` wire value is covered directly.
- TUI multi-agent filter: 5 passed.
- Exec library: 59 passed; JSONL output preserves the `unloaded` state.

Exact live replay:

- Commit: `29a8b3008`.
- Immutable qualification binary:
  `/tmp/pfterminal-native-orch-29a8b3008`.
- Binary SHA-256:
  `6f092ce4ee0ad646c7a3e7c68c6b19af667295d37e8a93948eced5631951b61e`.
- Fable resumed in `native_orch_replay` from the copied session home used by the failed
  qualification. Native `list_agents` reported the original Godel identity
  `/root/nazgul_angmar/troll_burzum/verification_reassign` with exact status `unloaded`.
- Fable used `followup_task` on that existing path. No replacement was spawned. Godel's original
  thread `019f980d-0ffd-7150-abc9-1906f81fe372` accepted the message and replied
  `no files edited`.
- Runtime evidence for that turn records
  `model=gpt-5.6-sol codex.turn.reasoning_effort=xhigh` on the same Godel thread ID.
- The invalid replay worktree was recreated only long enough to satisfy the stored cwd and is
  removed immediately after this proof.
- `cargo clean --profile dev` reclaimed 102.5 GiB after the immutable binary was copied; free disk
  rose from 63 GiB to 158 GiB before the replay.

## Phase 8A — Fresh qualification pass 1 reset: chat-wire agent mail was omitted

Status: invariant failure found; qualification count remains zero.

Timestamp: 2026-07-25T07:52:07Z

Candidate:

- Binary: `/tmp/pfterminal-native-orch-29a8b3008`.
- SHA-256: `6f092ce4ee0ad646c7a3e7c68c6b19af667295d37e8a93948eced5631951b61e`.
- Fresh copied home: `/tmp/pft-native-orch-pass1-home-20260725b`.
- Disposable game tree: `/tmp/isometric-native-orch-pass1-20260725b`.
- Session ran from 2026-07-25T07:43:29Z to 2026-07-25T07:52:07Z and is not counted.

Observed:

- `/spawn` created Opus root, Fable Nazgul, Sol Troll, Luna/Terra/Grok Orcs with the expected
  explicit runtimes.
- Burzum issued a real Grok follow-up with durable message ID
  `019f983f-12bb-71c2-b46a-e0e61581372c`. The full task appears in Krimp's typed
  `inter_agent_communication` rollout item.
- Krimp's immediately triggered turn nevertheless replied that it had received no task. Two
  adjacent follow-ups (`019f983f-d3d5-77c3-8402-8d91d76cc213` and
  `019f9840-31cb-7e21-882a-b96e7fba277b`) reproduced the same result.
- Source inspection found the boundary: `append_chat_message_for_response_item` deliberately
  discarded every `ResponseItem::AgentMessage`. Native Responses runtimes retained collaboration
  mail, while OpenRouter chat-wire runtimes received a turn with that mail omitted from the
  provider request.

Disposition:

- This is a Section 6.2 accepted-message application failure, not a model-quality observation.
- The process and watcher were stopped. Generated game artifacts are disposable.
- Repair the chat adapter so canonical typed collaboration mail becomes user-role provider input
  while untyped display/transcript agent messages remain excluded. Add generalized regressions and
  restart the three-session count from zero on a new immutable binary.

## Phase 8B — External-provider mailbox repair and exact live replay

Status: repaired and live-replayed in both directions; qualification count remains zero.

Timestamp: 2026-07-25T08:22:15Z

Repair:

- Canonical typed `AgentMessage` collaboration items now map to a user-role protocol envelope at
  the external-provider adapter edge. The envelope carries validated sender and recipient agent
  paths plus the plaintext message.
- Legacy untyped display/transcript `AgentMessage` items remain omitted, so transcript rendering
  cannot become provider input accidentally.
- The same mapping is applied to Chat Completions and Anthropic Messages. Native Responses
  transport is unchanged.
- Tests cover typed collaboration delivery and untyped omission for generic chat completions,
  OpenRouter, and Claude Plan.
- Commit `07150c78e` contains the adapter repair.

Automated evidence:

- All 50 `client::tests::` pass.
- `chat_completions_maps_typed_collaboration_mail_to_user_input`,
  `chat_completions_omits_untyped_agent_messages_from_history`,
  `openrouter_request_includes_typed_collaboration_mail`, and
  `claude_plan_request_maps_typed_child_mail_to_user_input` pass.
- The pending-input filter, `just fix -p codex-core`, `just fmt`, and
  `cargo check -p codex-tui` pass.

Exact live replay:

- Immutable binary:
  `/tmp/pfterminal-native-orch-07150c78e`.
- Binary SHA-256:
  `f42604ca2a2209cc729d0ccf499661d4f6978a330d936f9792e1ccfa30d179b3`.
- Fable resumed the exact failed crew and used `followup_task` on existing Krimp path
  `/root/nazgul_angmar/troll_burzum/orc_krimp`.
- Durable message ID `019f9851-e25d-78c0-a54c-fd217f95420a` asked Krimp to make no edits and
  return exactly `MAILBOX_FIX_OK`.
- Krimp's OpenRouter `x-ai/grok-4.5` rollout explicitly reasoned from the received mailbox message,
  returned `MAILBOX_FIX_OK`, and completed with that exact result.
- Fable received the child result and reported Krimp's exact `MAILBOX_FIX_OK` response. This proves
  both the OpenRouter inbound mapping and the Claude Plan inbound result mapping on real provider
  requests.

## Phase 8C — Fresh qualification reset: native task tree leaked into CrewSpec recovery

Status: repaired; exact fresh restart replay passed.

Timestamp: 2026-07-25T08:22:15Z

Observed:

- The exact mailbox replay also reproduced a restart warning for unmaterialized root thread
  `019f983a-ab0b-7020-8863-ce74e9d24485`:
  `thread/read failed` followed by `thread/resume failed: no rollout found`.
- The root was created before its first user turn, so it legitimately had no rollout. It was not a
  `/spawn` crew member, but recovery had inserted it beside the explicit Fable CrewSpec root.
- On a later resume, the leaked edge made role inference classify the unrelated root as a Troll
  and materialize a replacement endpoint. This violated the crew/task-agent class boundary and
  invalidated the candidate.

Repair:

- CrewSpec-backed recovery no longer infers another root from the currently resumed primary
  thread. Root inference remains only for legacy layouts without a CrewSpec.
- Recovery seeds retention from explicit CrewSpec member mappings, then retains only native
  descendants reachable from those members. Unrelated top-level native trees remain native task
  agents and are recovered by the native registry, not reclassified as `/spawn` panes.
- Existing layouts written by the faulty path are repaired during restore: leaked parent edges,
  `/spawn` runtime entries, and `/spawn` endpoint mappings are pruned before any attach or
  materialization attempt.

Automated evidence:

- `crewspec_restore_prunes_unrelated_native_tree_but_keeps_crew_descendants` passes. It proves a
  real crew descendant remains routable while an unrelated native root and its descendant are
  removed from `/spawn` recovery state.
- `/spawn` filter: 73 passed, 0 failed, 1 credentialed live test intentionally ignored.
- `native_task_agent_role_does_not_make_it_a_persistent_spawn_crew_member` passes.
- `cargo check -p codex-tui`, `just fmt`, and `git diff --check` pass.

Exact replay:

- A fresh home with no prior sessions created a standard CrewSpec before the primary root had any
  model turn or rollout.
- The saved `/spawn` map contained only the explicit Fable Nazgul, Sol Troll, and three Orcs.
  The unmaterialized primary remained a native registry root and did not enter CrewSpec recovery.
- Restarting from the Fable thread produced no pane restore errors, no replacement panes, and no
  duplicate crew members. Native `list_agents` showed the native `/root`, the one explicit Nazgul,
  one Troll, and three unloaded Orc descendants.

## Phase 8D — Exact crew runtime was overwritten at the bind/resume boundary

Status: repaired and exact provider-replayed; qualification count remains zero.

Timestamp: 2026-07-25T08:51:07Z

Observed:

- The first fresh restart replay showed `claude-opus-5-plan high` after directly resuming the
  explicit Fable Nazgul. A controlled provider turn confirmed that it actually sampled Opus, so
  this was not a cosmetic startup banner.
- The CrewSpec and pane runtime map still held the correct
  `claude-plan / claude-fable-5-plan / high` tuple.
- Two persistence boundaries were incomplete:
  `SpawnThreadStateMetadata` omitted reasoning effort, and binding a separate native Nazgul root
  overwrote its saved Fable runtime with the currently focused parent pane's Opus runtime.
- Generic startup resume also waited for the app-server fallback path instead of applying the
  state database's exact tuple before provider bootstrap.

Repair:

- Native spawn registration now persists provider, model, and reasoning effort together.
- Bound native roots prefer their own saved runtime tuple; the focused parent pane is only a
  fallback when no native runtime exists.
- Startup resume reads the persisted thread tuple before bootstrap and applies it as an explicit
  resume override unless the user supplied an explicit model/provider override.
- Commits `12dc2160f` and `0fb731874` contain the repair.

Automated evidence:

- `startup_resume_applies_persisted_runtime_before_bootstrap` passes.
- `native_spawn_registration_persists_started_session_model_provider_pair` now asserts persisted
  `xhigh` effort in addition to provider and model.
- `bound_nazgul_root_persists_role_metadata_to_state_db` proves a focused Opus parent cannot
  overwrite a bound Fable root's provider/model/effort.
- The 73-test `/spawn` suite passes with `RUST_MIN_STACK=33554432`; the repository's known default
  test-thread stack overflow is avoided without skipping any of these tests.
- `cargo check -p codex-tui`, `just fmt`, and `git diff --check` pass.

Exact replay:

- Candidate binary: `/tmp/pfterminal-native-orch-0fb731874`.
- SHA-256: `bb43a417dbfb71e3bc23be393829d3a09f0217cc472930a5552d6937b0b593c0`.
- Fresh home: `/tmp/pft-native-orch-runtime-replay4`.
- The standard crew was created before any primary-root model turn. SQLite immediately held:
  Fable/high Nazgul, Sol/xhigh Troll, Luna/xhigh Orc, Terra/xhigh Orc, and Grok/high Orc.
- After process termination, direct resume of Fable thread
  `019f9876-c834-7cf2-87f9-19d1b6986860` rendered `Claude Fable 5 Plan high`.
- The controlled turn returned `FABLE_RESUME_OK`. Provider logs contain
  `model=claude-fable-5-plan` and no `claude-opus-5-plan` for that turn.
- `list_agents` showed exactly the native root plus the five CrewSpec members. The three unloaded
  descendants retained their original canonical paths; no phantom or replacement child appeared.

Disposition:

- The prerequisite restart/recovery matrix now passes on the immutable candidate.
- Qualification count remains zero by design. Begin three fresh 45–60 minute free-form sessions
  from this exact source state; any new invariant failure resets the count.

## Phase 8E — Human manager input was rejected under worker saturation

Status: live failure reproduced; control-plane boundary repaired; qualification count reset to
zero.

Timestamp: 2026-07-25T09:12:07Z

Observed:

- Fresh qualification session `/tmp/pft-native-orch-s1-home-20260725` created the standard
  Fable/Sol/Luna/Terra/Grok crew on candidate
  `bb43a417dbfb71e3bc23be393829d3a09f0217cc472930a5552d6937b0b593c0`.
- The injected first `thread/spawnAgent` failure retried once and the complete crew materialized.
- Fable used the native mailbox to start Sol, which dispatched all three Orcs. Fable then created
  one independent native reviewer, occupying all five configured descendant execution slots.
- The user interrupted Fable and submitted a reprioritization. `turn/start` rejected the human
  message with `Cannot start turn: all 5 execution slots are in use.`
- The TUI remained alive and emitted no retry flood, but spec sections 6.3, 10, and 15 explicitly
  require the manager to remain human-addressable under saturation. The session is invalid and
  does not count.

Repair:

- `AgentControl::ensure_execution_capacity_for_op` now classifies only agent-triggered mailbox
  work and pending-work wakeups as worker execution. Human `Op::UserInput` is control-plane work.
- Once admitted, the human-started turn still acquires the ordinary execution guard. This permits
  the explicit control-plane turn to cross a saturated admission boundary while keeping the turn
  visible to the same atomic capacity accounting until it ends.
- The rule is provider-neutral and role-neutral: it does not special-case Nazgul, Troll, Orc,
  Fable, or the reported sentence.

Automated evidence:

- `human_input_is_control_plane_work_not_worker_execution` passes.
- `human_input_to_manager_is_accepted_while_worker_capacity_is_saturated` constructs a real native
  root and manager, reserves the only worker slot, proves a second autonomous reservation is
  rejected, and proves direct human input to the manager is still accepted.
- The complete `agent::control::execution` filter passes: 5 passed, 0 failed.
- `cargo check -p codex-tui`, `cargo fmt --all -- --check`, and `git diff --check` pass.

Next:

- Commit the repair, stop the invalid session, salvage its small forensic evidence, and remove its
  disposable game worktree.
- Build a new immutable candidate and restart the three-session qualification count at zero.

Wind-down:

- At `2026-07-25T09:21:15Z`, stopped only tmux session `native_orch_s1`; no process rooted in its
  temporary home or game tree remained afterward.
- Removed disposable worktree `/tmp/isometric-native-orch-s1-20260725` immediately after its
  failure proof completed and deleted branch `qual/native-orch-s1-20260725`.
- Root filesystem free space after reclaim: 113 GiB.

Replacement candidate:

- Repair commit: `e1d05c033`.
- Binary: `/tmp/pfterminal-native-orch-e1d05c033`.
- SHA-256: `a4b382c91a1c2f42061742dfe86d0ae8500e6ace430ec68442bf676eea5e0685`.
- Root filesystem free space after the build: 109 GiB.
- This candidate supersedes `bb43a417dbfb71e3bc23be393829d3a09f0217cc472930a5552d6937b0b593c0`;
  no qualification result from the superseded binary counts.

## Phase 8F — Exact live saturation seam passes on the replacement candidate

Status: prerequisite seam pass; long-session qualification count remains zero.

Timestamp: 2026-07-25T09:27:41Z

Evidence:

- Fresh home `/tmp/pft-native-orch-satfix-home-20260725` created the standard CrewSpec, then
  directly resumed its persisted Nazgul thread as `claude-plan / claude-fable-5-plan / high`.
- Fable used native `followup_task` to start Sol Troll Burzum and native `spawn_agent` to create
  independent reviewer `/root/nazgul_angmar/epicurus`.
- Burzum started Luna, Terra, and Grok Orcs. `/spawn status` showed Burzum, Snaga, Ghash, and
  Krimp all `running`; the native reviewer was also `running`.
- Process evidence at the control boundary showed four independent `sleep 600` jobs plus Burzum's
  active supervising turn, occupying all five configured descendant execution slots.
- While those five turns were active, direct human input to the idle Fable manager was accepted at
  `2026-07-25T09:27:20.455Z`; Fable returned `MANAGER_CONTROL_OK` at
  `2026-07-25T09:27:24.256Z`.
- Logs contain no `AgentLimitReached`, `Cannot start turn`, or execution-slot rejection for the
  control turn.

Disposition:

- The exact crash-shaped boundary that invalidated the first session now passes live.
- This short seam test is not one of the required 45–60 minute free-form qualification sessions.
  Begin the three-session count at zero on this same candidate hash.
- Stopped only tmux session `native_orch_satfix` after evidence collection. Four child jobs exited
  with the TUI; one known orphaned sleep PID was terminated explicitly, leaving no process rooted
  in the temporary home or workspace.

## Phase 8G — Standard-crew free-form rehearsal exceeded the qualification window

Status: useful adversarial evidence, but not a counted Section 15.3 pass.

Timestamp: 2026-07-25T12:31:00Z

Candidate and environment:

- Exact binary: `/tmp/pfterminal-native-orch-e1d05c033`.
- SHA-256: `a4b382c91a1c2f42061742dfe86d0ae8500e6ace430ec68442bf676eea5e0685`.
- Fresh home: `/tmp/pft-native-orch-q1-home-20260725`.
- Structured log: `/tmp/pft-native-orch-q1-logs-20260725/codex-tui.log`.
- Disposable game branch: `qual/native-orch-q1-20260725`.
- Game base: `0394dda07d066827a5b9a6ae977d617e3e47aba9`.

Observed:

- The first injected `thread/spawnAgent` failure fired at
  `2026-07-25T09:29:10.846928Z`; the bounded retry created the full crew without a stillborn
  member or notification flood.
- Direct resume of the explicit manager preserved
  `claude-plan / claude-fable-5-plan / high`.
- The standard Sol/Luna/Terra/Grok descendants materialized with their configured runtimes.
  Provider logs show real turns for Fable, Sol, Luna, Terra, and Grok rather than labels alone.
- The manager discovered that port 8444 belonged to an unrelated repository, rejected that false
  environment assumption, started the game on its own port, and drove real browser inspection.
- A natural human reprioritization submitted during the campaign was retained and applied at the
  next turn boundary. The manager continued using native follow-up and crew dispatch.
- A real upstream ChatGPT-backend 503 interval affected several descendants. The manager and Troll
  kept the campaign live, reassigned the smallest critical work, and did not duplicate accepted
  work or change providers.
- The campaign produced three benchmark commits:
  `0d53ce2`, `44c48f6`, and `f33f02e`. The manager rejected and rolled back two visual changes
  that failed live-composite review instead of accepting metric-only improvements.
- The TUI stayed alive and the manager remained addressable. No `Cannot start turn`, execution-slot
  rejection, compaction-trigger ordering error, assistant-prefill error, modified-thinking error,
  panic, or retry-notification flood was observed.

Disposition:

- The reported active duration was `1h 16m 25s`, outside the specification's explicit 45–60
  minute window. It is therefore a rehearsal, not qualification session 1.
- The three-session count remains zero. Replacement sessions will receive a closeout instruction
  at 45 minutes and a hard observer stop before 60 minutes.
- The benchmark commits remain reachable from branch `qual/native-orch-q1-20260725`; generated
  proof artifacts in the disposable worktree are not product changes and will be reclaimed after
  this journal entry.
- Tmux session `native_orch_q1` was stopped and disposable worktree
  `/tmp/isometric-native-orch-q1-20260725` was removed after evidence capture. Root filesystem
  free space remained 108 GiB.

## Phase 8H — Counted free-form session 1 passes, including native mixed-provider agents

Status: Section 15.3 session 1 PASS; qualification count is 1 of 3.

Timestamp: 2026-07-25T13:26:24Z

Candidate and duration:

- Exact binary: `/tmp/pfterminal-native-orch-e1d05c033`.
- SHA-256: `a4b382c91a1c2f42061742dfe86d0ae8500e6ace430ec68442bf676eea5e0685`.
- Fresh home: `/tmp/pft-native-orch-q1c-home-20260725`.
- Structured log: `/tmp/pft-native-orch-q1c-logs-20260725/codex-tui.log`.
- Real objective interval: `2026-07-25T12:40:02Z` through
  `2026-07-25T13:26:24Z` (46 minutes 22 seconds).
- Benchmark branch: `qual/native-orch-q1c-20260725`, based on
  `0394dda07d066827a5b9a6ae977d617e3e47aba9`.

Native orchestration evidence:

- `/spawn` was used only to materialize the required standard CrewSpec. The Fable manager then
  used ordinary native Codex collaboration tools for assignments, follow-ups, mailbox waits, and
  three independent task agents.
- Native Kimi agent `/root/nazgul_angmar/reviewer_k3`, thread
  `019f9956-6dfa-7283-a74f-e7b190a4c1ef`, was created with
  `kimi-code / k3 / high`. Provider logs show successful Kimi chat-completions requests and real
  browser/tool work.
- Native Opus agent `/root/nazgul_angmar/reviewer_opus_plan`, thread
  `019f9962-e83d-7e33-8132-940114967bc9`, was created with
  `claude-plan / claude-opus-5-plan / high`. The Anthropic endpoint returned 200 and reported
  `claude-opus-5`; the agent delivered a substantive independent gate review.
- Native Grok agent `/root/nazgul_angmar/reviewer_grok`, thread
  `019f9966-4349-7690-a494-0ea6eecb0c8a`, was created with
  `openrouter / x-ai/grok-4.5 / high`. OpenRouter returned 200. Its first terminal answer was a
  premature progress statement; native `followup_task` resumed the same agent and obtained the
  required four-section report without replacement or duplicated local application.
- These task agents were not CrewSpec members and never appeared as Troll/Orc replacements. They
  shared native thread identity, execution capacity, mailbox delivery, and completion handling
  with the standard crew.

Required free-form coverage:

- Natural reprioritization arrived during Fable tool activity and was applied once at the next
  turn boundary. Burzum rapidly dispatched all three Orc lanes and used later follow-ups for
  evidence rework.
- The human addressed Burzum and Snaga directly through the roster. Both returned terse durable
  state; Snaga reported commit `66a04a0` and explicitly confirmed the restart lost nothing.
- The process was stopped and resumed against the same Fable root. Crew thread IDs, worktrees,
  provider/model/effort tuples, queued work, and accepted results survived. No replacement crew
  appeared.
- Explicit `/compact` completed with `Context compacted`; no compaction-trigger ordering,
  assistant-prefill, or modified-thinking error appeared.
- The configured first `thread/spawnAgent` retryable fault fired before crew creation. The bounded
  retry materialized the crew without a stillborn member or notification flood.
- The first Grok native spawn was rejected cleanly at the resident-agent limit. After Opus
  completed and unloaded, the same Grok spawn succeeded. Later the manager reported all six
  execution slots occupied.
- While 6/6 slots were active, direct human input to Fable was accepted. Fable replied:
  `6/6 slots active ... I remain fully addressable; all lanes running undisturbed.`
- Fable, Sol, Luna, Terra, Kimi, Opus, and Grok all executed real provider turns. No provider
  silently changed.

Benchmark outcome and product findings:

- Durable branch commits are `ab19c83`, `15a4d60`, and `5d9e35c`. The work produced real
  before/after gait evidence, an eight-beat two-world player-journey harness, and independent
  visual/gameplay captures under `artifacts/session29-review-k3/`.
- Burzum rejected two metric-only submissions before integration: idle before/after captures that
  were byte-identical, and a route-stall predicate that stopped a legitimate long route early.
- Opus found that the run-gait renderer branch was test-hook reachable but not yet driven by
  ordinary gameplay. Kimi reproduced distant attack-approach failure and N/NW silent stalls.
  These are benchmark product findings, not PfTerminal orchestration failures.
- Grok's progress-only first terminal result is a model-quality/product finding. The native
  follow-up path worked and delivered the cached report; there was no lost message, duplicate
  application, or control-plane failure.

Invariant watcher and wind-down:

- No `Cannot start turn`, turn/start failure, compaction ordering error, assistant-prefill error,
  modified-thinking error, panic, fatal runtime error, `unknown_outcome` auto-replay, headless
  write, or retry-notification flood occurred.
- The observer sent a no-new-work closeout after 43 minutes and stopped the TUI at 46 minutes
  22 seconds. Tmux exited and no process remained rooted in the fresh home, benchmark tree, or
  child worktrees.
- Benchmark commits remain reachable from `qual/native-orch-q1c-20260725`. Generated proof
  artifacts are temporary and may be reclaimed after this journal is committed.
- Root filesystem free space before reclaim was 106 GiB.

Post-stop salvage:

- Two descendants completed commits during the bounded closeout. Preserved detached commit
  `c44e0b5` on `qual/native-orch-q1c-snaga-a2` and detached commit `4063b70` on
  `qual/native-orch-q1c-krimp-c2` before removing their worktrees. They are benchmark outputs,
  not PfTerminal candidate changes.
- Removed the main disposable tree and all five child worktrees immediately after preserving the
  branch refs. Root filesystem free space rose from 106 GiB to 109 GiB.

Disposition:

- Session 1 counts. The exact candidate remains immutable.
- Session 2 must create its custom heterogeneous agents directly through native `spawn_agent`
  runtime overrides; `/spawn` must not be used for agent creation in that session.

## Phase 8I — Native custom-crew attempt exposes hidden runtime controls

Status: invariant failure; prior count reset from 1 to 0; candidate superseded pending rebuild.

Timestamp: 2026-07-25T13:32:20Z

Observed:

- A fresh `/spawn`-free session started with Opus 5 as the root and a plain game onboarding/input
  objective. The user explicitly requested native Fable, Kimi, Grok, and Sol agents.
- The underlying V2 handler already accepts `model_provider`, `model`, and `reasoning_effort`.
  However, `MultiAgentV2Config` defaulted `hide_spawn_agent_metadata` to `true`, so the
  model-visible `spawn_agent` schema omitted all three runtime fields.
- Opus stated that its tool exposed only `task_name`, `message`, and `fork_turns`. It therefore
  placed provider/model labels in task text and omitted the actual runtime arguments.
- Native agents `/root/plan_lead` and `/root/kbd` consequently inherited
  `claude-plan / claude-opus-5-plan / high`. Logs prove the Kimi-labeled keyboard agent was
  actually Opus. This is a silent wrong-provider result and invalidates the session under
  Sections 6, 15.3, and 16.

Repair:

- Default `hide_spawn_agent_metadata` to `false`. Explicit configuration may still set it to
  `true`; the existing hidden-schema test remains.
- The ordinary V2 schema now exposes `model_provider`, `model`, `reasoning_effort`,
  `service_tier`, and `agent_type`, along with guidance that provider and model must be set
  together.
- Updated three stale durable-mailbox schema assertions from encrypted to plaintext. The product
  code intentionally removed encrypted tool fields in `ddaf1ff9c`; the tests had not followed
  that change.

Automated evidence:

- `multi_agent_v2_default_session_thread_cap_counts_root` now explicitly proves runtime metadata
  is visible by default.
- All five `spawn_agent_tool_*` schema tests pass, including visible runtime fields and the
  explicit hidden-metadata configuration.
- `send_message_tool_requires_message_and_has_no_output_schema` and
  `followup_task_tool_requires_message_and_has_no_output_schema` pass.
- `cargo fmt --all -- --check` passes.

Disposition:

- The attempted session does not count. Stop it, remove its disposable worktree, commit the
  generalized schema boundary repair, rebuild a new immutable candidate, and restart the
  three-session count at zero.

## Phase 8J — Rebuilt candidate and native runtime-override seam

Status: seam passed; qualification count remains 0 of 3.

Timestamp: 2026-07-25T13:43:10Z

Candidate:

- Source commit: `a786872cc` (`fix: expose native agent runtime overrides by default`).
- Immutable binary: `/tmp/pfterminal-native-orch-a786872cc`.
- SHA-256:
  `0ec2cc672677257aa1686fb102926b0ab91f69315f905b31e72f280f95520569`.
- Reported version: `0.1.22`.

Live seam evidence:

- Fresh copied home: `/tmp/pft-native-orch-schema-seam-home-20260725`.
- Tmux session: `native_orch_schema_seam`.
- Root runtime: `claude-plan / claude-opus-5-plan / high`.
- The root called native `spawn_agent` with structured arguments
  `model_provider=kimi-code`, `model=k3`, and `reasoning_effort=high`. Runtime labels were not
  placed in the task message as a substitute.
- Native child `/root/schema_kimi` completed with literal result `KIMI_SCHEMA_OK`.
- Child rollout metadata records `model_provider=kimi-code`; its `turn_context` records
  `model=k3` and `effort=high`. This is runtime evidence, not a label inferred from the prompt.

Qualification-home finding:

- The first seam attempt ran from the PfTerminal source directory and correctly selected Kimi,
  but could not resolve `KIMI_API_KEY`. The copied vault credential is environment-scoped to the
  benchmark repository; starting the same fresh copied home in the benchmark repository restored
  the credential without exposing or rewriting it.
- This was a qualification setup error, not counted as a product pass or failure. No candidate
  code changed after the immutable build.

Disposition:

- The generalized native runtime-override boundary is now proven live.
- Begin the three counted 45–60 minute free-form sessions on this exact candidate. Any invariant
  failure resets the count.

## Phase 8K — Counted session 1 passes on the rebuilt candidate

Status: Section 15.3 session 1 PASS; qualification count is 1 of 3.

Timestamp: 2026-07-25T14:31:20Z

Candidate and duration:

- Exact binary: `/tmp/pfterminal-native-orch-a786872cc`.
- SHA-256:
  `0ec2cc672677257aa1686fb102926b0ab91f69315f905b31e72f280f95520569`.
- Source commit: `a786872cc`.
- Fresh copied home: `/tmp/pft-native-orch-q1e-home-20260725`.
- Structured log: `/tmp/pft-native-orch-q1e-logs-20260725/codex-tui.log`.
- Real session interval: `2026-07-25T13:45:27Z` through
  `2026-07-25T14:31:20Z` (45 minutes 53 seconds).
- Benchmark branch: `qual/native-orch-q1e-20260725`.

Runtime and native-agent evidence:

- The standard preset materialized Fable Nazgul
  `019f9986-8e1d-70d1-aee4-3a3706cf11eb`, Sol Troll
  `019f9986-8ed6-78b0-90c7-393dec8a4e3f`, Luna Orc
  `019f9986-8f9b-7711-a43f-c41083cf4dbb`, Terra Orc
  `019f9986-9067-79c1-8aa1-192617b98bc9`, and Grok 4.5 Orc
  `019f9986-911f-7362-a734-41d6241a6e87`.
- A separate native reviewer `/root/nazgul_angmar/reviewer_k3`, thread
  `019f998a-de18-78a0-b681-1a08b5b20cef`, ran with persisted
  `kimi-code / k3 / high` runtime metadata. It was not a preset member or a relabelled Orc.
- After the PfTerminal process restart, native `followup_task` resumed that same Kimi thread.
  Its rollout retained `model=k3` and `effort=high`; it emitted `KIMI_RESTART_OK` and completed
  a substantive independent review.
- Logs show real provider traffic for Fable, Sol, Luna, Terra, Grok, and Kimi. Every server-reported
  model matched the requested model; no runtime silently migrated.

Required free-form coverage:

- The crew pursued a real game objective: diagnose and improve visible ground-plate banding,
  implement a deterministic render fix, build a frozen gate, inspect live gameplay, and conduct
  an independent review.
- Rapid dispatch created three concurrent Orc lanes plus the native Kimi reviewer. Natural
  reprioritization and follow-ups were applied at turn boundaries.
- The user directly addressed multiple members while all six execution slots were occupied.
  Burzum and the Orcs remained addressable; no worker was cancelled merely to accept control
  input.
- Snaga and Burzum both compacted successfully. Their stable thread IDs continued afterward and
  no `compaction_trigger`, assistant-prefill, or modified-thinking error appeared.
- The process was stopped and explicitly resumed from root
  `019f9986-10e9-7703-9da5-cdf20e184e44`. The same crew and Kimi thread IDs, provider tuples,
  shared branch, and accepted work survived.
- The configured first `thread/spawnAgent` fault fired at
  `2026-07-25T14:01:23Z`; the bounded retry created exactly one native user pane without a
  stillborn pane or notification flood.
- The native Kimi reviewer was explicitly interrupted near completion, resumed on the same
  thread, restored its turn-owned temporary changes, and produced one final report.
- The final accepted human closeout turn encountered a normal-closure Sol websocket disconnect.
  The bounded reconnect replayed the provider request within the same local turn ID and returned
  exactly one closeout response. No local message or assignment was duplicated.

Environmental and benchmark findings:

- The host Claude-plan OAuth refresh token expired during the session. Fable stopped with an
  actionable missing-refresh-token error; PfTerminal did not change its provider or credential,
  and Sol/Kimi/Grok/OpenAI lanes continued. This is the Section 15.3 environmental-failure case,
  not an invariant failure.
- The prescribed seam metric unexpectedly passed before the game fix. The agents did not weaken
  its thresholds; they recorded the deviation and used the frozen baseline as a non-regression
  ceiling.
- The D1 live game gate returned 0/8 at both parent `67798ae` and patch `a0427ff`, while its
  unchanged self-test passed 8/8. `gate:p9` likewise failed identically before and after. These
  are benchmark/harness product findings, not PfTerminal control-plane failures.

Outputs and invariant audit:

- Benchmark commits: `67798ae`, `a0427ff`, `13bfdbb`, `d7aa344`, `e387b38`, and evidence
  preservation commit `63c88ea`.
- Independent Kimi review:
  `artifacts/session-29/review/review.md` on branch `qual/native-orch-q1e-20260725`.
- No crash, `turn/start` failure, execution-slot rejection, headless write, status divergence,
  duplicate local application, provider mismatch, stillborn pane, unbounded retry loop, or
  automatic `unknown_outcome` replay appeared in the structured log.
- Burzum's closeout reported no lost or duplicated accepted message and preserved all four worker
  thread IDs. The only reported control-plane-impacting event was the isolated, accurately
  classified Fable authentication failure.
- The two 377 MiB comparison worktrees were removed as soon as their paired proof completed.
  Root filesystem free space remained 107 GiB.

Disposition:

- Session 1 counts on the exact rebuilt candidate.
- Session 2 must not use `/spawn` to create its crew. It must use native `spawn_agent` runtime
  overrides to create a custom crew spanning at least three provider families.

## Phase 8L — Count reset: Anthropic API native follow-up exposed assistant-prefill history

Status: INVARIANT FAILURE; qualification count reset from 1 of 3 to 0 of 3.

Timestamp: 2026-07-25T14:50:42Z

Failed candidate and session:

- Exact binary: `/tmp/pfterminal-native-orch-a786872cc`.
- SHA-256:
  `0ec2cc672677257aa1686fb102926b0ab91f69315f905b31e72f280f95520569`.
- Source commit: `a786872cc`.
- Fresh copied home: `/tmp/pft-native-orch-q2-home-20260725`.
- Structured log: `/tmp/pft-native-orch-q2-logs-20260725/codex-tui.log`.
- Root thread: `019f99bb-3085-76b3-8e22-d5d6e0f72c32`.
- Native Fable thread: `019f99bc-4301-7250-936d-23a1545862f0`.
- Session interval before stop: `2026-07-25T14:43:28Z` through approximately
  `2026-07-25T14:51:30Z`.

Observed failure:

- The custom crew was created exclusively through native `spawn_agent` runtime overrides:
  Anthropic Opus 5, Anthropic Fable 5, Kimi K3, OpenRouter Grok 4.5, and OpenAI Terra.
- Provider logs confirmed the requested models on live requests. The root remained addressable
  under six-slot saturation and applied one natural reprioritization without cancelling workers.
- After Fable emitted a tool call followed by assistant commentary, the tool result was persisted
  and the same native agent received a follow-up. Anthropic API rejected the request twice with
  `invalid_request_error: This model does not support assistant message prefill. The conversation
  must end with a user message.`
- This was not an injected fault or provider outage. Durable history was converted into a request
  shape the Anthropic API rejected, making the same native Fable identity unable to continue.
  Section 6 lifecycle/delivery and Section 15.3 therefore require a fix and a full count reset.

Root cause and repair:

- `ensure_anthropic_messages_end_with_user_turn` already repaired the exact
  `tool_use -> trailing assistant text -> tool_result-only user turn` shape, but the repair was
  gated by `is_claude_plan`.
- Live evidence proves Anthropic API-key Fable enforces the same message-shape constraint.
- The request-local normalizer now applies this narrow mechanical repair to every Anthropic
  transport. It does not modify durable history, inject provider-specific prompt policy, or alter
  normal histories whose assistant message ends in `tool_use`.
- Added a request-builder regression that recreates the live ResponseItem sequence against the
  Anthropic API-key Fable model and asserts a terminal user continuation after the tool result.

Initial verification:

- `cargo fmt --all -- --check` passes.
- Three focused history-shape tests pass.
- The full Fable request-builder regression
  `anthropic_fable_request_repairs_live_tool_then_commentary_history_shape` passes.

Disposition:

- Build a new immutable candidate after the broader Anthropic client suite passes.
- Prove the same Fable native thread can perform a tool call, accept a follow-up, and continue
  without prefill rejection before restarting the three 45–60 minute sessions.

## Phase 8M — Rebuilt candidate passes the live native Fable continuation seam

Status: targeted live seam PASS; long-session qualification remains 0 of 3.

Timestamp: 2026-07-25T15:01:14Z

Candidate:

- Exact binary: `/tmp/pfterminal-native-orch-b07ee141e`.
- SHA-256:
  `74a20e9ffe75bf299b4e135134988e935ba0c96a2ba490c5d8993293bc9df408`.
- Source commit: `b07ee141e2`.
- Fresh copied home: `/tmp/pft-native-orch-fable-seam-home-20260725`.
- Structured log:
  `/tmp/pft-native-orch-fable-seam-logs-20260725/codex-tui.log`.

Live proof:

- A Sol root used native `spawn_agent`, not `/spawn`, to create
  `/root/fable_history_probe` on thread
  `019f99ca-fb7d-7d61-95e8-1d00bd17bf94`.
- The persisted exact runtime was `anthropic / claude-fable-5 / high`; live provider requests
  reported `claude-fable-5` and returned HTTP 200.
- The first turn executed multiple shell tools and completed with `FABLE_FIRST_OK`.
- Native `followup_task` targeted the same agent path and thread. The follow-up executed another
  shell tool and completed with `FABLE_FOLLOWUP_OK`.
- The log contains no assistant-prefill or “conversation must end with a user message” error.
  No replacement agent, provider substitution, or new thread was used.

Disposition:

- The exact live seam that invalidated `a786872cc` is repaired on `b07ee141e2`.
- Restart the full three-session Section 15.3 count on the immutable candidate above.

## Phase 8N — Qualification restart session 1

Status: PASS; qualification count is 1 of 3 on the rebuilt immutable candidate.

Session interval: 2026-07-25T15:03:38Z through 2026-07-25T15:49:06Z
(45 minutes 28 seconds).

Candidate and isolation:

- Exact binary: `/tmp/pfterminal-native-orch-b07ee141e`.
- SHA-256:
  `74a20e9ffe75bf299b4e135134988e935ba0c96a2ba490c5d8993293bc9df408`.
- Source commit: `b07ee141e2`.
- Fresh copied home: `/tmp/pft-native-orch-r1-b07ee-home-20260725`.
- Fresh game worktree:
  `/tmp/isometric-native-orch-r1-b07ee-20260725`.
- Structured log:
  `/tmp/pft-native-orch-r1-b07ee-logs-20260725/codex-tui.log`.
- Root thread: `019f99cd-af34-7fc3-8662-49f098e4f414`.
- Injected-fault mode:
  `PFTERMINAL_INJECT_SPAWN_AGENT_FAILURES=1`.

Live heterogeneous topology:

- Standard `/spawn` created one Fable Plan Nazgul, one Sol Troll, and Luna, Grok 4.5,
  and Terra Orcs. The first `thread/spawnAgent` request was deliberately failed. The
  bounded retry created exactly one crew; no duplicate or stillborn worker appeared.
- Fable Plan:
  `019f99ce-75c9-7730-8a0b-1eb67651bbcc`.
- Sol:
  `019f99ce-7689-7f90-bb97-d3713f29b456`.
- Luna:
  `019f99ce-7754-7252-8141-259a97d6031d`.
- Grok 4.5:
  `019f99ce-7905-7e42-863b-457850f31699`.
- Terra:
  `019f99ce-7825-7ea3-bd2c-88d50f901386`.
- The root then used native `spawn_agent`, not `/spawn`, to add three exact-runtime
  independent reviewers:
  - Kimi Code K3/high, Hypatia, thread
    `019f99d0-1764-7283-8c7f-e022e4ec1e5f`.
  - Anthropic API Fable 5/high, Popper, thread
    `019f99d9-14f5-72c2-bbdc-d4c1a663d8fb`.
  - Anthropic API Opus 5/xhigh, Sagan, thread
    `019f99ed-8451-7ad1-bf0b-dca0c929a7a7`.
- Provider traffic returned HTTP 200 and reported the exact requested Kimi K3,
  Anthropic Fable 5, Anthropic Opus 5, and OpenRouter Grok 4.5 models. There was no
  silent provider or model substitution.

Free-form lifecycle coverage:

- All three Orc lanes were concurrently active under the Troll while the root remained
  addressable. The user naturally reprioritized work while the topology was saturated;
  the hierarchy accepted and propagated the change without cancelling workers.
- The root directly interacted with or followed up the Troll, every Orc, Kimi, Fable,
  and Opus workers.
- Burzum was interrupted and resumed on the same thread before completing its manager
  review.
- The root queued `/compact` while busy, interrupted at a natural boundary, and compacted.
  Durable proof is in
  `/tmp/pft-native-orch-r1-b07ee-home-20260725/sessions/2026/07/25/rollout-2026-07-25T15-03-41-019f99cd-af34-7fc3-8662-49f098e4f414.jsonl`,
  which records `type:"compacted"` and `context_compacted` at
  `2026-07-25T15:27:30Z`.
- The PfTerminal process was then stopped and the exact root was resumed with the exact
  candidate, copied home, and game worktree. It recovered the same Burzum, Ghash, Snaga,
  Krimp, Kimi, and Fable thread IDs and persisted runtime tuples without recreating them.
- Fresh post-restart Kimi and Fable requests returned successful provider traffic on their
  original threads. The repaired Anthropic Fable thread performed tool-bearing work,
  accepted same-thread follow-up, and continued after the process restart without an
  assistant-prefill rejection.
- Sagan completed an independent exact Anthropic Opus 5 review after multiple shell-tool
  calls and returned PASS on the frozen benchmark commit.

Benchmark result:

- Real objective: improve the isometric game's keyboard accessibility, semantic controls,
  loot interaction, combat feedback, focus management, and reduced-motion behavior.
- Frozen benchmark commit:
  `92083cb runtime: harden accessible interaction and motion`.
- The game worktree was clean at close.
- Final manager and Opus gates passed: syntax, world validation, math, interaction
  contract, combat feedback, the browser interaction gate across both worlds and both
  motion modes, and `gate:p3`.
- Hypatia and Popper produced useful earlier red reviews. Their reported focus, motion,
  shell-recursion, and combat-setup defects were subsequently repaired, but those two
  reviewers did not issue new final PASS verdicts. This record does not claim otherwise.
- The complete legacy `npm run gates` suite was not rerun. Earlier `g4` and `g9` magenta
  pixel probes remained known pre-existing benchmark baseline failures and are not claimed
  green.
- Transient benchmark evidence was preserved outside Git at
  `/tmp/ember-campaign-evidence-8zMAbv`.

Fault and invariant disposition:

- Expected injected fault: the first `thread/spawnAgent` request. It was bounded and did
  not duplicate work.
- Environmental auth failure: the standard Fable Plan credential lacked a usable refresh
  token. Its bounded attempts failed accurately and in isolation. Native Anthropic API
  Fable remained live and proved the required provider path.
- Two root request-shape errors were rejected before mutation and corrected once: malformed
  Kimi tool arguments, and a Fable full-history spawn combined with an explicit runtime
  override.
- No `turn/start` failure, assistant-prefill rejection, compaction-trigger ordering error,
  crash, panic, headless write, stillborn pane, duplicate local apply, provider mismatch,
  unbounded retry loop, status divergence, or automatic `unknown_outcome` replay appeared.
- The TUI remained alive through the 45-minute floor and stopped cleanly at
  `2026-07-25T15:49:06Z`.
- Root filesystem free space at close was 101 GiB.

Disposition:

- Session 1 passes and counts as 1 of 3.
- Session 2 must use a fresh copied home and custom native `spawn_agent` crew only, with no
  `/spawn` crew creation, on this exact binary and hash.

## Phase 8O — Native runtime discovery failure and generalized repair

Status: INVARIANT FAILURE; qualification count reset from 1 of 3 to 0 of 3.

Failed candidate session:

- Exact binary: `/tmp/pfterminal-native-orch-b07ee141e`.
- SHA-256:
  `74a20e9ffe75bf299b4e135134988e935ba0c96a2ba490c5d8993293bc9df408`.
- Source commit: `b07ee141e2`.
- Session interval: `2026-07-25T15:51:27Z` through
  `2026-07-25T15:54:11Z`.
- Fresh copied home: `/tmp/pft-native-orch-r2-b07ee-home-20260725`.
- Fresh game worktree:
  `/tmp/isometric-native-orch-r2-b07ee-20260725`.
- Structured log:
  `/tmp/pft-native-orch-r2-b07ee-logs-20260725/codex-tui.log`.
- Root thread: `019f99f9-6b6d-7112-8400-c20082b5dbd1`.

Observed failure:

- The plain objective required a custom crew created only through native `spawn_agent`,
  spanning Anthropic Opus 5, Anthropic Fable 5, Kimi Code K3, OpenRouter Grok 4.5, and an
  inherited Sol worker.
- The native tool contract exposed only the first five picker-visible model summaries and did
  not pair model slugs with provider IDs. Kimi, Grok, Opus, and Fable were absent from that
  bounded description.
- The Sol driver correctly recognized that Kimi and Grok were not exposed, then guessed
  `anthropic / claude-opus-5-plan`. Validation rejected it because plan models belong to the
  separate `claude-plan` provider. It next guessed provider `claude-code`, which is a display
  concept rather than a registered provider ID and was also rejected.
- Both failures occurred before mutation, so no child was created and the injected provider
  fault did not fire. The backend could run the requested exact tuples, as session 1 proved,
  but an uncoached native driver could not discover the identifiers needed to request them.
- This violates the provider-neutral runtime-selection acceptance bar. A backend-only ability
  that depends on the test operator supplying hidden provider/model strings is not a usable
  native orchestration product.

Generalized repair:

- Added `canonical_catalog_provider` to the shared provider registry. It maps every shipped
  picker-visible catalog route to one canonical known-good provider while explicitly allowing
  gateways and user-defined providers to serve additional routes.
- The native tool planner annotates model presets from that shared registry before constructing
  the `spawn_agent` contract. This repairs the discovery boundary without embedding game
  prompts or the four qualification examples into agent instructions.
- Replaced the five verbose model summaries with a compact list of up to 32 exact
  `model_provider / model` tuples plus supported efforts and service tiers. All 23 currently
  picker-visible bundled models fit under the bound.
- The TUI model picker now reuses the same canonical registry instead of maintaining a second
  provider-routing table.
- Added generalized coverage for catalog ownership, unknown/private models, compact exact
  runtime formatting, reasoning-effort truncation, visibility, and the bounded list.

Automated verification:

- `cargo fmt --all -- --check`: PASS.
- `cargo test -p codex-model-provider-info`: PASS, 49 tests.
- `cargo test -p codex-core spawn_agent_tool_ --lib`: PASS, 5 tests.
- `cargo test -p codex-tui model_provider_for_selection --lib`: PASS.
- `cargo check -p codex-tui`: PASS without code warnings.

Disposition:

- Commit the repair, build a new immutable candidate, and prove that a fresh Sol driver can
  discover and create the requested four external runtimes without operator-provided provider
  IDs.
- Restart all three 45–60 minute sessions on that rebuilt candidate.

## Phase 8P — Rebuilt runtime-discovery seam passes live

Status: PASS; qualification count remains 0 of 3 pending the long sessions.

Timestamp: 2026-07-25T16:20:00Z

Candidate:

- Source commit: `640c658f4` (`fix: expose exact native agent runtimes`).
- Immutable binary: `/tmp/pfterminal-native-orch-640c658f4`.
- SHA-256:
  `568c3d5398dd847eae0b81cc1f3824432049b63bdd200d79cc5c092e9692d4b6`.
- Structured log:
  `/tmp/pft-native-orch-discovery5-640c-logs-20260725/codex-tui.log`.
- Fresh isolated home:
  `/tmp/pft-native-orch-discovery5-640c-home-20260725`.
- Root thread: `019f9a11-fb5a-78c0-9ed0-c4309d87d43d`.

Live test:

- The Sol root received only human-facing runtime names: Anthropic API Opus 5, Anthropic API
  Fable 5, Kimi Code K3, and OpenRouter Grok 4.5. The prompt explicitly prohibited source or
  configuration inspection, runtime substitution, and `/spawn`; it did not provide provider
  IDs or model slugs.
- From the native `spawn_agent` contract, the root independently selected and successfully
  created these exact tuples with `fork_turns="none"`:
  - `anthropic / claude-opus-5`
  - `anthropic / claude-fable-5`
  - `kimi-code / k3`
  - `openrouter / x-ai/grok-4.5`
- All four children completed a real provider-backed read-only UTC probe. The final native
  `list_agents` snapshot showed all four as completed.
- The exact provider/model pairs are present in the structured `spawn_agent` tool calls and
  accepted tool results. Opus and Fable also self-reported their exact identities. Kimi
  self-reported its Kimi runtime; Grok intentionally found no provider/model shell environment
  variables, so those two identities are established from persisted native runtime requests
  and provider traffic rather than task labels or shell variables.

Qualification harness notes:

- Two preliminary attempts correctly created the exact native records but could not execute
  provider requests because an empty isolated home did not receive provider environment
  credentials. A copied Claude Plan configuration also lacked its non-portable OAuth refresh
  token. Neither attempt changed product code or counts as a product failure.
- The passing attempt used a fresh isolated OpenAI-authenticated home and injected Anthropic,
  Kimi, and OpenRouter credentials at process start through the canonical read-only vault
  auth-helper. Credential values were neither printed nor copied into the isolated home.
- This is the same credential boundary used by the prior live qualification. It separates
  product runtime selection from test-environment authentication while still exercising real
  provider traffic.

Disposition:

- The discovery defect from Phase 8O is repaired on the new immutable candidate.
- Start the three counted 45–60 minute sessions at zero on this exact binary and hash.

## Phase 8Q — Counted free-form session 1: custom native crew

Status: PASS; qualification count is 1 of 3.

Session:

- Coverage class: Section 15.3 item 2, custom crew with at least three provider families.
  The qualification order is intentionally non-sequential; the standard-crew and
  implementer-blind sessions remain.
- Interval: `2026-07-25T16:23:00Z` through `2026-07-25T17:10:14Z`
  (47 minutes).
- Exact source candidate: `640c658f4`.
- Exact binary: `/tmp/pfterminal-native-orch-640c658f4`.
- SHA-256:
  `568c3d5398dd847eae0b81cc1f3824432049b63bdd200d79cc5c092e9692d4b6`.
- Fresh copied home:
  `/tmp/pft-native-orch-640c-s1-home-20260725`.
- Fresh benchmark worktree:
  `/tmp/isometric-native-orch-640c-s1-20260725`.
- Structured log:
  `/tmp/pft-native-orch-640c-s1-logs-20260725/codex-tui.log`.
- Root thread:
  `019f9a15-c476-7813-a2c9-4ddc28a863cf`.

Native topology:

- The root received only the plain engineering objective and human runtime names. It used
  native `spawn_agent` and related native collaboration tools exclusively; `/spawn` was not
  used.
- It independently created five `fork_turns="none"` workers and saturated all six execution
  slots with the root:
  - Maxwell, thread `019f9a16-a711-7302-a9f5-e50c14e7c175`,
    `anthropic / claude-opus-5`.
  - Parfit, thread `019f9a16-c110-7cc0-be42-66520f5f53e4`,
    `anthropic / claude-fable-5`.
  - Leibniz, thread `019f9a16-db02-7e31-a073-9da23f67da16`,
    `kimi-code / k3`.
  - Hegel, thread `019f9a16-f911-7b21-affa-f85616b79a17`,
    `openrouter / x-ai/grok-4.5`.
  - Archimedes, thread `019f9a17-1170-71b0-b062-0fb8e5d73cfa`,
    inherited `openai / gpt-5.6-sol`.
- Real provider traffic returned successfully on every tuple. After restart and again after
  compaction, structured `get_model_info` and turn spans on these same thread IDs proved the
  exact five models; generic persona text from two workers was not treated as runtime
  authority.

Free-form coverage:

- The root remained addressable while root plus five workers saturated capacity. It accepted
  natural user steering while the five investigations were active.
- Rapid follow-up bursts, follow-ups at turn boundaries, direct conversations with multiple
  workers, and multiple independent review rounds all completed on the original threads.
- Kimi's overlong investigation was interrupted and resumed on the same thread. A later
  in-flight fog implementation was also interrupted at close; its one partial file edit was
  removed and no fog work was committed.
- Natural reprioritization moved the campaign from responsive rendering to the independently
  discovered keyboard inventory failure. A later five-way ranking chose fog over formation
  for a future cycle, then stopped without retaining that unrequested second implementation.
- A full process restart occurred while five amended-diff reviews were active. Resume
  recovered the same root and all five worker identities. Each interrupted review resumed on
  its original thread and returned a final disposition; no replacement workers were created.
- `PFTERMINAL_INJECT_TURN_START_FAILURES=1` injected one retryable first-attempt
  `turn/start` failure immediately after restart. The TUI visibly reported the fault,
  recovered through the bounded retry path, and continued without duplicate execution.
- Explicit `/compact` completed. The rollout records `type:"compacted"` and
  `context_compacted`; all five original threads were subsequently addressable and executed
  on their original runtime tuples.

Benchmark result:

- Real objective: adversarially inspect and improve the running isometric game using a custom
  mixed-model native crew.
- Commit:
  `d6a2ab21dadea82394de0995c69bedb9773d06a1 fix(runtime): stabilize responsive canvas and inventory keyboard`.
- The first repair removed anamorphic canvas scaling. Its browser regression covers eight
  viewport/DPR shapes, real pointer-to-tile round trips, exact 638-by-479 floor boundaries,
  and pathological-aspect pixel budgeting.
- The reprioritized repair gives the inventory full modal key ownership, clears pre-held
  camera keys, makes all 23 slots reachable through 92 deterministic transitions at desktop
  and narrow layouts, supports real Enter and Space activation, and renders/clears visible
  invalid-action feedback.
- Opus requested boundary changes rather than rubber-stamping the first green tests. Fable
  then found dead `invalidFlash` presentation. Both findings were repaired, independently
  rerun, and closed by Opus, Fable, Grok, Kimi, and the inherited Sol test owner.
- Final world validation, math, responsive browser, inventory browser, syntax, and diff
  integrity checks passed. Tracked files and staging were clean at close; only the deliberate
  untracked `node_modules` symlink remained.

Invariant and finding disposition:

- No crash, panic, duplicate local application, status divergence, stillborn pane, headless
  write, notification/retry loop, assistant-prefill failure, compaction-trigger ordering
  error, wrong-provider execution, or automatic `unknown_outcome` replay appeared.
- One interrupted Kimi turn hit SQLite `BUSY_SNAPSHOT` while recording the provider-request
  failure result. The guard released the lease, and the same Kimi thread acquired its next
  lease 129 ms later. No accepted message, result, or runtime was lost. Record as a P1
  telemetry/persistence hardening finding, not an invariant failure.
- One provider stream disconnected once and recovered on the bounded first retry.
- Opus emitted three malformed floating-point tool-call arguments early in the session. The
  router rejected them before mutation, the repetition stopped, and the worker completed its
  assignment. Record as a P2 model/tool-quality finding.
- Plugin-manifest length/icon warnings, missing model-personality metadata warnings, and
  missing analytics-context warnings were noisy but did not alter control-plane state.
- Known benchmark findings remain separate: legacy P4 front-occluder gate failure, visually
  ineffective fog, unbounded formation-slot scatter, and the architectural `world.json`
  loading gap.
- Root filesystem free space at close was 88 GiB.

Disposition:

- Session 1 passes and counts as 1 of 3.
- Run the standard heterogeneous crew session and the implementer-blind session next, each
  with a fresh copied home and worktree on this exact binary and hash.

## Phase 8R — Counted free-form session 2: standard heterogeneous crew

Status: PASS; qualification count is 2 of 3.

Session:

- Coverage class: Section 15.3 item 1, the standard heterogeneous crew.
- Interval: `2026-07-25T17:15:57Z` through `2026-07-25T18:12:18Z`
  (56 minutes).
- Exact source candidate: `640c658f4`.
- Exact binary: `/tmp/pfterminal-native-orch-640c658f4`.
- SHA-256:
  `568c3d5398dd847eae0b81cc1f3824432049b63bdd200d79cc5c092e9692d4b6`.
- Fresh copied home:
  `/tmp/pft-native-orch-640c-s2-home-20260725`.
- Fresh benchmark worktree:
  `/tmp/tih-native-orch-640c-s2-20260725`.
- Structured log:
  `/tmp/pft-native-orch-q2-logs-20260725/codex-tui.log`.
- Root thread:
  `019f9a47-1525-7f71-89c8-dfba801757e5`.

Native topology:

- The root used the ordinary standard-crew control. The persisted native crew was:
  - Angmar, thread `019f9a47-b7ab-70a0-bc7d-784aa57e538d`,
    `anthropic / claude-fable-5-plan`.
  - Burzum, thread `019f9a47-b846-74a1-991d-f33df484d1c9`,
    `openai / gpt-5.6-sol`.
  - Snaga, thread `019f9a47-b907-7e21-891d-a6aef3cab618`,
    `openai / gpt-5.6-luna`.
  - Ghash, thread `019f9a47-b9e5-7b92-8d54-c9708277fb27`,
    `openai / gpt-5.6-terra`.
  - Krimp, thread `019f9a47-babb-72e3-b685-27434da7b309`,
    `openrouter / x-ai/grok-4.5`.
- Fable Plan could not authenticate because the copied qualification home did not contain a
  portable Claude Code OAuth refresh token. The UI surfaced that exact corrective action.
  The original Angmar identity was retained and never silently changed to another runtime.
- The root briefly created an unauthorized replacement CTO,
  `019f9a49-773b-7751-a037-27d7ec4ccd6d` on `gpt-5.4`. Operator steering
  stopped it before hierarchy work and required preservation of the original failed Fable
  identity. This is model behavior caught by the product's addressable control plane, not a
  runtime substitution by the host.

Free-form coverage:

- Real objective: improve the text-improvement harness while preserving cross-project
  isolation and its existing CLI contract.
- All three Orcs worked concurrently on non-overlapping audits. The root naturally
  reprioritized after the first implementation broke historical `round --reuse` behavior,
  reopened acceptance, and sent the same Snaga thread through two evidence-driven rework
  rounds.
- Rapid two-message steering, turn-boundary follow-ups, direct conversations with Burzum,
  Snaga, Ghash, and Krimp, and a same-thread follow-up after an older report all completed in
  order.
- The process was killed at `17:42:53Z` while Snaga's rework was active. Resume on the same
  home and candidate restored the exact root and worker IDs; Burzum resumed on its original
  thread.
- Explicit `/compact` completed in about 61 seconds on the exact root. `/goal resume`
  preserved the post-compaction campaign state.
- An explicit operator interruption preserved state.
- All four available working crew turns were occupied with read-only work. The root remained
  responsive under saturation and messaged Burzum successfully without creating a new
  thread.
- `PFTERMINAL_INJECT_SPAWN_AGENT_FAILURES=1` caused the initial standard-crew creation fault
  and bounded recovery. After restart, `PFTERMINAL_INJECT_TURN_START_FAILURES=1` caused one
  attempt-one `turn/start` failure at `17:57:47Z`; attempt two succeeded without duplicate
  application.

Strict provider-fault proof:

- The session resumed before its 60-minute ceiling with a local OpenAI-chat-compatible
  qualification provider configured as `qual-fault / qualification-fault-model`. A native
  worker was created once as Linnaeus, thread
  `019f9a75-dfeb-71d0-93e7-03bfdc8eb9fd`.
- The controlled provider returned one HTTP 429 with `Retry-After: 1` and request ID
  `qual-injected-429`. The worker reported the retryable provider failure to its parent; the
  TUI, root, and worker identity remained alive.
- A later native `followup_task` reused the same worker and provider tuple. The provider
  returned valid streamed chat output, and the worker reported:
  `Provider retry recovered; native worker is responsive.`
- Fixture event evidence is
  `/tmp/pft-qualification-fault-provider-events.jsonl`; the exact fixture source is
  `/tmp/pft_qualification_fault_provider.py`.
- Two fixture bring-up attempts failed before reaching the intended HTTP seam: the first
  fixture process was not persistently supervised, and the first success response used a
  non-streaming JSON body where the client required chat SSE. Both failures were visible,
  did not mutate the benchmark, and were corrected in the qualification fixture rather than
  product code. They do not count as product recovery evidence; only the subsequent
  recorded 429 and same-worker streamed recovery count.

Benchmark result:

- Commit:
  `928d4b2cfa0b74a635b78ed897fc9179813be7f1 Fix project-scoped evaluation reuse`.
- Eleven files changed, with 353 insertions and 72 deletions. The repair enforces
  project-scoped score uniqueness, reads, cache reuse, migration, and exact historical
  run-index materialization.
- Focused tests passed 7 of 7; the full suite passed 44 of 44; compile and diff-integrity
  checks passed.
- A real v2-to-v3 migration retained IDs `7`, `11`, and `19`, preserved the named index, and
  retained colliding alpha/beta rows independently.
- Ghash independently reviewed the final rework. The root caught both an arbitrary zip
  pairing defect and a single-source-group reuse defect before accepting it.
- Phase 2 OpenRouter streaming findings remained read-only and deferred at
  `/tmp/ghash_sse_contract.txt`, `/tmp/krimp_stream_readonly.txt`, and
  `/tmp/snaga_stream_readonly.txt`.
- Worktree and index were clean at close.

Invariant and finding disposition:

- No panic, fatal signal, stack overflow, compaction-trigger ordering failure, assistant
  prefill failure, `unknown_outcome`, headless write, stillborn pane, duplicate local
  application, status divergence, or unbounded notification/retry loop appeared.
- The watcher exited silently early. The uncovered interval `17:18:04Z` through
  `17:23:08Z` was manually audited against the same signatures before the watcher was
  restarted. Its final alert file contained only the expected injected attempt-one
  `turn/start` failure.
- Three parent checkpoints to unavailable Angmar produced 54 duplicated ERROR telemetry rows
  for three user-visible Fable auth failures. Record as a P1 telemetry-fanout defect; it did
  not duplicate a task, retry invisibly, or change the selected provider.
- Eleven model/tool validation errors were bounded and non-mutating: invalid full-history
  runtime override requests, waits without children, one timeout below the minimum, two
  stale patch contexts, and three missing wait cell IDs.
- A model-policy false positive on adversarial terminology paused the campaign. Neutral
  clarification plus `/goal resume` preserved state. Record as an external model-policy
  finding.
- Root filesystem free space at close was 87 GiB.

Disposition:

- Session 2 passes and counts as 2 of 3.
- Run the implementer-blind session next on a fresh copied home and worktree using this exact
  candidate binary and hash. Its driver receives only a plain user objective and ordinary
  product help; it receives no implementation details, evidence tables, or defect list.

## Phase 8S — Structural audit invalidates the candidate

Status: FAIL; qualification count is reset from 2 of 3 to 0 of 3.

The exact candidate remained process-stable through the third, implementer-blind session,
including resume, explicit compaction, one injected `turn/start` failure, a controlled HTTP
429 followed by same-thread recovery, saturation, interruption, and preserved Opus 5 and
Kimi K3 follow-ups. That evidence does not override the structural invariant below.

Release-blocking duplicate terminal delivery:

- Core already forwards each native child completion through the durable native mailbox as
  one `AgentMessageKind::TerminalResult` with a stable `completion:<thread>:<turn>` message
  identity. The two production paths are
  `codex-rs/core/src/agent/control.rs::maybe_start_completion_watcher` and
  `codex-rs/core/src/session/mod.rs::forward_child_completion_to_parent`.
- The TUI separately observes the same native `TurnCompleted`, calls
  `record_spawn_child_report_for_thread`, stores a second queue in
  `spawn_pending_reports_by_thread`, and later calls
  `flush_pending_reports_for_thread`. That path creates a new
  `SubmitSpawnAgentTask`/`Assignment` with a different `dispatch-*` identity and embeds the
  same terminal result as a `child_report`.
- This is a second model-visible delivery mechanism, not a display-only projection. In the
  standard-crew session, Burzum's rollout at
  `/tmp/pft-native-orch-640c-s2-home-20260725/sessions/2026/07/25/rollout-2026-07-25T17-16-59-019f9a47-b846-74a1-991d-f33df484d1c9.jsonl`
  contains Ghash's native `terminal_result` at line 223 with a `completion:*` identity, then
  the same result again at line 229 as an `assignment` with a `dispatch-*` identity.
- The standard-crew rollouts contain 48 `child_report;` payloads in addition to the native
  final-answer terminal messages. The implementer-blind custom agents did not exhibit the
  duplicate only because the TUI's second path is restricted to persisted crew mappings.

Release-blocking authority gap:

- `AgentClass::{CrewMember, EphemeralTask}` exists in
  `codex-rs/protocol/src/crew.rs`, but no runtime, thread metadata, persistence, resume, or
  lifecycle path uses it.
- `/spawn` creates native app-server threads, then keeps crew classification and completion
  wake-up behavior solely in TUI `CrewInstanceState`. Native core therefore cannot decide
  whether one terminal mailbox event should wake a persistent manager, and the TUI compensates
  by injecting the duplicate assignment described above.

Required repair:

1. Persist `AgentClass` on native child thread metadata and preserve it across resume.
2. Make the single native `TerminalResult` mailbox event the only native completion
   transport. For crew-member parents, that native event must request exactly one bounded
   manager turn; ephemeral task behavior remains ordinary Codex behavior.
3. Delete the TUI's native report queue, injection, and flush path. Keep any external-Claude
   compatibility adapter isolated to non-native panes and display native state as a
   projection only.
4. Add integration regressions for idle-parent completion, busy-parent completion,
   redelivery deduplication, restart/resume, and absence of a generated `child_report`
   assignment.
5. Rebuild the exact candidate and restart all three 45–60 minute sessions. No session on
   source `640c658f4` counts after this finding.

## Phase 8T — Native completion authority repair

Status: implementation and focused automated gates pass; live qualification remains 0 of 3.

As of `2026-07-25T19:44:03Z`:

- `AgentClass` is persisted in `SubAgentSource::ThreadSpawn`, accepted by app-server
  `thread/spawnAgent`, attached to every `CrewSpec` member, exported in the stable schema,
  and preserved from the stored rollout across native resume.
- Native V2 tool-created descendants are classified as `EphemeralTask`; the `/spawn`
  `CrewSpec` path uses the exact crew ID and logical member ID as `CrewMember`.
- Core's existing stable `completion:<child-thread>:<turn>` `TerminalResult` is now the only
  native completion transport. A persistent `CrewMember` parent requests a manager turn from
  that mailbox item; an unclassified human root or ephemeral task agent remains queue-only.
- The TUI's native `spawn_pending_reports_by_thread`,
  `record_spawn_child_report_for_thread`, and `flush_pending_reports_for_thread` machinery
  has been deleted. Native `TurnCompleted` is now projection-only.
- External Claude panes remain an edge adapter. Their reports enter the canonical app-server
  mailbox with a stable SHA-256-derived ID and never create a second synthetic assignment
  queue.

Passing focused evidence:

- `cargo check -p codex-core -p codex-app-server -p codex-tui --tests`.
- `crew_child_terminal_result_uses_one_triggering_native_mailbox_message`: one pass. The test
  emits the same child completion twice and observes exactly one triggering native
  `TerminalResult`.
- `resume_thread_subagent_restores_stored_metadata_and_effective_multi_agent_mode`: one pass,
  including restored `CrewMember` class and manager auto-processing policy.
- `native_turn_completion_is_projection_only_and_never_schedules_tui_delivery`: one pass.
- `external_child_report_enters_native_parent_through_one_canonical_mailbox_message`: one
  pass, including duplicate edge input and rejection of the obsolete
  `SubmitSpawnAgentTask` route.
- `thread_agent_message_uses_native_mailbox_and_deduplicates_stable_id`: one app-server
  integration pass with an explicit persisted `CrewMember` class.
- App-server protocol schema fixtures: four of four pass after normal stable-surface
  regeneration. Experimental fixture churn was deliberately discarded.
- Structural search finds zero remaining references to the deleted native TUI queue,
  recorder, or flush symbols.

Disk:

- Focused test linking reduced root free space from 78 GiB to 49 GiB.
- `cargo clean --profile dev` reclaimed 90.8 GiB from this worktree only; no user worktree,
  session, or rental was touched.
- Schema generation and its focused test left 125 GiB free. A release build may proceed
  while retaining the 60 GiB reserve.

Qualification disposition:

- The old `640c658f4` sessions remain historical defect evidence and do not count.
- No repaired candidate session has started yet. Build one exact candidate from the repaired
  source, record its source commit and binary SHA-256, then restart the three-session count.

## Phase 8U — Repaired-candidate session 1 reset: root mailbox recovery was skipped

Status: INVARIANT FAILURE; qualification remains 0 of 3.

Session:

- Interval: successful credential-backed process from `2026-07-25T20:02:29Z` through
  `2026-07-25T20:50:01Z` (47 minutes 32 seconds).
- Source commit: `205588630` (`fix: unify native crew completion delivery`).
- Immutable binary: `/tmp/pfterminal-native-orch-205588630`.
- Binary SHA-256:
  `e089f87da8629ab4386e49a0643fe3f61b73f4d64c153644de7a00aab172b43f`.
- Fresh copied home:
  `/tmp/pft-native-orch-205588630-s1-home-20260725`.
- Fresh benchmark worktree:
  `/tmp/tih-native-orch-205588630-s1-20260725`.
- Root thread:
  `019f9ada-93e6-7473-834d-0920fb82064d`.

Useful live evidence:

- Native `spawn_agent` created and repeatedly reused the exact required runtimes:
  `anthropic / claude-opus-5`, `anthropic / claude-fable-5`,
  `openrouter / x-ai/grok-4.5`, and `kimi-code / k3`.
- The tree reached the effective native thread limit. Two additional spawns failed cleanly
  with `agent thread limit reached`; the manager remained addressable, listed the tree, and
  queued control-plane mail to an existing worker.
- A process kill and exact-binary resume preserved every original native identity and runtime.
- One injected `turn/start` failure recovered on the bounded retry.
- A one-shot local provider returned HTTP 429 once. The same
  `qual-fault / qualification-fault-model` thread recovered on exactly one follow-up; provider
  evidence is `/tmp/pft-native-orch-205588630-s1-fault-events.jsonl`.
- Explicit compaction persisted `compacted` and `context_compacted` events. The compacted root
  subsequently reused Opus, Fable, Grok, and Kimi on their original threads.
- The root mailbox rollout contains 33 terminal-result records with 33 unique stable IDs and
  zero legacy `child_report;` payloads.
- The real text-improvement-harness objective produced commits `5f0c02e` and `75ec154`, 71
  warning-clean tests, two passing 32-seed stress reruns, and a clean Git object graph.
- The external watcher heartbeat remained live after repair of the qualification harness; its
  alert file stayed empty. A full post-session structured-log and rollout signature scan found
  no panic, fatal signal, compaction-order error, assistant-prefill failure, notification loop,
  stillborn pane, or automatic unknown-outcome replay.

Release-blocking finding:

- The final `pfterminal_state_5.sqlite` audit found eight root-addressed mailbox rows still in
  `provider_running` after the deliberate process restart and 16 quarantined
  `unknown_outcome` rows. All 74 message IDs and all 41 assignment IDs were unique, so this
  was not duplicate delivery.
- `AgentControl::recover_inter_agent_communications` correctly quarantines `submitting` and
  `provider_running` work, but its only production caller is the lazy child reload path in
  `codex-rs/core/src/agent/control/spawn.rs`.
- Resuming the human root thread does not invoke mailbox recovery for that root. Root-addressed
  ambiguous rows can therefore remain `provider_running` indefinitely instead of entering
  `unknown_outcome`, producing persisted lifecycle/status divergence.
- The session does not count despite its otherwise successful live coverage. Repair the
  root-resume boundary, add a regression that resumes a root with `provider_running` mail and
  proves quarantine without replay, rebuild an immutable candidate, and restart all three
  sessions at zero.

## Phase 8V — Root-resume recovery repair and adjacent native lifecycle audit

Status: PASS; repaired source is ready for a new immutable candidate.

Repair:

- `ThreadManager::resume_thread_with_history` now calls
  `AgentControl::recover_inter_agent_communications` after restoring the persisted subtree.
  Child lazy reload already performed this reconciliation; the human root now crosses the
  same recovery boundary.
- A root-resume regression persists a root-addressed mailbox item through
  `ready -> submitting -> submitted -> provider_running`, shuts down the original manager,
  resumes from the rollout and same SQLite state, and proves:
  `provider_running -> unknown_outcome` and no application of the ambiguous message ID to
  resumed history.
- The broad native V2 suite exposed an adjacent unloaded-worker defect. `interrupt_agent`
  resolved the persisted identity and correctly returned `Unloaded`, but then attempted to
  deliver a live control message to the nonresident thread and converted the valid no-op into
  an error. Live audit-message delivery is now skipped only for `Unloaded` and `NotFound`;
  the persisted worker remains visible and its open spawn edge remains unchanged.
- Three stale native-spawn assertions now enforce the intended stable non-empty nickname
  result rather than expecting nickname metadata to be absent.

Passing evidence:

- `resumed_root_quarantines_provider_running_mail_without_replay`: one pass.
- Adjacent recovery tests: five of five pass
  (`resumed_root_restores_open_descendants_as_unloaded_with_exact_runtime`,
  `resumed_subagent_rejoins_loaded_parent_control_plane`,
  `durable_agent_mailbox_deduplicates_and_completes_after_rollout_flush`,
  `ensure_v2_agent_loaded_reloads_registered_unloaded_agent`, and
  `resume_thread_subagent_restores_stored_metadata_and_effective_multi_agent_mode`).
- Native V2 filtered suite: 71 of 71 pass after fixing the unloaded interrupt boundary and
  aligning stable-identity assertions.
- `cargo check -p codex-core -p codex-app-server -p codex-tui --tests`: pass.
- `just fmt`: pass.
- Root filesystem free space after the dev compile gate: 103 GiB.

Qualification disposition:

- Count remains 0 of 3. No evidence from candidate `205588630` is promoted.
- Commit this repair, build one exact binary from that commit, run a short copied-home replay
  against the old ambiguous mailbox as seam proof, then begin three fresh 45–60 minute
  sessions on the same immutable binary.
