# Recovery Spec: Pane Injection That Actually Works

**Date:** 2026-07-11
**State:** working tree DIRTY on top of `6186f5c96` with an in-flight, partially verified fix
(last recorded verification: `VERIFY_EXIT=101`). Live-use qualification: **FAILED.**
**Authority:** This spec supersedes all prior orchestrate specs/plans where they conflict.
The assignee owns execution end to end. There is no human review gate anywhere in this
document; every gate below is a command an agent runs and evidence a file captures.

## 0. The product, restated in one sentence

One pane injects prompts into another pane, on a timer, when the target is idle — reliably,
visibly, and without the user ever discovering breakage that automation could have caught.

Everything else — briefs, phases, markers, cadences — exists to serve that sentence. Any
behavior that cannot be demonstrated working inside the real TUI does not exist.

## 1. Non-negotiable rules (violation = the work is rejected)

1. **"Done" is a reserved word.** It may only be claimed when the full TUI qualification
   matrix (§5) passes at the exact release SHA with artifacts on disk. A unit-test pass, a
   compile, or a happy-path probe is progress, never "done."
2. **Every user-visible workflow must be exercised through the real TUI** (tmux-scripted,
   real binary, real panes) before any claim about it is made. If a path was not driven in
   the TUI, say "untested in TUI" in exactly those words.
3. **No silent scope changes.** If a fix alters what appears in any picker, pane list, or
   status view, the qualification matrix must re-run in full — pickers are product surface.
4. **Claims must name their evidence.** Every assertion in the completion report links a
   matrix row to an artifact path (capture log, JSON state dump, or test name).
5. **No new features.** This is a recovery pass: fix the enumerated defects, build the
   qualification harness, stabilize. Nothing else lands.

## 2. Ground state to establish first (Phase 0)

1. Reconcile the dirty working tree: identify what the in-flight edits (pane-picker filter,
   drafting-brief changes, `is_managed_spawn_crew_thread`) fix versus break; land them as
   reviewed commits or revert them. `VERIFY_EXIT=101` means the tree as-is is unproven —
   nothing carries forward without passing §5.
2. Inventory the full diff before deciding: the dirty tree currently touches **9 files
   including `app-server/src/request_processors/turn_processor.rs`** — an app-server change
   outside the TUI crate that no fix note mentions. Every modified file must be explained in
   the completion report (what it changes, why, which defect it serves) or reverted.
   Unexplained edits do not land.
3. Deal with stale stashes: `stash@{0..2}` are July 2-3 WIP ("unreviewed", "interrupted
   mid-write", "likely discard"). Do not pop them into this work. Record their existence and
   leave them untouched, or drop them if inspection confirms they are dead — either way the
   decision is logged in the report.
4. Reset runtime state before any matrix run: the live `~/.pfterminal` still contains
   pane-layout and assignment state from the failed sessions (orphaned Manager panes,
   detached/broken assignments, the vanished-Worker registry entries). The matrix must run
   against a dedicated fresh `PFTERMINAL_HOME` fixture (plus one dedicated row that restores
   a copy of the polluted layout to prove graceful recovery of the user's real state). Never
   test against, and never destroy, the user's live home — snapshot it read-only for the
   recovery row.
5. Record the exact baseline SHA, toolchain, and binary hash in the completion report.
6. Kill and restart any stale PFTerminal and background cargo/rustc processes so all
   subsequent evidence comes from the rebuilt binary at the recorded SHA.

## 3. Defects to fix (all release-blocking, all observed in live use on 2026-07-10/11)

### D1. Assignment panes vanish from the pane picker

- **Observed:** user created a Worker pane; it disappeared from /panes. Thread existed on
  disk; the pane list filtered it out.
- **Root cause (verify, don't assume):** `SubmitSpawnAgentTask` inserts the destination into
  `spawn_status_by_thread` (`app/event_dispatch.rs` submit handler), which makes
  `is_spawn_orchestration_thread` true, which the pane picker used as an exclusion filter.
  Any pane that ever receives an injected task (brief, mandate, dispatch) silently leaves
  the user's pane list.
- **Required:** receiving an injected task must never remove a pane from any user-facing
  list. The in-flight `is_managed_spawn_crew_thread` split is the right direction — finish
  it, audit **every** call site of `is_spawn_orchestration_thread` for the same conflation
  (routing, cleanup, status, picker, persistence), and add a TUI matrix row: create pane →
  bind as Worker → receive dispatch → pane still listed, still selectable, still labeled.

### D2. The Manager doesn't know how to dispatch (protocol confusion)

- **Observed, three distinct failures:** Manager invoked Task Node instead of dispatching
  the Worker; Manager treated the dispatch mechanism as a shell command (`command -v`,
  `which` probing); first dispatch failed on nickname routing and only a retry via durable
  thread ID succeeded.
- **Required:** the brief and every mandate must carry, verbatim: the durable Worker target
  ID (never a nickname), a literal fenced dispatch example the Manager can copy, an explicit
  statement that dispatch is a host message protocol (not a shell command, not an
  executable, not Task Node), and a prohibition on tool discovery for dispatch. Nickname
  routing may exist as alias sugar, but generated briefs/mandates always use the durable ID.
  The in-flight drafting-brief fix covers part of this — extend it to the prewritten-spec
  path and mandate text too, then prove it in the TUI matrix: a fresh Manager's **first**
  dispatch attempt must succeed on both the draft path and the prewritten-spec path.

### D3. Dispatch failure was silent-ish and self-healed only by luck

- **Observed:** "Dispatch #1 failed before work began because nickname routing could not
  resolve the sibling Worker pane."
- **Required:** a failed Manager→Worker dispatch is a first-class event: visible message
  naming assignment, Manager, Worker, and cause; counted by the failure-backoff machinery;
  retried against the durable ID automatically exactly once; if the retry fails, the
  assignment pauses with reason. Matrix row: force a bad target and assert the visible
  failure + pause.

### D4. Untested workflow paths were represented as working

- **Observed:** "Draft with Manager → approve → first dispatch" had never been executed
  before the user ran it; the prewritten-spec probe was generalized into "orchestrate
  works."
- **Required:** the qualification matrix (§5) enumerates every path through the guided flow
  as separate rows. A row that has never run is reported as failing, not missing. The
  completion report contains the full row-by-row table. This is what makes rule 1
  enforceable and it is the permanent fix for "done" inflation.

### D5. Outstanding P1s from the 2026-07-10 code review remain in force

- Marker false-positives (Manager acknowledgment can Done/Block the assignment, including
  during Drafting), native pane-loss silent stalls (`fire_destination_for_node` trusts any
  thread id; `ThreadClosed` unhooked; no sweep watchdog), Codex Main as Manager broken.
  Full detail and required tests: `ORCHESTRATE_MANAGER_WORKER_REQUIRED_FIXES_20260710.md`
  Fixes 1-3 (plus 4-8). Those requirements are incorporated here by reference; the partially
  landed `5b7cb5e61` "harden" commit must be audited against each of them rather than
  trusted.

## 4. Simplicity mandate

The user asked for one pane injecting another. Deliver that as the visible product:

- `/orchestrate` fast path: pick Worker → pick Manager (or create) → running. Duration
  defaults to 8h, spec defaults to Draft-with-Manager, no other questions. The long guided
  chain stays available but the two-choice path is the front door.
- Status must answer, at a glance: who manages whom, what phase, when the next injection
  happens, when it ends, and the last dispatch result. If a field can't be filled truthfully
  (e.g. duration while drafting), show the truthful deferred value — never "unlimited" for a
  bounded assignment.
- Remove or repair any control that errors when pressed in the phase where it is shown
  (Test/Mandate-now during Drafting). A visible control that cannot work is a defect.

## 5. TUI qualification matrix (the gate; fully automated, zero humans)

Build `qa/orchestrate_tui_matrix.sh` (or a Rust harness under `tui/tests/`): drives the real
`target/debug/pfterminal` binary inside scripted tmux, uses a deterministic/shortened cadence
via env or test-config hook, captures panes with `tmux capture-pane` after every step, and
writes per-row artifacts (capture text, exit statuses, pane-layout JSON snapshots) under
`qa/artifacts/<SHA>/`. Every row asserts on captured content programmatically — grep/jq, not
eyeballs. The matrix, minimum rows:

1. Guided flow, prewritten spec, create-Manager: assignment created, brief delivered once,
   Worker pane still in /panes (D1), first mandate fires after shortened cadence, Manager's
   first dispatch reaches the Worker (D2), Worker runs, loop repeats twice.
2. Guided flow, Draft-with-Manager, bind-existing Manager: draft conversation → first
   dispatch auto-starts execution → same assertions as row 1.
3. `/orchestrate` fast path (§4): two choices to running.
4. Marker discipline: scripted Manager output restating the contract in prose → phase
   unchanged; line-anchored `WHIP_DONE` → Done, mandates stop (D5/F1).
5. Blocked round-trip: line-anchored `ASSIGNMENT_BLOCKED: <reason>` → Blocked + notice; user
   reply to Manager → Executing.
6. Dispatch failure: invalid/closed Worker target → visible failure, one durable-ID retry,
   pause-with-reason on second failure (D3).
7. Pane loss: kill the Worker thread mid-Executing → pause + notice naming the Worker;
   repeat for Manager (D5/F2).
8. Restart: kill PFTerminal mid-Executing, relaunch, assert resume notice, one-cadence
   delay, loop continues; repeat with the Worker deliberately not restored → pause + notice.
9. Pane hygiene: every pane created during rows 1-8 remains listed, selectable, and
   correctly labeled in /panes at every checkpoint; detaching removes assignments and never
   panes.
10. Codex Main constraints: Manager candidates exclude Codex Main; Worker=Codex Main either
    behaves honestly about idleness or mandates are suppressed with a visible caveat (D5/F3
    resolution as chosen).
11. User precedence: user message to Manager immediately before due mandate → delayed a full
    base cadence, even under failure backoff.
12. Legacy: a persisted legacy nudge from old state fires with old semantics, capped and
    pausable, unaffected by all of the above.
13. Jargon lint over every captured screen: no `whip|holder|target|review` in guided flow,
    status, details, or notices (typed power-command help exempt).
14. Responsiveness: keystroke-to-render checks on every popup in rows 1-3 (the vault-freeze
    lesson; assert bounded latency via timestamped captures).

Plus the full unit/integration gates: focused suites, `cargo fmt --check`, `git diff
--check`, no pending snapshots, no *new* clippy findings, and the restart-lifecycle +
persistence-failure-injection tests still owed from the original plan.

## 6. Definition of Done (machine-checkable, in order)

1. Working tree clean; all work in reviewed commits on the branch; SHA recorded.
2. All §3 defects fixed with named regression tests.
3. `qa/orchestrate_tui_matrix.sh` exits 0 at the release SHA; artifacts committed or
   archived under `qa/artifacts/<SHA>/` with a row-by-row PASS table.
4. Completion report (`ORCHESTRATE_RECOVERY_REPORT_<SHA>.md`): baseline, per-defect
   fix+test+evidence mapping, full matrix table with artifact paths, and an explicit list of
   anything untested-in-TUI (which must be empty for "done").
5. The matrix script is wired into `justfile`/CI so it runs on every future orchestrate
   change — this failure mode gets structurally harder to repeat, not just patched.

If any step cannot be completed, the report says exactly which row fails and why, and the
overall status is "NOT DONE" — a truthful NOT DONE is acceptable; an inflated done is not.
