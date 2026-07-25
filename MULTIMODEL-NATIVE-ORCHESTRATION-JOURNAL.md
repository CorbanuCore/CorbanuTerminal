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
