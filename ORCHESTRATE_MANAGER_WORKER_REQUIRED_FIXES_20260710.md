# Required Fixes: Manager/Worker Assignments

**Date:** 2026-07-10
**Base:** `integrate/pfterminal-20260707` at `292cea31e` (`feat(tui): add manager worker assignments`)
**Source review:** `ORCHESTRATE_MANAGER_WORKER_CODE_REVIEW_FINDINGS_20260710.md` (verdict:
approve with required fixes)
**Rule:** Fixes 1-3 are release blockers for the assignment feature. Fixes 4-5 ship in the same
PR. Fixes 6-8 are cleanup and may follow, but 6 is strongly recommended now (it is user-visible
in the first minute of use). One commit per fix, each with its regression tests.

All paths relative to `codex-rs/`. Line numbers at `292cea31e`.

---

## Fix 1 (P1, blocker): Marker detection must not trigger on the Manager's own acknowledgment

**Problem.** `pause_matching_whips_on_stop_marker` substring-scans Manager output with no phase
filter: `output.contains(DEFAULT_STOP_MARKER)` (`tui/src/orchestrate.rs:2403`) and
`assignment_blocked_reason` via `line.split_once("ASSIGNMENT_BLOCKED:")` (`:2412`, `:2861`).
The birth brief instructs the Manager about both markers, so a first reply like "I will emit
WHIP_DONE when the work is complete" flips the assignment to terminal `Done` **during
Drafting**. "I'll report ASSIGNMENT_BLOCKED: <reason> if stuck" false-enters `Blocked`.

**Required behavior.**

1. A marker counts only when it is deliberate: trimmed output line equals `WHIP_DONE`, or
   trimmed line starts with `ASSIGNMENT_BLOCKED:` followed by a non-empty reason on that line.
   Substring matches inside prose must not count.
2. `Done` and `Blocked` transitions apply only while the assignment is `Executing`. Drafting
   acknowledgments and Blocked-state chatter must never re-trigger.
3. The birth brief (`assignment_birth_brief`, `:2817`) and mandate text
   (`assignment_mandate_task`, `:2795`) instruct: "When you emit a marker, place it alone on its
   own line." Keep marker mentions in the brief phrased so they never satisfy rule 1 (they
   currently do not, once line-anchored — verify).
4. Legacy nudge stop-marker behavior is unchanged (its substring semantics are pre-existing and
   its pause is recoverable).

**Tests (required).**

- Drafting Manager reply containing "I will emit WHIP_DONE when complete." → phase stays
  Drafting.
- Executing Manager reply with prose mentions of both markers → no transition; reply with
  `WHIP_DONE` alone on a line → Done; `ASSIGNMENT_BLOCKED: missing API key` on its own line →
  Blocked with that reason; `ASSIGNMENT_BLOCKED:` with empty reason → no transition.
- Legacy whip stop-marker substring test still passes unchanged.

---

## Fix 2 (P1, blocker): Native pane loss must pause and notify, never silently stall

**Problem.** Three gaps let an Executing assignment stop with zero signal:

1. `fire_destination_for_node` (`tui/src/orchestrate.rs:2682`) returns
   `Ok(Native(thread_id))` for any `thread:` node without checking the thread exists, so
   `audit_restored_assignments` (`:1760`) can never flag a missing native Manager/Worker after
   restart.
2. A native Worker absent from `agent_navigation` makes `target_node_is_idle` (`:2716`) return
   false forever; sweep planning errors are swallowed → no mandate, no notification.
3. The `ThreadClosed` path (`tui/src/app/thread_routing.rs:1591-1603`) marks the thread closed
   but never calls `note_assignment_node_gone`; only collab `Shutdown`/`NotFound` (`:1714`) is
   hooked.

**Required behavior.**

1. `fire_destination_for_node` validates native threads (present in `agent_navigation` or
   `thread_event_channels`, not closed) and errors otherwise, so the restart audit detects
   missing native panes exactly as it does Claude panes.
2. `ThreadClosed` handling and `mark_agent_picker_thread_closed`
   (`tui/src/app/session_lifecycle.rs:192`) call `note_assignment_node_gone` for the closed
   thread's node.
3. Sweep watchdog: an `Executing` assignment whose Worker or Manager has been unresolvable or
   never-idle for **4 consecutive cadence windows** is paused with a visible reason
   ("Worker unreachable"), reusing the existing pause+notify path. This is the backstop for any
   loss path not explicitly hooked.
4. Restored panes that are valid must not be falsely paused (existing behavior preserved).

**Tests (required).**

- Restart fixture with an Executing assignment whose native Worker thread is absent → Paused +
  "Worker is unavailable" notice; same for missing native Manager; valid native + Claude panes
  restore un-paused.
- `ThreadClosed` for Worker mid-Executing → Paused + notice naming the Worker.
- Deterministic-clock sweep test: Worker permanently non-idle/unresolvable → paused with reason
  after 4 cadences, exactly one notification.

---

## Fix 3 (P1, blocker): Codex Main must not be offered as Manager while it cannot fulfill the contract

**Problem.** The Manager picker (`open_orchestrate_manager_picker`, candidates from
`orchestrate_target_entries`, `tui/src/orchestrate.rs:1518`) offers the primary thread. The
primary thread emits no collab-agent state changes, so for a Codex-Main Manager: markers are
never scanned, failed-turn backoff never applies, and `is_running` is never maintained
(`ordered_path_backed_subagent_threads` excludes the primary;
`tui/src/app/agent_navigation.rs:251`) — mandates inject into the user's running main chat and
`WHIP_DONE` is ignored until expiry.

**Required behavior.**

1. Exclude the primary thread (and the `codex-main` alias) from Manager candidates in the
   guided picker, with no gap when Codex Main is the only pane (the "Create Manager pane" row
   remains).
2. `attach_whip` rejects assignment attaches whose resolved holder is the primary thread node,
   with a clear error ("Codex Main cannot be an assignment Manager; create a Manager pane"),
   covering typed commands and agent-origin blocks.
3. Worker = Codex Main remains allowed but must stop lying about idleness: gate mandate planning
   for a primary-thread Worker on the chat widget's actual running state (or the primary
   thread's live-channel `active_turn_id`), so mandates never claim "Worker stopped" mid-turn.
   If this gating is not practical in this pass, mandates for primary-thread Workers must be
   suppressed and the confirm screen must say so — do not ship the false-premise behavior.

**Tests (required).**

- Manager picker snapshot with only Codex Main open: exactly one candidate row ("Create Manager
  pane").
- Typed `attach codex-main <spec> --holder codex-main`-style and agent-origin attempts → error.
- Worker=Codex Main with the primary thread mid-turn (simulated running state) → mandate plan
  returns "not idle"; after turn completes → mandate fires.

---

## Fix 4 (P2, same PR): No orphaned state on partial attach/create failure

**Problem.** `attach_whip` deletes replaced assignments and inserts+persists the new whip
_before_ `inject_assignment_birth_brief(&id)?` can fail (`tui/src/orchestrate.rs:1993` and the
function tail), leaving a brief-less assignment presented as ready and the old assignment
destroyed. `CreateOrchestrateManager` (`tui/src/app/event_dispatch.rs`, handler added in this
commit) can spawn a Manager pane and then fail attach, leaving a phantom pane.

**Required behavior.**

1. Validate the Manager destination (`fire_destination_for_node`) and read the spec _before_
   mutating the registry; only then remove replaced assignments, insert, persist, and inject.
2. If brief injection still fails, roll back the insert and restore the replaced assignments,
   or mark the new assignment `Paused` with reason "brief not delivered" — never silent
   Drafting-as-ready.
3. If `CreateOrchestrateManager` succeeds at spawn but attach fails, tell the user explicitly
   that the pane exists without an assignment ("Manager pane <name> created but not bound:
   <err>") so it is not a mystery pane.

**Tests:** brief-injection failure (missing Claude pane manager) → old assignment intact, no
new assignment (or Paused-with-reason per chosen strategy); create-then-attach-failure → error
message names the orphan pane.

---

## Fix 5 (P2, same PR): Drafting assignments must show their real duration

**Problem.** Status and detail views render expiry from `whip.expires_at`, which is `None`
until execution starts, so a 15-minute assignment shows `unlimited` while Drafting (observed in
PTY). The truth lives in `execution_duration_s`.

**Required behavior.** Pre-execution phases render the deferred budget: `15m after start`,
`8 hours after start`, or `unlimited` only when `execution_duration_s` is `None`. Status row
(`open_orchestrate_status_view`) and details (`open_orchestrate_whip_details`) both fixed.

**Tests:** snapshot of a Drafting row with an 8h duration showing `8 hours after start`;
unlimited assignment still shows `unlimited`.

---

## Fix 6 (P3, strongly recommended now): Assignment-aware messages and phase-aware actions

- `set_whip_state_by_ref` prints `Whip whip-1 detached.` (`tui/src/orchestrate.rs:2067`) and
  `mark_whip_terminal` prints `Whip {id} ...` (`:2665`) for assignments → use
  "Assignment {id} ended/paused/expired ...".
- Detail view: hide "Mandate now" and "Test" while Drafting (both currently error), or make Test
  preview the mandate regardless of phase. Hide "Start execution" outside Drafting (verify).
- Plan-gating errors surfaced on manual actions say "Whip {id} is inside cooldown" → assignment
  variants.
- `whip_suffix_for_target` (`:679`) shows raw node ids → use `node_label`.

**Tests:** detach/expiry message text for assignments; Drafting detail-view snapshot without the
dead actions.

## Fix 7 (P3): User-yield window uses base cadence

`plan_whip_fire_for_generation` compares user activity against
`assignment_effective_cadence_s` (up to 2h under backoff). Use `whip.cooldown_s` (base cadence)
for the user-yield check so a user message never delays automation by more than 15 minutes.
Test: backoff level 3 + user turn 20 minutes ago → mandate allowed (cadence permitting).

## Fix 8 (P3): Reserve the `draft-with-manager` spec name

`attach_whip` special-cases `DRAFT_WITH_MANAGER_SPEC` (`tui/src/orchestrate.rs:1941`) but
mandate planning still calls `read_whip_instruction` with it, so a user doc named
`draft-with-manager.md` silently becomes the spec. Bypass doc lookup for the sentinel everywhere
and reject the name in `validate_whip_name`/the save flow. Test: doc named `draft-with-manager`
present → mandate task still says "drafted with Manager"; saving a spec under that name is
refused.

---

## Verification gates for the fix PR

1. Per-fix regression tests above, plus the still-owed items from the implementation plan:
   a restart lifecycle test (kickoff → restart mid-Executing → resume notice → one-cadence
   delay → loop continues) and one persistence-failure injection proving visible degradation.
2. Focused suites green: `cargo test -p codex-tui --lib assignment`, `orchestrate_`, `whip`;
   `cargo fmt --check`; no new clippy findings (the pre-existing `redundant clone` error at
   `claude_panes/app_integration.rs:149` is tracked separately — do not fold it in here).
3. Real PTY (shortened cadence via test hook or config): lock a spec, observe one real
   Executing-phase mandate land in the Manager pane and a dispatch reach the Worker; then
   Manager emits line-anchored `WHIP_DONE` → assignment Done, mandates stop. Record commit SHA,
   binary path, and tmux environment in the PR.
4. No jargon regression: rendered guided-flow/status/detail strings contain no
   `whip|holder|target|review` after Fix 6 (excluding typed power-command help).
5. PR description records: marker anchoring decision, the Codex-Main-Worker choice made in
   Fix 3.3, and confirmation that legacy nudge suites pass unchanged.
