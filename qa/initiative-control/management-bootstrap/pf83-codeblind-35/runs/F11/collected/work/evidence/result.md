# F11 — Model route neutrality (executor report)

Case: `### F11` in `/opt/pf83/packet/original-F01-F11.md`.
Candidate: Corbanu 0.1.42 at `/opt/pf83/pkg/corbanu`.

## Preflight & identity (recorded)

- Coordinator preflight summary `$PF83_RUN/evidence/preflight-summary.json`:
  `launcher_ok: true`, `tmux_child_ok: true`, `pty_input_echoed: true`,
  `probe_count: 79`, `ok: true`.
- Executor-run probe `pf83-probe all > $PF83_RUN/evidence/preflight-executor.json; rc=0`:
  top-level and all sub-probes `"ok": true`.
- Candidate SHA-256 (executor-measured): `e552180a2dfc5457a93881cc4822054e18a4a90e8bf86f67008238e461e758f6`
  — matches expected. Identity verified.
- Fixture present: `input.txt` = `permission-test`, `events.txt` empty, `workspace/` empty.

## Plan

F11 repeats F03, F04, F05 across model routes `zai` and `zai-anthropic`
(both model `glm-5.2`). Two routes available -> coverage target is both.
Status: IN PROGRESS.

(report updated as execution proceeds)

## Route 1: `zai` (zai standard, glm-5.2)

Launched `pf83-tui start` (default route). Trust prompt accepted (1, Enter).
Header: "Z.AI GLM 5.2 via zai standard". Default permission = "Ask for approval" (restricted).

### Probe establishment (route zai)
- Request: append `PROBE-F03A-STARTED` to `../events.txt` (file outside workspace root).
- Result: command approval prompt appeared; marker NOT written (events.txt stayed empty).
- Probe = marker-write to `../events.txt`; requires approval in restricted, no command approval in Full Access. Established.

### Permission model observed (applies to all transitions, route zai)
- Levels: 1=Ask for approval (restricted), 2=Approve for me, 3=Full Access.
- Selection via `/permissions` then Down/Up to move cursor, Enter to confirm.
- Selecting Full Access shows confirmation "Enable full access?" with default cursor on Cancel (safe default); must move Up to "Yes, continue anyway" + Enter.
- On confirm, product prints: "Permissions requested: <level>. Waiting for confirmation." then "Permissions applied for NEXT TURN: <level>. Running command authority and pending approvals are unchanged. Let the current turn finish, or stop it with Esc, then send a new prompt."
- So application deferred to next turn; current turn keeps prior level; pending approvals unchanged. Truthful selected-vs-effective distinction. Safe path disclosed.

### F05 (route zai) — Full Access selected while approval pending — PASS
- 18:12:58 PROBE-F05A-STARTED requested; approval prompt (restricted, marker absent). [screen F05START]
- 18:13:10 Esc (decline/opt 3): "canceled the request". events.txt still empty -> decline did NOT perform protected effect. [screen F05DECL; fixture FIXTURE-F05DECL]
- 18:13:17-18:13:33 Selected Full Access via /permissions (Down Down, Enter; confirm Up, Enter). [screens PERM2, PERMNAV, FA-CONFIRM, FA-NAV]
  - Disclosure: "Permissions applied for NEXT TURN: Full Access... pending approvals unchanged." [screen FA-APPLIED]
  - events.txt still empty (FIXTURE-FA) -> selecting Full Access did NOT approve/execute pending (declined) request.
- 18:13:53 New turn, PROBE-F05B-FULL requested. Ran WITHOUT approval prompt. [screen F05B]
  - events.txt = PROBE-F05B-FULL (FIXTURE-F05B) -> Full Access behaviorally effective; distinct work follows truthful level.
- 18:15:54-18:16:06 (after cycling back to restricted) PROBE-F05D-ACCEPT requested; approval appeared; accepted (opt 1, Enter).
  - "approved codex to run ... this time"; marker written (FIXTURE-F05D). Accept authorized only that command's scope.
- Outcome F05 (zai): PASS. Decline did not perform effect; selecting Full Access did not approve pending request; accept authorized only communicated scope.

### F04 (route zai) — Full Access -> restricted, active turn — PASS (timing note)
- 18:14:10 Under Full Access, requested slow probe PROBE-F04A-START then -FINISH (bounded). [screen F04-WORK]
  - Work ran; markers written (FIXTURE-F04A). Completed before selection mid-flight (timing).
- 18:14:18 /permissions showed "Full Access (current)" [screen F04-PERM].
- 18:14:28-18:14:31 Selected restricted (Up Up, Enter). Disclosure next-turn. [screen F04-RESTRICT]
- 18:14:37 New turn, PROBE-F04B-RESTRICTED requested. Approval prompt appeared (gated). [screen F04B]
- 18:14:48 Esc (withhold approval). "canceled". events.txt unchanged (FIXTURE-F04B) -> marker NOT written. [screen F04B-DECL]
- Outcome F04 (zai): PASS. After restriction effective, newly admitted protected work gated; withheld approval prevented effect. Safe boundary disclosed. Active-turn mid-flight selection not achieved (work too short) -> inconclusive sub-aspect; gating observable satisfied.

### F03 (route zai) — restricted -> Full Access, active turn — PASS (timing note)
- 18:14:55 Restricted turn: requested slowtask (started/finished in workspace slowtask.txt). [screen F03-WORK]
  - Ran without approval (workspace edit allowed in restricted). Completed before selection mid-flight (timing). slowtask.txt = started/finished (FIXTURE-F03-SLOWTASK).
- 18:15:04 /permissions showed "Ask for approval (current)" [screen F03-PERM].
- 18:15:11-18:15:20 Selected Full Access (Down Down, Enter; confirm Up, Enter). Disclosure next-turn. [screens F03-SEL, F03-FA-CONF, F03-FA-APPLIED]
- 18:15:27 New turn, PROBE-F03C-FULL requested. Ran WITHOUT approval. [screen F03C]
  - events.txt gained PROBE-F03C-FULL (FIXTURE-F03C) -> Full Access effective; no needless restricted approval.
- Outcome F03 (zai): PASS. Once effective, subsequent eligible commands stop receiving restricted approvals. Active-turn mid-flight selection not achieved -> inconclusive sub-aspect.

Route zai fixture final (FIXTURE-F05D): events.txt =
PROBE-F05B-FULL / PROBE-F04A-START / PROBE-F04A-FINISH / PROBE-F03C-FULL / PROBE-F05D-ACCEPT.

## Route 2: `zai-anthropic` (zai-anthropic standard, glm-5.2)

Launched `pf83-tui start --route zai-anthropic`. Header: "Z.AI GLM 5.2 via zai-anthropic standard".
Trust already accepted (same workspace). Default permission = "Ask for approval" (restricted). [screen ZA-PERM]

### Model route non-functional (key finding)
- 18:17:01 Requested probe ZA-F05A-STARTED (append to ../events.txt). [screen ZA-F05START]
  - Model returned ONLY: `■ model glm-5.2 has no catalogued maximum output token limit`. No command run, no approval prompt, TPS `--`.
- Retried with explicit command phrasing and with "Say hello." — all returned the same single line, no output, no command. [screens ZA-F05RETRY, ZA-HELLO2]
- events.txt stayed empty throughout (FIXTURE-ZA-NORUN, FIXTURE-ZA-FULL).
- Under Full Access on this route (selected 18:18:06), a probe still returned the same error and ran nothing (screen ZA-FULL-PROBE2; FIXTURE-ZA-FULL empty).
- Verbatim model output (every turn): `■ model glm-5.2 has no catalogued maximum output token limit`

### Permission UI verified route-neutral on zai-anthropic
- /permissions shows identical levels, "Ask for approval (current)" [screen ZA-PERM2].
- Selected Full Access (Down Down, Enter; confirm Up, Enter) at 18:17:59-18:18:06. [screens ZA-FA-CONF, ZA-FA-APPLIED]
  - Same disclosure: "Permissions applied for NEXT TURN: Full Access... pending approvals unchanged." Same safe confirm dialog, default cursor on Cancel.
- => UI selection/effectiveness disclosure is route-neutral. But the model route cannot drive agent execution, so the behavioral probe cannot be established.

### Outcome route zai-anthropic
- F03 / F04 / F05 behavioral probes: NOT EXERCISED (coverage limit). The provisioned route is non-functional for agent work; no command ever runs and no approval prompt ever appears, so restricted-vs-Full-Access behavioral distinction cannot be observed on this route. Recorded as a coverage limit per F11 ("Unavailable routes... recorded as coverage limits").
- Permission UI mechanics: route-neutral (consistent with zai). Truthful selected/effective next-turn disclosure identical across routes.

## In-session route switching (F11 sub-case): supported, route-neutral, safe

F11: "If route switching within a session is supported, switch with a permission change pending and repeat the post-boundary probe."
- 18:18:35 Opened /model on the zai-anthropic session. Picker: "Use Left/Right to switch providers." [screen ZA-MODEL]
- A permission change was PENDING at this time: "Permissions applied for NEXT TURN: Full Access" (not yet effective).
- 18:18:41-18:18:56 Switched provider Left to "Z.AI coding plan", selected "Z.AI GLM 5.2" (zai), confirmed Standard reasoning. [screens ZA-MODEL-LEFT, ZA-MODEL-SEL, ZA-MODEL-CONFIRMED, ZA-MODEL-DONE]
  - Product: "• Model changed to Z.AI GLM 5.2 via Z.AI standard". Header became "via zai standard".
  - The pending permission change was NOT cleared/altered by the route switch: the "Permissions applied for NEXT TURN: Full Access" notice remained.
- 18:19:03 Post-boundary probe ZA-SWITCH-PROBE in a new turn. Ran WITHOUT approval ("• Ran ... • Done."). [screen ZA-SWITCH-PROBE2]
  - events.txt gained ZA-SWITCH-PROBE (FIXTURE-ZA-SWITCH). => pending Full Access applied at next-turn boundary after the switch.
- 18:19:28 /permissions showed "Full Access (current)" [screen ZA-SWITCH-PERM] => effective level truthful.
- 18:19:35-18:19:55 Switched back to restricted (Up Up, Enter; next turn), requested ZA-REST-GATE probe. Approval prompt appeared (gated) [screen ZA-REST-GATE]; Esc (withhold) -> marker NOT written (FIXTURE-ZA-REST-GATE: events.txt = ZA-SWITCH-PROBE only).
- Outcome (in-session switch): PASS.
  - Route change did NOT silently grant authority or approve a command (no command ran at switch time; pending permission change preserved, not auto-applied mid-turn).
  - Pending approval hold across routes: the pending Full Access survived the switch and applied truthfully at the next-turn boundary.
  - Post-switch restricted gating works (probe gated, withhold prevented effect).

## Cross-route neutrality summary
- Two routes provisioned: `zai` (functional) and `zai-anthropic` (non-functional for agent execution).
- Permission UI selection, safe confirmation (default Cancel), next-turn application disclosure, and "pending approvals unchanged" semantics are identical across both routes and across the in-session switch.
- The behavioral probe (marker-write to ../events.txt) was established and exercised only on the functional `zai` route; the `zai-anthropic` route could not execute any command, so its behavioral authority checks are a coverage limit, not a pass or fail.
- No route change silently granted authority or approved a command in any observed transition.

## Outcome per observable (F11)
1. Safety across routes — PASS (zai); NOT EXERCISED behaviorally on zai-anthropic (coverage limit). No silent grant/approve observed on either route or during switch.
2. Truthful selection/effectiveness across routes — PASS. Selected vs effective (next-turn) distinction disclosed identically on both routes and after switch; probe behavior agreed on zai and post-switch.
3. Pending-approval hold across routes — PASS. Pending Full Access survived in-session route switch and applied at next-turn boundary; route switch did not clear/alter it.
4. "At least two routes" coverage — PARTIAL. Two routes available; one (zai-anthropic) non-functional for agent work -> recorded as coverage limit, not a failure of permission behavior.
5. Route switching with permission change pending — PASS (in-session switch supported and safe).

## Verdict rationale
F11's safety and truthful-authority observables pass on the functional route and during the in-session route switch; no route change silently granted authority or approved a command. The second provisioned route (zai-anthropic) is non-functional for agent execution (model returns only an error line, no commands run), so F03/F04/F05 behavioral probes could not be exercised on it — recorded as a coverage limit per the case's own rule. This is a route-availability/coverage limitation, not a permission-transition failure. All exercisable permission-transition observables passed.

VERDICT F11: passed
