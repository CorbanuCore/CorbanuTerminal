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
