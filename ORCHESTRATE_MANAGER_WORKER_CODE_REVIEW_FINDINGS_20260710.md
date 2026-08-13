# Code Review Findings: Manager/Worker Assignments

**Date:** 2026-07-10
**Reviewed range:** `f20a13186..292cea31e` (one commit: `292cea31e feat(tui): add manager worker
assignments`; 9 files, +1,200/-131)
**Reviewer basis:** full diff read, call-site tracing in the untouched surrounding code, focused
test runs, baseline verification at `f20a13186`, and a real-PTY guided-flow exercise.
**Deliverable per:** `ORCHESTRATE_MANAGER_WORKER_CODE_REVIEW_REQUEST_20260710.md`

## Verdict

**Approve with required fixes.**

The core architecture is faithful to the spec: assignment kind rides the existing whip registry,
legacy gating is cleanly branched, phases/backoff/user-yield are data-driven and deterministic-
clock testable, and the guided flow works end to end in a real PTY with only Codex Main open.
No credential, data-loss, or legacy-regression defect found.

Three P1 findings break the flagship unattended-overnight invariant in reachable configurations
and must be fixed before this feature is promoted: marker false-positives that can kill an
assignment at kickoff (F1), silent stalls on native pane loss/restart (F2), and broken semantics
when Codex Main is bound as Manager (F3).

---

## Findings

### F1 (P1): Substring marker scan lets the Manager's own acknowledgment kill or block the assignment

- **File:** `codex-rs/tui/src/orchestrate.rs:2403` (`output.contains(DEFAULT_STOP_MARKER)`),
  `:2412` + `:2861` (`assignment_blocked_reason` — `line.split_once("ASSIGNMENT_BLOCKED:")`),
  scanned from `pause_matching_whips_on_stop_marker` (`:2367`) with **no phase filter** (`:2393-2400`).
- **Trigger:** create an assignment. The birth brief (`assignment_birth_brief`, `:2817`)
  explicitly instructs: "emit ASSIGNMENT_BLOCKED: <reason> ... emit WHIP_DONE only when...".
  A Manager's first response very commonly restates its contract, e.g. "I will emit WHIP_DONE
  when the work is complete." When the Manager goes idle, that output is substring-scanned →
  phase flips to `Done` **during Drafting**, terminally. Likewise "I'll report
  ASSIGNMENT_BLOCKED: <reason> if stuck" false-enters `Blocked { reason: "<reason> if stuck" }`.
- **Actual:** assignment dies or blocks from an acknowledgment; overnight run never starts or
  silently ends. **Expected:** only a deliberate marker emission changes phase.
- **Why tests miss it:** `assignment_overnight_loop_survives_cycles_backoff_and_manager_markers`
  and the blocked-reason unit test feed clean, intentional marker strings; no test feeds a
  contract-restating Manager reply, and none scans during Drafting.
- **Repair:** anchor detection to a line that _is_ the marker (trimmed line == `WHIP_DONE`;
  trimmed line starts with `ASSIGNMENT_BLOCKED:`), restrict Done/Blocked transitions to
  `Executing` (Drafting acknowledgments are the highest-risk turn), and amend the brief to say
  "place the marker alone on its own line". Legacy nudges have the same substring behavior, but
  their pause is recoverable; assignment `Done` is terminal, hence P1.

### F2 (P1): Native pane loss is undetectable — Executing assignments stall silently

Three compounding gaps, one invariant broken ("must not silently stop"; "pane loss pauses and
reports"; "missing pane detected at restart"):

1. `fire_destination_for_node` (`orchestrate.rs:2682`) returns `Ok(Native(thread_id))` for any
   `thread:` node **without checking the thread exists**. Therefore
   `audit_restored_assignments` (`:1760`) can never flag a missing _native_ Manager or Worker
   after restart — only Claude panes are validated.
2. If a native Worker fails to restore into `agent_navigation`, `target_node_is_idle` (`:2716`)
   returns `false` forever; sweep-planning errors are swallowed (`evaluate_whips_for_target`
   ignores non-manual errors) → no mandate, no notification, armed assignment forever.
3. The `ThreadClosed` path (`app/thread_routing.rs:1591-1603`) marks the thread closed but never
   calls `note_assignment_node_gone`; the new hook only fires on collab `Shutdown`/`NotFound`
   status changes (`thread_routing.rs:1714`). A worker closed via `ThreadClosed` leaves the
   assignment armed and silently stalled (`is_closed` ⇒ never idle ⇒ never mandated).

- **Trigger:** 8h overnight run; native Worker thread is closed or the app restarts and the
  Worker thread does not re-attach. The loop stops with zero user-visible signal — the exact
  failure the feature exists to prevent.
- **Why tests miss it:** tests drive Claude-pane destinations and pre-seeded
  `agent_navigation` entries; no restart-with-missing-native or `ThreadClosed` coverage.
- **Repair (smallest):** (a) validate native existence in `fire_destination_for_node` via
  `agent_navigation`/`thread_event_channels`; (b) call `note_assignment_node_gone` from the
  `ThreadClosed` handler and `mark_agent_picker_thread_closed`; (c) add a sweep-side watchdog:
  an Executing assignment whose Worker or Manager has been unresolvable/never-idle for N
  consecutive cadences pauses with a visible reason.

### F3 (P1): Codex Main is offered as Manager but cannot fulfill the Manager contract

- **File:** Manager candidates come from `orchestrate_target_entries` (`orchestrate.rs:1518`),
  which includes the primary thread; `open_orchestrate_manager_picker` excludes only the Worker.
- **Mechanics:** the primary thread emits no collab-agent state changes, so
  `note_whip_target_idle_with_fire_control` never runs for it → a Codex-Main Manager's
  `WHIP_DONE` / `ASSIGNMENT_BLOCKED:` output is never scanned and failed-turn backoff is never
  applied. Its `agent_navigation` entry's `is_running` is not maintained (only collab paths call
  `set_running`; `ordered_path_backed_subagent_threads` explicitly excludes the primary), so the
  Manager-idle gate passes even mid-turn: mandates are injected into the user's active main chat
  while it is running.
- **Actual:** Manager says done → assignment mandates it every 15 minutes until expiry anyway.
  **Expected:** Done ends the loop; mandates wait for an idle Manager.
- **Worker-side variant (P2 within this finding):** Worker = Codex Main is treated always-idle
  for the same reason → mandates fire while the Worker is mid-turn with a false "Worker stopped
  at <t>" premise. (My PTY run used exactly Worker = Codex Main — the guided flow steers new
  users into this configuration when only Main is open.)
- **Why tests miss it:** all Manager/Worker fixtures are Claude panes or spawn threads with
  synthetic collab events.
- **Repair (smallest):** exclude the primary thread from Manager candidates and reject
  `codex-main` holders for assignments in `attach_whip`; for the Worker side, either wire the
  primary thread's turn start/completion into the idle notes or display an honest caveat and
  gate mandates on `chat_widget` running state.

### F4 (P2): Partial-failure paths orphan state

- **File:** `orchestrate.rs` `attach_whip` (replacement removal at `:1993`, brief injection at
  the tail: `inject_assignment_birth_brief(&id)?` after insert+persist), and
  `app/event_dispatch.rs` `CreateOrchestrateManager` (spawn → attach, no rollback).
- **Trigger:** bind an existing Claude-pane Manager that disappears between picker and confirm →
  `attach_whip` has already **deleted the replaced assignment and inserted+persisted the new
  one** before `inject_assignment_birth_brief` fails; result: brief-less Drafting assignment
  presented as ready, old assignment gone. Conversely, if Manager creation succeeds but attach
  fails (Worker closed meanwhile), a phantom Manager pane remains with no assignment.
- **Expected (request invariants):** replacement must not discard the old assignment before the
  new Manager is valid; Manager spawn failure leaves no partial state.
- **Why tests miss it:** no failure injection between spawn/attach/brief steps.
- **Repair:** resolve and validate the Manager destination _before_ mutating the registry;
  on brief-injection failure, roll back the insert (and restore the replaced assignment) or
  mark the new assignment Paused with an explicit "brief not delivered" reason.

### F5 (P2): Drafting assignments display "unlimited" regardless of the chosen duration

- **File:** status/detail expiry rendering reads `whip.expires_at`
  (`orchestrate.rs` status view `:836+`, details `:1406+`), which is `None` until execution
  starts because the duration is parked in `execution_duration_s`.
- **Trigger (observed in PTY):** guided flow, duration "15 minutes" → status row shows
  `drafting; awaiting execution; unlimited; spec draft-with-manager`.
- **Actual:** user cannot verify their 8h overnight budget stuck. **Expected:** show the deferred
  duration, e.g. `15m after start`.
- **Why tests miss it:** no snapshot of a Drafting assignment row with a bounded duration.
- **Repair:** render `execution_duration_s` when phase is Drafting/pre-execution.

### F6 (P3): Jargon and dead actions leak into assignment UX

- Detach prints `Whip whip-1 detached.` (`orchestrate.rs:2067`) — observed in PTY; violates the
  no-jargon invariant for user-visible assignment flows. `mark_whip_terminal` similarly prints
  "Whip {id} ..." (`:2665`) for assignment expiry/pauses. Plan errors say "Whip {id} is inside
  cooldown."
- Detail view offers "Mandate now" and "Test" for Drafting assignments; both always error with
  "Assignment ... is not executing" (observed). Hide them in Drafting or let Test preview anyway.
- `whip_suffix_for_target` renders raw node ids (`managed-by thread:<uuid>`; `orchestrate.rs:679+`).
- **Repair:** assignment-aware message variants; filter detail actions by phase; use
  `node_label` in suffixes.

### F7 (P3): User-yield window uses the backed-off cadence

- **File:** `orchestrate.rs` plan gating — the user-activity check compares against
  `assignment_effective_cadence_s` (up to 2h under backoff) instead of the base cadence.
- **Effect:** during failure backoff, one user message delays automation by up to 2h rather than
  15m. Spec §4 says user activity resets by one (base) cadence.
- **Repair:** use `whip.cooldown_s` for the user-yield comparison.

### F8 (P3): `draft-with-manager` spec name can collide with a real doc

- `attach_whip` special-cases the name (`orchestrate.rs:1941`), but mandate planning calls
  `read_whip_instruction` with it; a user file `~/.pfterminal/whips/draft-with-manager.md`
  silently becomes the assignment spec. Reserve the name in `validate_whip_name`/save flow or
  route through a sentinel that bypasses doc lookup everywhere.

---

## Test and verification evidence

- Ground truth: branch clean apart from known untracked review docs; `git diff --check` clean;
  HEAD `292cea31e`.
- Focused gates (all pass): `assignment` (4), `orchestrate_` (16), `whip` (10),
  `guided_attach_args_parse_to_attach_command` (1); `cargo fmt --check` clean.
- Broad gates, baseline-verified: full `codex-tui --lib` run hits 3 pre-existing `exec_flow`
  snapshot mismatches, then aborts in a pre-existing stack overflow
  (`app::tests::discard_side_thread_removes_agent_navigation_entry`; **reproduced at base
  `f20a13186`**, passes with `RUST_MIN_STACK=16MiB`). Clippy: 1 pre-existing `redundant clone`
  **error** (`claude_panes/app_integration.rs:149`, blame `cbf00bcaa`, ancestor of base) plus 3
  pre-existing warnings — note the request described this as "three warnings"; it is actually a
  clippy _failure_, pre-existing, tracked separately.
- Left no artifacts: generated `.snap.new` files deleted; no implementation files touched.

### Real PTY evidence

- Binary: `~/repos/PfTerminal-triage-clean/codex-rs/target/debug/pfterminal` built at
  `292cea31e`, launched via `pfterminal-debug --yolo` in tmux (120x40, `orchreview` session).
- Verified: `/orchestrate` renders assignment language; guided flow Worker (Codex Main, sole
  pane, selectable) → Duration (15m) → Spec (`Draft with Manager` default) → Manager (create;
  Codex Main correctly absent from bind list) → assignment `whip-1` created in Drafting; brief
  sent exactly once ("Task sent to Manager"); status and detail views show phase and `Start
execution`; keyboard responsive at every popup; detach cleans the registry ("No assignments").
- Observed defects during the run: F5 ("unlimited" for a 15m assignment), F6 ("Whip whip-1
  detached.", Test-in-Drafting error).
- Not observed live: a real Executing mandate cycle (would require locking a spec and waiting a
  cadence; deterministic-clock coverage exists in `app/tests.rs`, but see F1-F3 for what that
  coverage misses).

## Spec requirements not implemented or not proven

1. No restart lifecycle test (kickoff → restart mid-Executing → resume notice → one-cadence
   delay → continue), and no persistence-failure injection (plan Commit 3/5 gates).
2. Pane-loss coverage is partial (F2): only collab `Shutdown`/`NotFound` is hooked.
3. Live mandate/dispatch cycle not exercised in a real PTY (only Drafting).
4. Spec open questions are not recorded as resolved anywhere (cadence flag: not implemented —
   acceptable per spec default; Worker-turn timer reset: implemented as recommended).

## Residual risk and minimum release gate

- Minimum gate before promotion: fix F1, F2, F3; add a regression test for each (marker-echo
  reply during Drafting; restart with missing native Worker must pause+notify; Codex-Main
  Manager either excluded or fully functional); re-run focused suites plus one real PTY
  Executing-phase mandate observation with a shortened cadence.
- F4/F5 should ride the same fix PR; F6-F8 may follow.
- Residual: marker protocol remains inherently text-based even after anchoring (a Manager
  quoting a marker on its own line still false-triggers); long-term fix is a structured
  end-of-assignment signal, out of scope here.
