# Spec v2: Orchestrate as a Conversational Manager/Worker Assignment

**Date:** 2026-07-10
**Base:** `integrate/pfterminal-20260707` at `0420104aa`
**Supersedes:** v1 of this document (mechanical mandate loop). v2 adds the conversational
lifecycle: the Manager is a pane the user talks to, not a scheduler artifact.

**Problem:** Orchestrate exposes plumbing (whips/holders/targets/modes) instead of the product:
a Manager pane the user converses with, which continuously rides one Worker pane. It automates
the user's manual workflow — watching tmux panes and injecting prompts — without taking the
conversation away from the user.

All paths relative to `codex-rs/`. Line numbers at `0420104aa`.

---

## 1. Product model

- An **assignment** binds one **Manager pane** to one **Worker pane** (1:1) with a duration.
- The Manager is a first-class conversational pane — "a Nazgul without a Troll":
  - The user **iterates the spec with the Manager** in normal conversation before execution.
  - During execution the user drops in any time for status ("what's the worker doing?") —
    Manager idle time between mandates is a feature, not waste.
  - The Manager may report **blocked**; that surfaces to the user instead of being re-rammed.
- The keep-alive (**mandate**) lives on the Manager: by default a pane's process stops when its
  turn ends, so automation re-mandates the Manager when the user is away and the Worker has
  stopped. **User conversation always takes precedence over mandates.**
- The Worker needs no whip: it is driven by Manager dispatches (and occasionally the user).
- Once execution starts, the loop must not stop from exhaustion, silence, or Manager
  indecision — only from: done, expiry, blocked-awaiting-user, user pause/detach, pane loss, or
  repeated unrecoverable Manager failure.
- **Flagship scenario — unattended overnight run:** the user locks the spec in the evening,
  sets an 8h duration, and goes to bed. The loop must survive the night with zero user input:
  the Manager audits and re-tasks the Worker all night and can answer "what happened?" in the
  morning from its own context. Every design decision below is subordinate to this scenario:
  when user-attention rules and continuity conflict, unattended continuity wins by default and
  user activity only _adds_ precedence while the user is actually present.

## 2. Assignment lifecycle

```
        create/bind                 first dispatch to Worker
 [Drafting] ── user+Manager iterate spec ──► [Executing] ◄──────────┐
     │                                          │    ▲              │
     │ user: /orchestrate start (override)      │    │ user replies │
     │                                          ▼    │ or Manager   │
     └────────────────────────────────►      [Blocked]──────────────┘
                                                │
[Done] Manager emits stop marker               │
[Expired] duration ends (execution clock)      │
[Paused] user pause, pane loss ◄───────────────┘
```

### Drafting

- Starts at kickoff. Manager receives the **birth brief** (§3) and converses with the user to
  produce/refine the assignment spec. **No mandates fire during Drafting.**
- Transition to Executing: automatic on the Manager's **first dispatch to the Worker** ("starts
  ramming once the spec is locked"), or explicitly via `/orchestrate start <id>`.
- The duration clock **starts at Executing**, not at kickoff — drafting is user-paced.

### Executing

- The mandate loop (§4) keeps the Manager riding the Worker whenever the user is away.
- User conversation with the Manager is expected and never counts as failure, ignoring, or
  idleness abuse; it resets the mandate timer (§4).

### Blocked

- Manager emits `ASSIGNMENT_BLOCKED: <reason>` in its output → state Blocked, prominent
  notification with the reason, mandates suspend.
- Blocked is a **last resort**, and the brief says so explicitly (§3): overnight, a Blocked at
  1am halts the run until morning. The Manager must first exhaust autonomous options (retry,
  work around, re-scope within the spec, task the Worker differently) and may only block on
  genuinely user-owned decisions: missing credentials, destructive/irreversible choices, or
  spec ambiguity that cannot be resolved conservatively.
- Auto-resume to Executing when the user sends the Manager a message, or the Manager dispatches
  to the Worker (whichever first). No manual resume command required (but `/orchestrate resume`
  works).

### Terminal / paused

| Condition                               | Behavior                                                                                                                                                                         |
| --------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Manager emits stop marker (`WHIP_DONE`) | `Done`; notify; panes remain                                                                                                                                                     |
| Execution duration expires              | `Expired`; notify; user may extend                                                                                                                                               |
| User pause/detach                       | Existing semantics; detach deletes (fix 3)                                                                                                                                       |
| Worker pane closes                      | Paused + notify ("Worker gone")                                                                                                                                                  |
| Manager pane closes                     | Paused + notify ("Manager gone")                                                                                                                                                 |
| Manager turn failures                   | **Backoff, not pause** (§4.1) — a transient 2am API outage must not end the night. Notify after 3 consecutive failures; hard-pause never happens on failures alone before expiry |

Worker-side empty-output/failed-turn auto-pauses are removed for assignments: judging a silent
or failing Worker is the Manager's job.

## 3. Manager birth brief

Injected as the Manager's first turn (created pane) or immediately on bind (existing pane):

- Role: "You are the Manager of Worker <label>. You will ride it continuously until <duration>
  after execution starts."
- Phase contract: "First, iterate with the user to lock the spec. Your first dispatch to the
  Worker starts execution."
- Execution contract: when mandated, audit the Worker's latest output, decide, dispatch the next
  concrete task via the existing dispatch block (`pfterminal_send_task`); answer user status
  questions from your context whenever asked; emit `ASSIGNMENT_BLOCKED: <reason>` if you cannot
  proceed without the user; emit `WHIP_DONE` only when the work is genuinely complete.
- Overnight autonomy: "Assume the user may be away for hours. Resolve problems autonomously —
  retry, work around, re-scope within the spec — and treat `ASSIGNMENT_BLOCKED` as a last
  resort reserved for user-owned decisions. Keep a concise running progress log in your
  responses (what the Worker did, what you dispatched, open risks) so a morning status question
  can be answered from context."
- Cadence disclosure: "While executing, you are re-mandated every 15 minutes when the Worker is
  stopped and the user is away. User messages always take priority over mandates."
- Initial instructions document (inline + source path) if the user selected one at kickoff;
  otherwise the spec is expected to emerge from Drafting (the Manager should save it via the
  existing whip-doc save flow so restarts and mandates can reference it).

## 4. The mandate loop (Executing only)

Inject the Manager with a short mandate ("Worker <label> stopped at <t>. Audit and act.
Spec: <path>") when ALL hold:

1. State is Executing.
2. Worker is idle (`target_node_is_idle`).
3. Manager is idle.
4. ≥ cadence (default **15 min**) since the last mandate, AND
5. ≥ cadence since the **last user-initiated turn on the Manager pane** — user activity resets
   the timer. The user talking IS management; automation only fills gaps when the user walks
   away. Mandate turns themselves do not reset the user-activity clock.

Removed relative to today's review-whip semantics (these are what make the current build stop):

- **No fire cap** (`DEFAULT_MAX_FIRES = 20` exhaustion does not apply; duration is the budget).
- **No ignore-pause** (`ignored_review_fires` never pauses an assignment — a Manager that
  reviews without dispatching, or spends its turn answering the user, is simply re-mandated
  next cadence).
- **No per-idle-generation gate** — re-mandate on cadence even if the Worker never ran in
  between; otherwise Manager indecision deadlocks the loop (today's silent-stop bug).
- Worker-idle **edge** mandates immediately when conditions 4-5 are already satisfied.

The 45s sweep (`WHIP_SWEEP_INTERVAL`) remains the clock; cadence and user-activity timestamps
are data on the assignment.

### 4.1 Manager failure backoff (overnight resilience)

Failed Manager turns (errors, not "reviewed without dispatching") do not pause the assignment:

- Each consecutive failed mandate turn doubles the effective cadence: 15m → 30m → 60m → 120m
  (cap 2h). A successful Manager turn resets to the base cadence.
- After 3 consecutive failures, post a notification ("Assignment <id>: Manager failing,
  retrying with backoff") — visible in the morning, but the loop keeps trying until expiry.
- `consecutive_failed_turns` (from the earlier fix pass) is repointed at the Manager and feeds
  the backoff counter instead of a pause.

## 5. Setup flow

`/orchestrate` guided flow:

1. **Pick Worker** (existing pane picker).
2. **Duration** (existing picker; applies to Executing).
3. **Instructions** — existing doc picker, plus explicit `Draft with manager` option (skip doc;
   spec emerges in Drafting). "Write new..." kept.
4. **Manager** — `Create manager pane` (default; Codex-native, `native_spawn_default_model()`,
   optionally the spawn tabbed model picker later) or `Bind existing pane` (any idle pane except
   the Worker).
5. Confirm → Drafting begins (brief injected). No mode/holder/cooldown/max-fires questions
   anywhere in the guided flow; those remain typed power-user flags only.

## 6. Naming, status, visibility

- User-facing vocabulary: **assignment / Manager / Worker / mandate / spec**. No
  whip/holder/target/review strings in guided flow or status views.
- Status row: `Manager <M> -> Worker <W>; drafting|executing|blocked(<reason>)|paused|done;
next mandate in 9m; ends 18:40Z; spec <name>`.
- Detail view actions: talk to manager (focus pane), start execution (drafting only),
  mandate now, pause/resume, extend, test, detach.
- Pane suffixes (`whip_suffix_for_target`, `orchestrate.rs:638`): Worker shows
  `managed-by <M>`; Manager shows `managing <W>; <phase>; next mandate <t>`.

## 7. Persistence and restart

- Assignments persist with phase, cadence timestamps, and spec path; they **resume** after
  restart with a notice ("Assignment resumed: M -> W, <phase>, ends <t>").
- First post-restart mandate waits one full cadence (supersedes the fresh-idle-edge gate, fix 4,
  for assignments; the gate stays for legacy whips). Drafting assignments resume silently — the
  user picks the conversation back up.
- Missing Manager or Worker pane after restart → Paused + notify.

## 8. Implementation sketch

- `Whip` gains `kind: Assignment { phase, last_user_turn_utc, execution_started_utc }` (serde
  defaults for legacy state); manager = existing `holder`, worker = existing `target`.
- Phase transitions hook existing signals: first dispatch = `note_whip_holder_dispatched`
  (`orchestrate.rs:1379`); user-turn tracking hooks the same submit paths that mark
  `spawn_operator_input_seen` (`claude_panes/app_integration.rs:688`) plus native user submits.
- Mandate gating: branch in `plan_whip_fire_for_generation` for assignments (skip max-fires /
  idle-generation / holder-ignore; add cadence + user-activity checks).
- Blocked: extend the existing output scan (`pause_matching_whips_on_stop_marker` pattern) on
  the **Manager's** output for `ASSIGNMENT_BLOCKED:`; store reason on the assignment.
- Brief/mandate builders replace `review_whip_task` for assignments; injection reuses
  `SubmitSpawnAgentTask` / `SubmitSpawnClaudePaneTask`; Manager creation reuses
  `CreateSpawnAgent` with the brief as initial task.
- Legacy: auto-mode whips remain typed-command-only ("legacy nudge"); review-mode attach via
  typed command maps to an Assignment.

## 9. Acceptance

1. **Kickoff + drafting:** guided flow creates Manager with brief; no mandate fires while the
   user and Manager iterate (deterministic clock, > 1h simulated drafting).
2. **Auto-arm:** Manager's first Worker dispatch flips Drafting→Executing and starts the
   duration clock; `/orchestrate start` does the same explicitly.
3. **Continuity:** > 20 mandate/dispatch cycles without exhaustion; Manager reviewing without
   dispatching is re-mandated next cadence (no ignore-pause, no idle-generation deadlock).
4. **User precedence:** user message to the Manager delays the next mandate by a full cadence;
   a status Q&A turn mid-execution neither fires a mandate early nor counts toward Manager
   failure; mandates never interrupt a running Manager turn.
5. **Blocked:** `ASSIGNMENT_BLOCKED: <reason>` → Blocked + notification with reason + no further
   mandates; user reply auto-resumes Executing.
6. **End conditions:** one test per row of the §2 table, including failure-backoff cadence
   doubling, cap, notification after 3 failures, and reset on success — with no pause.
   6a. **Overnight simulation (flagship):** deterministic-clock run of an 8h Executing assignment
   with zero user activity: ≥ 15 mandate/dispatch cycles complete; two injected transient
   Manager failures mid-run cause backoff then recovery, not a stall; at "morning," a user
   status question is answered without disturbing the loop, and the assignment reaches Done or
   Expired — never Paused.
7. **Restart:** Executing assignment resumes, notice shown, first mandate after one full
   cadence; Drafting assignment resumes silently; native and Claude Workers both covered
   (closes the architecture review's end-to-end lifecycle gap).
8. **1:1 + self-check:** replace/block semantics on busy Manager/Worker; a pane cannot manage
   itself.
9. **No jargon:** guided flow and status snapshots contain no whip/holder/target/review strings.
10. **Legacy:** old persisted whips load and behave unchanged; typed auto-mode unchanged.

## 10. Open questions

1. Cadence flag (`--cadence 5m`) — default 15m, flag only? (Recommend yes.)
2. Should a user message to the **Worker** also reset the mandate timer? (Recommend yes —
   user is actively driving; Manager resumes when they stop.)
3. `ASSIGNMENT_BLOCKED` marker text vs. structured block — plain marker line for v2?
4. Manager model default: `native_spawn_default_model()` now; tabbed picker step later?
