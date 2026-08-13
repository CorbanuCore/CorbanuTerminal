# Spec: Use the Tabbed Model-and-Effort Picker in the /spawn Flow

**Date:** 2026-07-10
**Branch:** `integrate/pfterminal-20260707` at `0420104aa`
**Goal:** When creating a Codex-native spawn pane (Nazgul create, and Troll/Orc via the Codex
harness), model selection uses the same clean provider-tabbed "Select Model and Effort" pane as
`/model`, including the reasoning-effort step — instead of the current flat list with a
silently auto-picked effort.

All paths relative to `codex-rs/`. Line numbers at `0420104aa`.

---

## Current behavior

### The clean picker (target UX, already exists for /model)

- `tui/src/chatwidget/model_popups.rs:332` — `open_all_models_popup` groups presets by
  `model_provider_for_selection` into the 9 `MODEL_PICKER_PROVIDER_GROUPS` (line 56: OpenAI,
  Ambient, Z.AI, Claude Plan, Anthropic, Meta, Vercel, Baseten, OpenRouter), renders them as
  `SelectionTab`s with Left/Right switching, sets the initial tab from the current model, and
  shows the "Select Model and Effort" header per tab.
- Item action → `AppEvent::OpenReasoningPopup { model }` (`model_popups.rs:428`) →
  `open_reasoning_popup` (`model_popups.rs:672`) → effort list → `apply_model_and_effort`
  (mutates the **current session**). Single-effort models direct-select and skip the effort step.

### The spawn picker (to be replaced)

- `tui/src/spawn_orchestration.rs:1008` — `open_spawn_model_picker(role, parent_node_id)` builds
  one flat searchable list: a "Codex Native Agent" section with the spawn-default model, then
  "Other Codex Models" with every remaining picker-visible preset.
- Effort is chosen automatically by `spawn_reasoning_effort_for_role`
  (`spawn_orchestration.rs:5476`, prefers XHigh) with no user input; the effort only appears as a
  suffix in the item label.
- Selecting an item immediately sends `AppEvent::CreateSpawnAgent { role, parent_node_id,
agent_nickname: None, model, provider, effort }` (`spawn_orchestration.rs:5461-5470`).
- Entry points that land here (all inherit the new picker automatically):
  - Nazgul → "Create Nazgul pane" (`spawn_orchestration.rs:3487`).
  - Troll/Orc → harness picker → "Harness: Codex" (`spawn_orchestration.rs:980`).

---

## Required behavior

1. `open_spawn_model_picker` presents the identical tabbed provider-grouped view as
   `open_all_models_popup`: same grouping, same tab order, same per-tab headers and subtitles,
   same Left/Right footer hint, same search behavior, same `is_default`-first sorting.
2. Title reflects the spawn context: header title "Select Model and Effort", subtitle includes
   the role, e.g. "Codex Nazgul pane - OpenAI Codex plan" (keep the provider subtitle; prefix or
   suffix the role so the user knows what they are creating).
3. Initial tab and highlighted row come from `native_spawn_default_model()`
   (`spawn_orchestration.rs:2440`), not from the chat session's current model.
4. Selecting a model opens the same reasoning-effort popup, with these spawn-specific rules:
   - The highlighted/default effort is `spawn_reasoning_effort_for_role(role, preset)` (the
     current auto-pick), so Enter-Enter reproduces today's one-shot behavior.
   - Models with ≤1 supported effort skip the effort step (existing `direct_select` behavior).
   - Confirming effort sends `AppEvent::CreateSpawnAgent { role, parent_node_id,
agent_nickname: None, model, provider, effort }` and dismisses the whole picker stack
     (`dismiss_parent_on_child_accept`, as the /model flow already does).
5. Spawn selection must NOT mutate the session: no `apply_model_and_effort`, no config persist,
   and the plan-mode reasoning-scope prompt (`should_prompt_plan_mode_reasoning_scope`,
   `model_popups.rs:551`) must not trigger for spawn selections.
6. Esc behavior: effort popup → back to model tabs → back to the prior spawn menu (existing
   selection-stack semantics).
7. Unaffected paths (explicitly out of scope): the Claude harness / Claude profile picker
   (`open_spawn_claude_profile_picker`), the standard-crew one-shot
   (`STANDARD_NAZGUL_MODEL`/`STANDARD_TROLL_MODEL`/`STANDARD_ORC_MODEL` and
   `ensure_standard_crew_providers_ready`), "Bind existing pane", and the `/model` flow itself.

## Design

Parameterize the existing picker instead of duplicating it. Suggested shape:

- Add `ModelSelectionPurpose` (name flexible):
  - `Session` — current behavior, the default everywhere today.
  - `SpawnAgent { role: SpawnRole, parent_node_id: Option<String> }`.
- Thread the purpose through:
  - `open_all_models_popup(presets, purpose)` — controls title/subtitle, initial tab source,
    `is_current` source, and the item action payload.
  - `AppEvent::OpenReasoningPopup { model, purpose }` (`app_event.rs:1104`,
    `event_dispatch.rs:1141`).
  - `open_reasoning_popup(preset, purpose)` — controls highlighted effort and the terminal
    action: `Session` → existing apply/scope-prompt path; `SpawnAgent` → `CreateSpawnAgent`.
- `open_spawn_model_picker` becomes a thin wrapper: fetch presets, call the shared popup with
  `SpawnAgent` purpose. Delete the flat-list construction and, if then unused,
  `spawn_model_item`.
- `purpose` must be `Clone + Send` plain data (it crosses the event channel); no closures in
  events.

## Open questions to resolve during implementation

1. **Provider validity for native spawn.** `/model` tabs include every provider group, including
   Claude Plan. Verify `CreateSpawnAgent` produces a working native pane for every provider that
   can appear in a tab (especially `claude-plan`). If any provider group cannot back a native
   spawn pane, filter that tab out for the `SpawnAgent` purpose only — do not show creatable-
   looking rows that fail after creation.
2. **Plus-plan rate-limit warning** (`model_popups.rs:680-698`): spawned panes consume the same
   plan quota, so keep the warning for spawn unless it reads misleadingly; cheap either way.

## Acceptance

- `/spawn nazgul` → "Create Nazgul pane" shows the tabbed picker: provider tabs, Left/Right
  switching, spawn-default model's tab preselected and row marked current.
- Choosing a multi-effort model shows the effort popup with the role-default effort highlighted;
  Enter creates the pane with exactly that model/provider/effort in `CreateSpawnAgent`.
- Choosing a single-effort model creates the pane directly.
- After the full flow, the chat session's model, effort, and config are unchanged (test asserts
  no `apply_model_and_effort` side effects and no persist).
- Troll and Orc Codex-harness creation get the same picker with role-appropriate subtitle and
  effort defaults.
- Esc from the effort popup returns to the model tabs; Esc again returns to the prior menu; no
  pane is created.
- Snapshot tests: spawn tabbed picker (at least two tabs), spawn effort popup with role default
  highlighted. Interaction tests: Left/Right tab switch, full create path asserting the
  `CreateSpawnAgent` payload, session-unchanged assertion, plan-scope prompt not triggered.
- Existing `/model` snapshots unchanged (purpose defaults to `Session`).

## Delivery constraints

- One PR; refactor commit (thread `purpose` through with `Session` behavior identical) followed
  by the spawn-wiring commit, so the /model regression surface is reviewable in isolation.
- No changes to `CreateSpawnAgent` handling, role defaults, or crew presets.
- Run the existing model-popup and spawn-orchestration test suites; add the new tests beside the
  existing snapshot/interaction tests in `model_popups.rs` and `spawn_orchestration.rs`.
