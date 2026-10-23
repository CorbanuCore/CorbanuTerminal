# Frozen functional cases: permission transitions

Status: proposed original cases, not execution results or pass evidence. Frozen from the user intent supplied to this designer; no repository source, existing tests, logs, implementation findings, other task history, or screenshots were consulted. Priorities: P0 = permission safety or truthful authority; P1 = usability and continuity. Cases describe human-observable behavior without prescribing controls, architecture, or timing mechanisms.

## Safe fixture and observation rules

Use a disposable local session and a dedicated temporary folder containing only synthetic files: `input.txt` with `permission-test`, an append-only `events.txt`, and disposable output files. Use local read, marker-write, and bounded slow-write operations only; no secrets, network, financial effects, or changes outside the fixture. Label each requested operation uniquely so its start and completion can be distinguished. A slow operation may write `started`, wait briefly, then write `finished`; keep it bounded and harmless.

Establish through visible behavior a harmless operation that requires approval in the chosen restricted mode and is permitted without command approval in Full Access. Call this the probe. If no such probe can be identified, mark behavioral authority checks inconclusive; do not infer effectiveness from a label alone. Observe the permission UI, approval prompts, ordinary conversation output, and fixture contents. Do not inspect implementation or diagnostic logs. Only submit probe work when the case calls for it.

At each transition, record the user's selected level, the UI's claimed effective level, and whether the probe behavior agrees. A requested selection may be displayed while it is pending, but must not be represented as already effective. If immediate application is unavailable, the UI must explain the pending state and provide an understandable safe path to application. No exact visual design is assumed.

## Cases

### F01 — P0: Idle restricted → Full Access

- Starting state: idle session, restricted/ask-for-approval visibly effective; probe behavior established.
- Actions: choose Full Access; follow any clearly presented application path; submit a new probe after effectiveness is reported.
- Observable outcomes: selected and effective levels are distinguishable during any delay. A truthful success state appears only once Full Access applies. The new probe completes without the needless command approval previously caused by the restricted level. Any remaining prompt must have an understandable independent basis; a stale restricted command gate is a failure.

### F02 — P0: Idle Full Access → restricted

- Starting state: idle session with Full Access visibly and behaviorally effective.
- Actions: choose restricted; submit a fresh probe during any pending interval, then again after the UI reports application. Do not approve either probe initially.
- Observable outcomes: the UI does not imply protection is active before it is. The application path prevents newly admitted over-privileged work from slipping through the transition. Once restriction is effective, a probe requiring approval cannot perform its protected effect without approval. If restriction cannot take effect immediately, the user sees the safe boundary and what is needed to reach it.

### F03 — P0: Active turn restricted → Full Access, no pending approval

- Starting state: restricted session; a turn is actively doing harmless local work; no approval is pending.
- Actions: choose Full Access while the turn is active; request a uniquely labeled probe as the next operation in that turn where supported. If application is deferred, follow the offered path and repeat the probe at the stated boundary.
- Observable outcomes: working status does not hide a selected/effective mismatch. Once safely effective, subsequent eligible commands stop receiving restricted-mode approvals. If the current turn keeps its prior level, that limitation and the path to Full Access are clear. Task progress and any required continuation are understandable.

### F04 — P0: Active turn Full Access → restricted

- Starting state: active Full Access turn with harmless work underway and another probe requested but not yet started.
- Actions: select restricted; request an additional uniquely labeled probe after the selection. Withhold all approvals.
- Observable outcomes: the transition has an observable safe boundary; new work cannot quietly acquire excess authority while restriction is requested or applying. After restriction is effective, newly admitted protected work is gated. Work already authorized before the boundary is assessed under F06, not assumed retroactively revoked. If timing prevents distinguishing admission order, record that part as inconclusive and repeat with clearly separated actions.

### F05 — P0: Full Access selected while approval is already pending

- Starting state: restricted session paused at a visible approval for a probe; its protected marker does not yet exist.
- Actions: select Full Access without accepting the pending approval. Observe the original operation. In fresh repetitions, explicitly decline the original approval, then explicitly accept it. If the change invalidates or replaces the request, follow the disclosed resolution path instead.
- Observable outcomes: selecting Full Access alone does not count as approval of the pending request or silently execute its protected effect. The original request has an understandable disposition; it is not quietly reissued to bypass the withheld decision. Declining does not perform the declined effect; accepting authorizes only the scope communicated by that approval. Subsequent distinct work follows the truthfully effective level. Deferral, cancellation, or explicit resolution are acceptable only if clear and safe.

### F06 — P0: Permission change during an in-flight operation, both directions

- Starting state: a bounded slow operation has visibly written its `started` marker. Run once under Full Access; run again under restricted mode after explicitly approving that operation.
- Actions: while the operation is running, select the opposite level. Request a separate probe afterward and observe both operations.
- Observable outcomes: the UI truthfully describes whether the change is effective or waiting for a boundary. It does not imply that already performed effects were undone. The separate probe follows the safe transition boundary and effective level. Completion, interruption, or continuation of the already-authorized operation is recorded as product behavior, not judged against invented retroactive termination/revocation rules. If interrupted, partial fixture effects and continuation are understandable.

### F07 — P0: Rapid or conflicting selections

- Starting state: an active harmless turn with no pending approval; use a visible pending-application interval if the product provides one.
- Actions: select Full Access → restricted → Full Access, then in a fresh repetition restricted → Full Access → restricted. Make changes before earlier changes settle where possible. Also try a later conflicting selection while an application confirmation is open. Submit a probe after the UI settles.
- Observable outcomes: the UI makes clear which requests were accepted, superseded, cancelled, or refused. An older completion cannot falsely overwrite the latest accepted selection or effective level. Final probe behavior matches the reported effective level; an intermediate Full Access request does not silently defeat a final accepted restriction. If controls serialize changes, that limitation is visible and the refused selection is not reported as accepted. No ordering policy is assumed beyond truthful, safe behavior.

### F08 — P0: Cancelled or failed application

- Starting state: repeat from each effective level using any naturally available confirmation or safe application step.
- Actions: cancel the change at an offered cancellation point. Separately exercise a reproducible benign application failure through exposed behavior, if available; do not corrupt settings or invent a hidden failure mechanism. Submit a probe after each resolved attempt.
- Observable outcomes: cancellation/failure is reported accurately; neither the label nor probe behavior falsely presents the requested change as successful. The UI shows the actual remaining effective authority and any unresolved request. Retry, dismissal, or recovery is understandable and preserves the safe restriction boundary. If no cancellation or safe failure route exists, record the relevant branch as not exercised, not passed.

### F09 — P1: Continuation and new turn across an application boundary

- Starting state: active work with completed synthetic output and a permission change selected. Repeat both directions; include a deferred change if supported.
- Actions: continue the task through the stated application path; let the turn finish; submit a new turn containing a fresh probe. If the path requires stopping or starting a session, follow it and inspect the visible retained context.
- Observable outcomes: the user can tell which level applies to continuation and to the new turn. A deferred change is not silently forgotten or falsely shown as applied. Completed work is not silently duplicated, and any lost context, paused work, or need to resubmit is clear. Conversation wording alone does not silently replace the user's chosen permission level.

### F10 — P0: Restart and recovery

- Starting state: separate disposable runs with (a) effective restricted, (b) effective Full Access, (c) a change pending in either direction, and (d) a pending command approval.
- Actions: use an ordinary app/session restart. For pending states, also use an exposed safe interruption/recovery route if available. Reopen the relevant session, inspect permission state, then request a new probe without accepting any restored pending approval.
- Observable outcomes: restored selected/effective state is intelligible before further protected work. Persistence or reset behavior is accurately communicated and must not silently increase authority. An unfinished transition is not reported as complete without application, and recovery never treats the pending approval as accepted. The user can identify retained work and the next safe continuation step. No persistence default or automatic resumption policy is assumed.

### F11 — P1: Model route neutrality

- Starting state: the same fixture and permission starting state on each exposed supported model route; use at least two routes if available.
- Actions: repeat F03, F04, and F05 per route. If route switching within a session is supported, switch with a permission change pending and repeat the post-boundary probe.
- Observable outcomes: safety, truthful selection/effectiveness, and treatment of pending approval hold across routes. A route change does not silently grant authority or approve a command. Different capabilities or application timing are acceptable when disclosed with a safe path; identical controls or timing are not required. Unavailable routes and unsupported switching are recorded as coverage limits.

## Unresolved product ambiguities

- No visual design was supplied: placement, wording, indicators, confirmation controls, and accessibility presentation remain open. Tests require understandable distinctions, not a specific widget.
- The scope of a selection (current operation, turn, session, future sessions) and the exact safe application boundary are unspecified. The product must communicate them; these cases do not choose them.
- Admission/authorization timing for queued work and the treatment of operations already authorized or running are unspecified. These cases require safe handling of new work and impose no retroactive termination, revocation, or rollback semantics.
- Full Access may still encounter independent platform or tool restrictions. Their scope is unspecified; establish the probe before deciding whether a prompt is needless.
- Pending-approval disposition during an upgrade is unspecified; explicit resolution, deferral, or cancellation may fit, but implicit acceptance does not.
- Rapid-change ordering, cancellation availability, failure recovery, and whether applying a change requires a new turn or session are unspecified. Truthful state and a safe, understandable path remain required.
- Persistence defaults, automatic restart/resume behavior, and the supported model routes are unspecified. Recovery must not silently escalate authority or approve outstanding work.

Freeze rule: preserve this original set as written. Record execution results, coverage gaps, and later product decisions separately; this document itself makes no execution or pass claims.
