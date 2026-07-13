# Implementation Plan: Conversational Manager/Worker Orchestration

**Date:** 2026-07-10
**Spec:** `ORCHESTRATE_MANAGER_WORKER_SPEC_20260710.md` (v2, overnight-flagship revision)
**Base:** `integrate/pfterminal-20260707` at `0420104aa`
**Shape:** One PR, five commits, each independently green. Deterministic-clock work lands first
because every later gate depends on it.

All paths relative to `codex-rs/`. Primary module: `tui/src/orchestrate.rs`.

---

## Commit 1 — Clock injection + assignment data model (no behavior change)

### Steps

1. **Deterministic clock.** Orchestrate logic currently calls `Utc::now()` directly
   (~15 sites in `orchestrate.rs`, plus restore in `app.rs`). Add a clock source on `App`
   (e.g. `orchestrate_now: fn() -> DateTime<Utc>` or a small `Clock` trait object, test-settable
   via `app/test_support.rs`) and route every orchestrate call site through it. The 45s sweep
   stays real-time; only decision logic reads the injectable clock.
2. **Assignment model.** Extend `Whip` (`orchestrate.rs:97`) with a serde-defaulted
   `kind` field:
   - `WhipKind::LegacyNudge` (default; today's auto/review behavior).
   - `WhipKind::Assignment { phase: Drafting|Executing|Blocked{reason}|Done,
     execution_started_utc: Option<DateTime<Utc>>, last_user_turn_utc: Option<DateTime<Utc>>,
     failure_backoff_level: u8 }`.
   Manager = existing `holder` (required for assignments), Worker = existing `target`.
3. Persistence: fields flow through the existing pane-layout snapshot
   (`claude_panes/pane.rs:157`, `app_integration.rs:352`, restore in `app.rs:1305`). Old
   layouts deserialize as `LegacyNudge` (serde default).
4. No caller constructs an `Assignment` yet.

### Tests (gate)

- Round-trip test: assignment whip with every phase persists and restores intact; legacy
  layout JSON (fixture from current format) restores as `LegacyNudge` with unchanged behavior.
- Clock injection test: a whip expiry decision responds to the test clock, not wall time.
- Full existing orchestrate/whip suites pass unchanged (`cargo test -p codex-tui --lib
  orchestrate_`, `whip`); zero snapshot diffs. Clippy + fmt clean.

---

## Commit 2 — Assignment mandate gating, backoff, blocked/stop scanning

### Steps

1. **Gating branch** in `plan_whip_fire_for_generation` (`orchestrate.rs:~1700`): assignments
   skip the max-fires check, the `last_idle_generation_fired` gate, and are exempt from
   `note_whip_holder_idle` ignore-pausing (`orchestrate.rs:~1900`). Fire conditions per spec §4:
   Executing + worker idle + manager idle + `now - last_fire ≥ effective_cadence` +
   `now - last_user_turn ≥ cadence`. Base cadence 900s, stored on the whip (`cooldown_s`).
2. **Failure backoff (§4.1):** repoint `consecutive_failed_turns` at the **Manager** node for
   assignments (`pause_spinning_whips_on_failed_turn`, `orchestrate.rs:~1878`): failures double
   effective cadence (15m→30m→60m→2h cap) via `failure_backoff_level`; success resets; posts a
   notification at 3 consecutive failures; never pauses. Worker-side empty-output and
   failed-turn pauses are skipped entirely for assignments.
3. **Marker scans move to the Manager** for assignments: extend the idle-output scan
   (`pause_matching_whips_on_stop_marker` pattern, `orchestrate.rs:~1810`) to watch the
   Manager's output for `WHIP_DONE` → phase Done, and `ASSIGNMENT_BLOCKED: <reason>` → phase
   Blocked with captured reason + notification. Worker output is not scanned for assignments.
4. **User-activity tracking:** stamp `last_user_turn_utc` on assignments whose Manager (and
   Worker, per spec open question 2 — recommended yes) receives a **user-initiated** turn:
   native submits and Claude pane prompts (`claude_panes/app_integration.rs:615` area and the
   native submit path; the `spawn_operator_input_seen` sites mark where operator input is
   already distinguished). Mandate/whip injections and agent dispatches must NOT stamp it.
5. Mandate task builder (short nudge per spec §4) replacing `review_whip_task` for assignments;
   fire destination is always the Manager.

### Tests (gate; deterministic clock throughout)

- Cadence: no double-mandate inside 15m; worker-idle edge mandates immediately when both
  clocks are satisfied; mandate blocked while manager running.
- User precedence: user turn on Manager (and Worker) delays next mandate a full cadence;
  injected mandate does not reset the user clock.
- No-stop guarantees: >20 cycles without exhaustion; manager-reviews-without-dispatch is
  re-mandated next cadence (no idle-generation deadlock, no ignore-pause).
- Backoff: failure doubling, 2h cap, reset on success, notification at 3, no pause.
- Markers: Manager `WHIP_DONE` → Done; `ASSIGNMENT_BLOCKED: x` → Blocked with reason; Worker
  emitting either marker has no effect on an assignment.
- Legacy nudges: all existing gating tests still pass (behavior untouched).

---

## Commit 3 — Lifecycle transitions, briefs, restart

### Steps

1. **Birth brief** builder (spec §3: role, phase contract, execution contract, overnight
   autonomy + progress-log clause, cadence disclosure, inline instructions + path). Injected as
   first task for created Managers, immediate injection for bound Managers.
2. **Drafting → Executing:** hook `note_whip_holder_dispatched` (`orchestrate.rs:1379`; callers
   `spawn_orchestration.rs:1303,1390`) — first Manager→Worker dispatch flips phase and stamps
   `execution_started_utc`; expiry becomes `execution_started_utc + duration` (store the chosen
   duration, resolve expiry at execution start). Add `/orchestrate start <id>` (parser at
   `orchestrate.rs:260`, user-origin only) as the explicit override.
3. **Drafting fires nothing:** gating from Commit 2 already requires Executing; verify sweep
   and edge paths both respect it.
4. **Blocked auto-resume:** user turn to the Manager, or Manager dispatch to the Worker, flips
   Blocked → Executing (reuses the user-activity and holder-dispatch hooks).
5. **Pane-loss pauses:** on pane/thread close of Manager or Worker, assignment → Paused +
   notification (hook the existing pane close/registry removal paths in
   `claude_panes/app_integration.rs` and thread close handling in `app/thread_routing.rs`).
6. **Restart (spec §7):** on restore, Executing assignments post a resume notice and set the
   next-mandate clock to one full cadence from startup (supersedes the fix-4 fresh-edge gate
   for assignments; the gate remains for `LegacyNudge`); Drafting restores silently; missing
   Manager/Worker → Paused + notify.

### Tests (gate)

- One test per §2 lifecycle table row (Done, Expired, user pause/detach, Worker gone, Manager
  gone, failure backoff).
- Auto-arm on first dispatch (phase flip + duration clock start) and `/orchestrate start`.
- Drafting: simulated 1h+ of drafting with idle Worker produces zero mandates.
- Blocked → user reply auto-resume; Blocked → Manager dispatch auto-resume.
- Restart matrix: Executing (resume notice + one-cadence delay), Drafting (silent), missing
  pane (Paused + notify); legacy whips keep fix-4 fresh-edge behavior.

---

## Commit 4 — Guided flow, Manager creation, status UX

### Steps

1. **Setup flow (spec §5):** extend the picker chain (`open_orchestrate_target_picker` →
   duration → instructions, `orchestrate.rs:863-1010`): relabel to Worker/Assignment terms,
   add `Draft with manager` to the instructions step, add the **Manager step**
   (Create manager pane | Bind existing pane — reuse the pane-picker pattern, excluding the
   Worker). Confirm screen shows Manager → Worker, duration, spec source.
2. **Manager creation:** `CreateSpawnAgent` with `native_spawn_default_model()` and the brief
   as initial task; bind-existing injects the brief immediately. Guided flow always creates
   Assignments; `--mode auto` typed commands remain LegacyNudge; typed review-mode attach maps
   to an Assignment.
3. **Status UX (spec §6):** status rows, detail view (talk-to-manager focus action, start
   execution during Drafting, mandate now, pause/resume, extend, test, detach), pane suffixes
   (`whip_suffix_for_target`, `orchestrate.rs:638`: `managed-by <M>` / `managing <W>; <phase>;
   next mandate <t>`). No whip/holder/target/review strings anywhere user-visible in the
   guided flow or status.

### Tests (gate)

- Snapshots: each guided step (Worker, duration, instructions incl. `Draft with manager`,
  Manager step, confirm), status view per phase, detail view, both pane suffixes.
- Interaction: full guided flow → `CreateSpawnAgent` payload + brief content asserted; bind
  path asserts immediate brief injection; Worker excluded from Manager candidates; self-manage
  rejected; 1:1 replace/block semantics.
- Jargon lint test: rendered guided-flow and status strings contain no
  `whip|holder|target|review` (case-insensitive), excluding typed power-command help.

---

## Commit 5 — Flagship overnight simulation + integration + PTY evidence

### Steps & required tests (all gates)

1. **Overnight simulation (spec §9.6a), the flagship gate:** deterministic-clock integration
   test — 8h Executing assignment, zero user activity, ≥15 mandate/dispatch cycles; two
   injected transient Manager turn failures mid-run → backoff then recovery; "morning" user
   status question answered without disrupting cadence; run ends Done or Expired, **never
   Paused**. Run the matrix twice: native Worker and Claude-pane Worker.
2. **End-to-end lifecycle test** (closes the architecture-review gap): kickoff → draft →
   auto-arm → cycles → restart mid-execution → resume → expiry, with persistence-failure
   injection at one transition proving visible degradation rather than silent state loss.
3. Full suites: `cargo test -p codex-tui --lib`, clippy `-D warnings` on touched code, fmt,
   `git diff --check`, no pending snapshots.
4. **Real-PTY pass (tmux), recorded in the PR:** create assignment end-to-end with a real
   Claude/Codex Worker; watch one real mandate fire after a shortened cadence (test hook or
   temporary config); ask the Manager a status question mid-execution; `ASSIGNMENT_BLOCKED`
   round-trip; detach cleans up. Keyboard responsiveness throughout (vault-freeze lesson).

### Exit criteria

- All gates green in CI; overnight sim and lifecycle test attached to the PR by commit SHA.
- PR description records: spec open-question resolutions (cadence flag, Worker-turn timer
  reset, blocked marker format, manager model default), the PTY checklist with results, and
  confirmation legacy whip behavior is unchanged.

## Out of scope (do not touch)

- `WhipRegistry` extraction / effects refactor (architecture-review follow-up debt; this plan
  stays within the existing `orchestrate.rs` + `App` structure).
- Spawn tabbed model picker for the Manager step (separate spec; use
  `native_spawn_default_model()` now).
- Nazgul/Troll/Orc crew semantics, `pfterminal_send_task` dispatch internals, whip-doc storage
  format (`~/.pfterminal/whips`).
- Multi-worker managers (data model permits later; 1:1 enforced now).
