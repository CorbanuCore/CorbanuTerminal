# Requested Code Review: Manager/Worker Assignments

**Date:** 2026-07-10
**Repository:** `/home/pfrpc/repos/PfTerminal-triage-clean`
**Branch:** `integrate/pfterminal-20260707`
**Review range:** `f20a13186..292cea31e`
**Implementation commit:** `292cea31e feat(tui): add manager worker assignments`
**Source documents:**

- `ORCHESTRATE_MANAGER_WORKER_SPEC_20260710.md`
- `ORCHESTRATE_MANAGER_WORKER_PLAN_20260710.md`

## Review Mission

Perform an adversarial code review of the new conversational Manager/Worker Assignment feature.
Determine whether it can safely support an unattended eight-hour run without silently stopping,
losing lifecycle state, overriding user activity, dispatching to the wrong pane, or changing legacy
automation behavior.

Review only. Do not edit implementation files, update snapshots, commit, push, merge, or rewrite
history. Write findings to:

`ORCHESTRATE_MANAGER_WORKER_CODE_REVIEW_FINDINGS_20260710.md`

Treat tests as claims to inspect, not proof that the implementation is correct.

## Required Finding Format

Every finding must contain:

1. Severity: `P0`, `P1`, `P2`, or `P3`.
2. A concise broken-invariant title.
3. Exact file and line reference.
4. A reachable trigger or reproduction sequence.
5. Actual behavior and expected behavior.
6. Why current tests do not detect it.
7. The smallest defensible repair direction.

Do not report style preferences, speculative risks without a reachable path, unrelated pre-existing
issues, or generic requests for more tests.

## Establish Ground Truth

Run and record:

```bash
cd /home/pfrpc/repos/PfTerminal-triage-clean
git status --short --branch
git log --oneline f20a13186..292cea31e
git diff --stat f20a13186..292cea31e
git diff --check f20a13186..292cea31e
git diff f20a13186..292cea31e -- codex-rs/tui/src/orchestrate.rs
```

Read all changed files, not only `orchestrate.rs`:

- `codex-rs/tui/src/app.rs`
- `codex-rs/tui/src/app/event_dispatch.rs`
- `codex-rs/tui/src/app/test_support.rs`
- `codex-rs/tui/src/app/tests.rs`
- `codex-rs/tui/src/app/thread_routing.rs`
- `codex-rs/tui/src/app_event.rs`
- `codex-rs/tui/src/claude_panes/app_integration.rs`
- `codex-rs/tui/src/claude_panes/tests.rs`
- `codex-rs/tui/src/orchestrate.rs`

## Product Invariants

The review must verify these invariants against implementation and call sites.

### Lifecycle

- A new Assignment starts in `Drafting` and cannot receive periodic mandates.
- The first Manager-to-Worker dispatch starts `Executing` exactly once and anchors expiry to that
  instant, not Assignment creation.
- `/orchestrate start <id>` provides the same explicit transition without allowing agent-origin
  escalation.
- `ASSIGNMENT_BLOCKED: <reason>` from the Manager enters `Blocked`; an empty reason does not.
- A user turn to the Manager or a Manager dispatch to the Worker resumes a Blocked Assignment.
- `WHIP_DONE` from the Manager enters `Done`. The same marker from the Worker has no effect.
- Pane loss pauses an active Assignment and reports whether Manager or Worker disappeared.
- Done Assignments are not later converted to Paused because a pane closes or is missing at restart.
- Unlimited duration remains unlimited after execution begins.

### Scheduling and user precedence

- Mandates run only while `Executing`, with both Manager and Worker idle.
- The base cadence is 15 minutes and cannot double-fire through simultaneous edge and sweep paths.
- User turns to Manager or Worker delay automation; injected mandates and dispatch tasks do not
  masquerade as user activity.
- Assignments have no fire cap, idle-generation deadlock, ignore pause, or Worker-side empty/failure
  pause inherited from legacy automation.
- Manager failures back off 15m -> 30m -> 60m -> 120m, notify on the third consecutive failure,
  reset after success, and never terminate the Assignment by themselves.
- Expiry, Done, Blocked, Paused, and Drafting all suppress mandate delivery.

### Persistence and restart

- Every lifecycle, cadence, failure, marker, user-activity, and pane-loss mutation is persisted after
  the in-memory transition.
- Old layouts without `kind` deserialize as `LegacyNudge` with no semantic change.
- Restored Executing Assignments wait one full cadence before the next mandate.
- Restored Drafting Assignments remain silent.
- A missing Manager or Worker is detected only after pane/thread restoration has had a chance to
  load valid destinations; valid restored panes must not be falsely paused.
- Persistence failures are visible and do not leave the UI claiming durable state that was never
  saved.

### Routing and ownership

- Manager and Worker cannot resolve to the same normalized pane/thread identity.
- The guided Manager picker excludes the selected Worker across native and Claude pane identifiers.
- Birth briefs and periodic mandates are delivered to the Manager, never the Worker.
- First-dispatch transition observes every supported native and Claude dispatch path, including
  aliases and restored pane identifiers.
- An agent cannot start, resume, fire, test, replace, detach, or otherwise seize a user-owned
  Assignment outside the intended authorization rules.
- Creating a Manager uses the expected model/provider pair and does not accidentally bind it into
  Nazgul/Troll/Orc hierarchy state.
- A Manager spawn failure leaves no Assignment, nickname reservation, pending task, or phantom pane.

### Guided UX

- With only Codex Main open, it can be selected as Worker and a new Manager can be created.
- Flow order is Worker -> duration -> spec -> Manager -> confirmation, except the create-Manager
  path where creation and attach are intentionally atomic.
- Binding an existing Manager shows confirmation before mutation and injects the birth brief once.
- The Worker cannot appear as a Manager candidate.
- Replacing an existing Assignment is explicit and does not silently discard the old Assignment
  before the new Manager is valid.
- Status and detail views accurately expose phase, Manager, Worker, expiry, next mandate, pause,
  resume, start, extend, test, mandate-now, and end behavior.
- Guided screens use Manager/Worker/Assignment language rather than implementation jargon.
- Popup interaction remains responsive under keyboard, search, escape/back, and repeated entry.

### Legacy compatibility

- Existing `LegacyNudge` auto and review records retain their fire caps, idle-generation gating,
  stop-marker handling, ignore/failure pauses, persistence, and restore behavior.
- Typed `--mode auto` remains legacy automation.
- Typed `--mode review` creates an Assignment as specified.
- Serde defaults do not reinterpret an existing persisted record as an Assignment.
- Status/detail changes do not remove required legacy controls.

## Required Adversarial Scenarios

Manually trace or add temporary local instrumentation for these cases. Do not commit instrumentation.

1. Create an eight-hour Assignment, spend one hour Drafting, then dispatch. Confirm expiry is eight
   hours after dispatch.
2. Advance a deterministic clock through at least 20 successful mandate cycles without Worker state
   changing between cycles. Confirm no exhaustion or generation deadlock.
3. Race a Worker-idle edge against the 45-second sweep at the cadence boundary. Confirm one mandate.
4. Send user input immediately before a due mandate on both native and Claude Manager panes. Confirm
   a full-cadence delay.
5. Inject three Manager failures, restart after the second, then recover. Confirm durable backoff,
   one notification, and reset after success.
6. Emit Blocked and Done markers from Worker, then Manager. Confirm only Manager output controls
   Assignment phase.
7. Close Worker and Manager independently during Drafting, Executing, Blocked, and Done.
8. Restart with valid restored native panes, valid Claude panes, a missing Worker, and a missing
   Manager. Look specifically for false missing-pane detection caused by startup ordering.
9. Select Codex Main as Worker with no other panes, create a Manager, and confirm the brief is sent
   exactly once.
10. Force Manager spawn failure and persistence failure at each transition. Confirm no partial
    Assignment is presented as ready.
11. Bind an existing pane as Manager, cancel at confirmation, then repeat and accept. Confirm no task
    was sent on cancel and one brief was sent on accept.
12. Resume a legacy persisted review record and exercise its existing ignore-pause behavior.

## Test Commands

Run the focused gates:

```bash
cd /home/pfrpc/repos/PfTerminal-triage-clean/codex-rs
cargo test -p codex-tui --lib assignment_ -- --nocapture
cargo test -p codex-tui --lib orchestrate_ -- --nocapture
cargo test -p codex-tui --lib guided_attach_args_parse_to_attach_command -- --nocapture
cargo check -p codex-tui --lib
cargo fmt --all -- --check
git diff --check f20a13186..292cea31e
```

Attempt the broader gates, but separate new regressions from baseline failures:

```bash
cargo test -p codex-tui --lib --no-fail-fast
cargo clippy -p codex-tui --lib --no-deps -- -D warnings
```

At implementation time, the focused Assignment and legacy orchestrate suites passed. The full TUI
suite had four unrelated snapshot mismatches before aborting in the pre-existing
`open_agent_picker_prunes_terminal_metadata_only_threads` stack overflow. Clippy reached three
pre-existing warnings in `app_server_session.rs`, `chatwidget/settings.rs`, and
`claude_panes/app_integration.rs`. Verify this baseline rather than assuming every broad-gate failure
belongs to the reviewed commit.

## Real PTY Review

Use the built debug binary, not a mocked selection view:

```bash
cd /home/pfrpc/repos/PfTerminal-triage-clean
pfterminal-debug --yolo
```

Exercise `/orchestrate` through both Manager paths. Confirm keyboard responsiveness at every popup,
then observe at least one real Manager brief/turn. Clean up the test Assignment afterward. Record the
exact binary path, commit SHA, terminal/tmux environment, and observed result in the findings file.

## Reviewer Deliverable

Write `ORCHESTRATE_MANAGER_WORKER_CODE_REVIEW_FINDINGS_20260710.md` with:

1. Verdict: `approve`, `approve with required fixes`, or `do not release`.
2. Findings ordered by severity.
3. Test and PTY evidence, including commands and outcomes.
4. Spec requirements not implemented or not proven.
5. Residual risks and the minimum release gate.

If there are no findings, state that explicitly and identify remaining untested risk. Do not describe
the feature as release-ready solely because focused tests pass.
