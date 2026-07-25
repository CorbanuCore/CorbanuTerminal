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
