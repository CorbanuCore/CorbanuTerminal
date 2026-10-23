# Owner decisions this amendment must follow (verbatim), and the fix descriptions they refer to

## Decisions

### #447, 2026-10-15T04:53:23Z (IridiumMaster, https://github.com/CorbanuCore/CorbanuTerminal/pull/447#issuecomment-6184708795)

Owner decision (Travis, 2026-10-15, as recommended): Aggressive: amend AGG-T01/T02 to #739 option B (no offer, plain refusal or sandbox denial, file unchanged, note for a remembered rule; test the #626 step separately with a rule saved without protected launch); AGG-04 (#816) amended to the product's behaviour with screen 07/review/picker wording fixed, resting on run targeted2's evidence; AGG-R03 (#818) re-tested from a second Corbanu Terminal on the same home; AGG-R14 (#817) fix the prompt then replay with the frozen driver; hold sign-off until those land, then re-run only AGG-T01, T02, R03 and R14.

### #816, 2026-10-15T04:53:25Z (IridiumMaster, https://github.com/CorbanuCore/CorbanuTerminal/issues/816#issuecomment-6184709088)

Owner decision (Travis, 2026-10-15, as recommended): amend AGG-04 to today's behaviour and fix the wording on screen 07, the review and the picker so they match what's enforced before the restart.

### #817, 2026-10-15T04:53:26Z (IridiumMaster, https://github.com/CorbanuCore/CorbanuTerminal/issues/817#issuecomment-6184709314)

Owner decision (Travis, 2026-10-15, as recommended): fix the child-agent prompt (Enter refused 11 s after it appeared), then replay with the frozen driver unchanged.

### #818, 2026-10-15T04:53:27Z (IridiumMaster, https://github.com/CorbanuCore/CorbanuTerminal/issues/818#issuecomment-6184709598)

Owner decision (Travis, 2026-10-15, as recommended): re-test AGG-R03 from a second Corbanu Terminal on the same home.

### #739, 2026-10-14T22:58:13Z (IridiumMaster, https://github.com/CorbanuCore/CorbanuTerminal/issues/739#issuecomment-6180342417)

Owner decision (Travis, 2026-10-14, all as recommended): option B (SECURITY SIGN-OFF): with the security features on, Permissive stops offering approvals that protected launch will refuse (writes outside the project / runs outside the sandbox); the prompt and the /security review text say so.

## Context, not decisions: what the fixes say the product now does (verbatim issue comments)

These describe the candidate under test (origin/main 4c9b0a0cf0 contains #813, #834 and #835). They are the
developers' descriptions, not test results.

### #739, 2026-10-15T01:04:44Z (IridiumMaster, https://github.com/CorbanuCore/CorbanuTerminal/issues/739#issuecomment-6181983404)

Implemented as option B (Travis's security sign-off, 2026-10-14) in PR #813.

What happens now with `secretless_agent_launch` (protected launch) on, at Permissive:
- A request to run outside the sandbox isn't offered to you. It's refused before anyone is asked, and you see a warning: "Not run: this command asked to run outside the OS sandbox, and that isn't possible here. …"
- After a sandbox denial, "retry without sandbox?" isn't offered, and a warning says why.
- A remembered approval or allow rule ("don't ask again") that would skip the sandbox still skips the prompt, but the command runs inside the sandbox. A note says so, once per session. It no longer ends in "Protected launch refused".
- The model is told that commands never leave the sandbox here.
- `/security` no longer says approved commands can run outside the sandbox. The Permissive summary and review say that protected launch keeps commands in the sandbox. The rows for leaving Moderate or Aggressive no longer list Corbanu's own files as unprotected.

Unchanged: Permissive without protected launch, and Moderate and Aggressive (#747).

Evidence:
- `core/tests/issue_739.rs` uses the real sandbox. It fails on main (a prompt, then "Protected launch refused") and passes with the fix, on macOS and on Linux (glitch).
- Replay of AGG-T01/T02 under Permissive with the milestone flags (GLM 5.3 Flash): `qa/security-levels/issue-739-20261014/`.
- [Video](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-739-issue739-permissive-protected-launch-643a2bdc9196-2026-10-14.mp4)

**For the milestone:** AGG-T01 and AGG-T02 expect the Permissive writes outside the project to land (N+2 and N+3 lines). Under option B they can't, so those expected results need amending. The README in the replay folder suggests replacement wording.

### #816, 2026-10-15T07:09:33Z (IridiumMaster, https://github.com/CorbanuCore/CorbanuTerminal/issues/816#issuecomment-6186570634)

Fixed by #834 (merged `f64bc0e690`, head `a364ae6dbc`), per the owner decision of 2026-10-15.

Before the restart, screen 07, the Aggressive review and the level list now say what holds from confirmation, using the same facts as the inspector's "This session" rows:
- approved and allow-listed commands stay inside this session's sandbox (with its writable folders, and "a permission request you approve can widen it" when those tools are on);
- network off for agent commands, when the sandbox has no network and its permission-request tools are off;
- web search, environment trimming and model keys in the broker, when the session's own config already does them.

"Starts when you restart" lists only what doesn't hold yet. The inspector's status line now says "the rest of Aggressive's session controls apply after restart". A session that holds none of these is told nothing more.

Evidence: AGG-04 screens before and after in `qa/security-levels/fixes/issue-816-20261015/`; tests in `security_level_picker_issue_816_tests.rs` (TUI snapshots) and `current_tests.rs::issue_816_before_restart_facts_match_the_inspector_rows`; [video](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-816-issue816-aggressive-before-restart-says-what-holds-a842d5a4f8c5-2026-10-14.mp4). Gate details are on #834.

Known limit (conservative): if a managed requirement keeps a session on `never` while its permission-request tools are on, the screens leave network-off under the restart while the inspector says it's off. This understates; it never overstates.

For the milestone record: per the owner decision, AGG-04's expected result 3 (and the screen 07 split in AGG-04/AGG-37) needs amending to today's behaviour; that is for the milestone integrator.

### #817, 2026-10-15T07:30:29Z (IridiumMaster, https://github.com/CorbanuCore/CorbanuTerminal/issues/817#issuecomment-6186909097)

Fixed by #835 (merged `4c9b0a0cf0`, head `39359df224`), per the owner decision of 2026-10-15 (option a).

- Keys are ignored only before a prompt is drawn and in the second after (#787's guard; each dropped key restarts the second). After that, a deliberate Enter answers the highlighted option. This holds however the prompt arrives: new, next in a queue, revealed when the prompt above it is answered, held back while you typed, the next question of a form, or the app link's next screen.
- "Keys pressed as this request appeared were ignored" (↑/↓ first) shows only when keys were dropped for that prompt. Keys dropped for an earlier prompt, or text typed into a request another client answered, don't carry over.
- Type-ahead still never answers a new prompt. Typed or pasted text, held keys and options that last beyond the request keep their existing rules.
- What runs and what is denied is unchanged at every level, Permissive included.

**AGG-R14 replay with the frozen driver, unchanged** (`qa/security-levels/fixes/issue-817-20261015/`; disposable home, GLM 5.3 Flash on zai, host Mac):
- The child's prompt appeared after the `wait_agent_v1` question, and the driver's Enter about 9 s later approved it.
- The kill switch then ended the child's `sleep 60`, `ks-child.txt` was absent 90 s later, and the driver completed all steps.
- A second replay on the final head had the prompt appear directly, with the same result.

[Video](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-817-issue817-child-prompt-takes-a-later-enter-8d7269090963-2026-10-14.mp4): an Enter 11 s after the revealed child prompt approves it, and `ks-child.txt` is written. Gate details are on #835.

The milestone can re-run AGG-R14 with the frozen driver unchanged.
