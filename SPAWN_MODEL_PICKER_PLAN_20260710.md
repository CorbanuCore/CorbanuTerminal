# Implementation Plan: Tabbed Model-and-Effort Picker for /spawn

**Date:** 2026-07-10
**Spec:** `SPAWN_MODEL_PICKER_SPEC_20260710.md`
**Base:** `integrate/pfterminal-20260707` at `0420104aa`
**Shape:** One PR, three commits, each independently green.

All paths relative to `codex-rs/`.

---

## Commit 1 — Thread `ModelSelectionPurpose` through the picker (no behavior change)

### Steps

1. Define in `tui/src/chatwidget/model_popups.rs`:
   ```rust
   #[derive(Debug, Clone, PartialEq, Eq)]
   pub(crate) enum ModelSelectionPurpose {
       Session,
       SpawnAgent { role: SpawnRole, parent_node_id: Option<String> },
   }
   ```
   Plain data only (crosses the event channel); derive what `AppEvent` requires.
2. Add `purpose` to `AppEvent::OpenReasoningPopup` (`tui/src/app_event.rs:1104`) and its
   dispatch arm (`tui/src/app/event_dispatch.rs:1141`).
3. Change signatures, passing `Session` at every existing call site:
   - `open_all_models_popup(presets, purpose)` (`model_popups.rs:332`)
   - `model_picker_item(preset, purpose)` (`model_popups.rs:419`) — embeds purpose in the
     `OpenReasoningPopup` action.
   - `open_reasoning_popup(preset, purpose)` (`model_popups.rs:672`).
4. Inside `open_reasoning_popup`, factor the terminal action into a match on purpose. `Session`
   keeps the exact existing path (`apply_model_and_effort`, plan-scope prompt). Leave the
   `SpawnAgent` arm `unimplemented-in-behavior` (send nothing yet is NOT acceptable — implement
   it in Commit 2; in this commit the arm can simply not be constructible because no caller
   passes it).

### Tests (gate for this commit)

- `cargo test -p codex-tui --lib model_` and the model-popup snapshot tests pass with **zero
  snapshot diffs** — proves pure refactor.
- `cargo test -p codex-tui --lib spawn` passes untouched.
- `cargo clippy -p codex-tui` and `just fmt` clean.

---

## Commit 2 — Wire /spawn into the tabbed picker

### Steps

1. Rewrite `open_spawn_model_picker(role, parent_node_id)` (`tui/src/spawn_orchestration.rs:1008`)
   as a thin wrapper: fetch presets via `model_catalog().try_list_models()`, call
   `open_all_models_popup(presets, SpawnAgent { role, parent_node_id })`.
2. In `open_all_models_popup`, branch on purpose for:
   - Subtitle/header: role-labelled, e.g. "Codex Nazgul pane - <provider subtitle>".
   - Initial tab + `is_current` row: from `native_spawn_default_model()`
     (`spawn_orchestration.rs:2440`) instead of `current_model()`.
   - Provider-tab filter: only groups valid for native spawn (resolve spec open question #1;
     verify `CreateSpawnAgent` works per provider group, especially `claude-plan`, before
     deciding the filter set — record the finding in the PR description).
3. In `open_reasoning_popup` for `SpawnAgent`:
   - Highlight `spawn_reasoning_effort_for_role(role, &preset)` (`spawn_orchestration.rs:5476`).
   - On confirm (and on the ≤1-effort direct path), send `AppEvent::CreateSpawnAgent { role,
     parent_node_id, agent_nickname: None, model, provider:
     model_provider_for_selection(&model), effort }` and dismiss the stack
     (`dismiss_parent_on_child_accept` like the /model flow).
   - Never call `apply_model_and_effort`; never trigger
     `should_prompt_plan_mode_reasoning_scope`.
4. Delete the flat-list construction; remove `spawn_model_item` (`spawn_orchestration.rs:5443`)
   if now unused.

### Tests (gate for this commit; new tests are required, not optional)

Add beside the existing tests in `model_popups.rs` / `spawn_orchestration.rs` / `app/tests.rs`:

1. **Snapshot: spawn tabbed picker** — ≥2 provider tabs, role-labelled subtitle, spawn-default
   model's tab active and row marked current.
2. **Snapshot: spawn effort popup** — role-default effort highlighted (XHigh where supported).
3. **Interaction: full create path** — select model → select effort → assert exactly one
   `CreateSpawnAgent` with the expected `role`, `parent_node_id`, `model`, `provider`, `effort`;
   assert the view stack is dismissed.
4. **Interaction: single-effort model** — direct create, no effort popup.
5. **Interaction: tab switching** — Left/Right changes tabs; selection lands on the new tab's
   default row.
6. **Session isolation** — after the full spawn flow: `current_model()`, effective reasoning
   effort, and persisted config are unchanged; no plan-scope prompt event was emitted (test in
   plan mode to prove the guard).
7. **Esc path** — Esc from effort popup returns to tabs, Esc again returns to the prior spawn
   menu, zero `CreateSpawnAgent` events.
8. **Role coverage** — repeat test 3 for Troll and Orc (parameterized), confirming
   `parent_node_id` propagation for Orc.
9. **Provider validity** (from spec open question) — for every provider group shown under the
   spawn purpose, a unit/integration test asserting `CreateSpawnAgent` resolves that provider to
   a constructible native pane config; excluded groups asserted absent from the tab list.
10. **/model regression** — existing `/model` snapshots still byte-identical.

---

## Commit 3 — Verification sweep and evidence

### Steps

1. `cargo test -p codex-tui --lib` (full TUI lib suite), `cargo clippy -p codex-tui -- -D
   warnings` on touched code, `just fmt` check, `git diff --check`.
2. Real-PTY manual pass (tmux), recorded in the PR description:
   - `/spawn nazgul` → Create Nazgul pane → tabs render, Left/Right responsive, Enter-Enter
     creates a Nazgul with the default model/effort; pane appears and accepts a task.
   - Repeat once for Troll via `/spawn troll` → Harness: Codex.
   - Esc-out at every level leaves no pane and no state change.
   - `/model` afterward: unchanged behavior, session model intact.
3. Confirm no snapshot files are left pending (`cargo insta pending-snapshots` or equivalent).

### Exit criteria

- All commits green in CI (`pfterminal-ci`, codespell, cargo-deny unaffected).
- PR description contains: the provider-validity finding (which tabs are shown/filtered for
  spawn and why), the PTY checklist with results, and confirmation that `/model` snapshots are
  unchanged.
- No changes outside: `model_popups.rs`, `spawn_orchestration.rs`, `app_event.rs`,
  `event_dispatch.rs`, and test files — anything else needs justification in the PR.

## Out of scope (do not touch)

- `CreateSpawnAgent` handling and pane creation internals.
- Claude harness / `open_spawn_claude_profile_picker`.
- Standard-crew shortcut, role default constants, crew preflight.
- Nazgul bind-existing-pane flow.
