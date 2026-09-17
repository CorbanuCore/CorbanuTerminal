# owner-recurrence-47 — diagnosed domain deferral
Base: `a2e4e4286286d63a5920b5ac63932cc8bbb3b8e6`; frozen brief verified:
`d406deb485734ce556a27e631719d1b2142e9de712177f4f8638b5ae9eb53dc3`.
Product-initiative revision within active initiative-delivery-control / PF-80-S01
(in_progress). **Internal delivery control — TO BUILD**: “durable event dispatch,
acknowledgments and watchdog”; “Show blockers, rendered sprints, human test plans,
machines, run logs and freshness”. Frozen worker allocation owns this worktree;
shared plan/sprint metadata and acceptance remain manager-owned. The plan records
this branch with its earlier allocation base; this revision uses the frozen base above.
No live job enabled, global domain setting changed, credentials read, push or release.
This is implementation/diagnostic evidence, not independent functional acceptance
or a human-test handoff; those later manager-owned gates remain open.

The launchd log explains the pending interval: **pending spawn, domain in
on-demand-only mode**. Every sampled GUI domain reports on-demand count 1,
session Aqua. The diagnostic user domain reports session Background and successfully
runs interval jobs when explicitly given LimitLoadToSessionType=Background.
The current GUI schedule cannot be relied on in this host state. StartInterval
itself is achievable here: the user-domain comparison completed four automatic
firings, with launchd blame and immediate reason both identifying interval.
Changing the live domain/mechanism is an owner decision, not part of this revision.
The user-domain lane is diagnostic only: production service observation still targets
GUI, so its own dashboard does not qualify user-domain recurrence. Four firings prove
feasibility, not long-term reliability or a qualified production migration.
Why the GUI domain entered on-demand-only mode was not established or altered.

| Candidate | Instrumentation and result |
| --- | --- |
| (a) Surviving descendant | Tracked Popen PIDs/executables/exit codes, sampled process groups and service active count. All sysctl/ps children exited 0 before the tick returned; no GUI descendant survived. AbandonProcessGroup=true still pended. Falsified for these fixture ticks. |
| (b) Background power/thermal deferral | Per-interval pmset batt/therm/custom plus Interactive comparison. AC power, lowpowermode 0, no thermal/performance warning recorded throughout; Interactive pended too. Falsified as the explanation of this pending spawn, not a general claim about all power states. |
| (c) Domain/session | Per-interval gui/user domain headers, service print and running-process blame. Domain condition confirmed: GUI on-demand-only blocks the timer; user/Background runs. The GUI session is Aqua, so a literal non-Aqua GUI mismatch is falsified. |
| (d) Timer coalescing | Per-interval launchd log show, plus raw stream attempt. Timers reach pending state about 30 seconds after exit; logs explicitly attribute suppression to on-demand-only mode. Coalescing is falsified as the cause of the missing GUI firings; small scheduling jitter is not excluded. |

Final diagnostic: `/private/tmp/owner-recurrence-47-318wwe08/diagnostic.json`,
SHA-256 `bce169f9c0bb95edf3b26977c4eb4da3376a9d842e72b494a3569d16760a530a`.
Raw logs, per-process records, tick artifacts and plists remain beside it.
Host macOS 26.6.2 (25G83); 30-second interval, zero throttle; elapsed 153.253s.
Each row below includes all three GUI variants; B/A/I are baseline/abandon/Interactive.
Every row: AC power; lowpowermode=0; therm reports no thermal warning, performance
warning or CPU power status recorded. GUI on-demand count=1, session=Aqua;
user session=Background. “Pending” means pended nondemand spawn=interval.

| UTC 2026-09-17 | Interval | GUI completed B/A/I | GUI state B/A/I | User completed | User state |
| --- | ---: | --- | --- | ---: | --- |
| 04:19:02.720 | 0 | 1/1/1 | idle/idle/idle | 1 | idle |
| 04:19:32.766 | 1 | 1/1/1 | pending/pending/pending | 2 | idle |
| 04:20:02.786 | 2 | 2/1/1 | idle/pending/pending | 3 | idle |
| 04:20:32.788 | 3 | 2/1/1 | pending/pending/pending | 4 | idle |
| 04:21:02.725 | 4 | 2/1/1 | pending/pending/pending | 5 | idle |
| 04:21:32.706 | 5 | 2/1/1 | pending/pending/pending | 5 | running |

Idle/pending samples have active count 0 and last exit 0; the last user sample has
active count 1. Its fifth interval attempt was in flight at cleanup and is not
counted as completed. At interval 3 the earlier ps sample caught user PID 75925
still running; its later completion record proves exit, not a surviving descendant.
User completion count includes one kickstart plus four interval firings. Entry gaps:
30.621512, 30.546785, 30.543452, 30.517456 seconds. Measured instrumented GUI tick
durations 0.088828–0.320380s; user 0.226968–0.290738s, not live-workload bounds.
All four bootouts returned 0 and subsequent prints returned absence (113).
Live GUI owner label independently verified absent.

Initial attempt preserved at `/private/tmp/owner-recurrence-47-pjecv6xh/diagnostic.json`
(SHA-256 `ee452d8716043af033039900d1b03f54f4cc1ec7b9369eb7344687c6b1b1dc27`).
Three GUI variants each ran once and pended. User bootstrap without an explicit
Background session failed EIO; it is a fixture defect, not a failed recurrence.
The replay records the corrected session and per-interval log/domain capture.
The stream yielded no diagnostic events; log show supplied the causal records.

Recurrence now requires two successive successful interval-attributed runs at the
existing timing bounds. Attribution checks launchd's current running PID, immediate
reason=interval, owned plist path/hash; it accepts no caller-provided firing flag.
A different PID is manual, a kickstart is other, unavailable diagnostics are unknown.
Old observations lacking attribution cannot qualify. launchctl print is a diagnostic,
not a stable API: unknown output fails closed, and no activation authority is inferred.
Two real baseline kickstarts 33 seconds apart both reported non-ipc demand;
tick.json has firing=other and previous_success=null, so cannot prove recurrence.
Uninstall retains tick/refusal records and now renders never-installed alongside
owner_run_refused. Publication exceptions retain the successful tick (or actual
refusal), separately return publication_error, and persist a failure count/class.
If local error recording also fails, stdout says publication_recorded=false.
A later successful publication carries the error history; a broken publisher cannot
deliver fresh dashboard data during its outage. Unit tests cover both outcomes.

Final source files match the copied diagnostic runtime byte-for-byte:
owner_daemon.py SHA-256 `e02f0917736dfafbce8cae502dbc8876d80578e446a2b2bc724f55d17eb2cf40`;
decision_feed.py `be5eac5d1a7df67afb03dda730be4b34c1e095c450f6de112270113b67af97cb`.
Focused tests: 18/18 in 1.119s. First complete suite: 732 passed, 3 failed of 735
in 455.934s (owner-recurrence-47-suite-final.log). Three unchanged TMUX tests timed
out with blank panes; their isolated replay passed 3/3 in 1.419s without edits
(owner-recurrence-47-exit-replay.log). Full-suite replay result follows below.
Disposable venv: /private/tmp/owner-recurrence-47-venv; only pinned requirements
markdown-it-py 3.0.0, mdurl 0.1.2 and slack-sdk 3.44.1 installed beyond venv tooling.
All tests use env -i, synthetic HOME/all profile aliases, no bytecode and private
temporary fixtures. The first full attempt was interrupted (exit130) after noticing
the wrong denial variable name; its log is owner-recurrence-47-suite.log.
No native prompt occurred. The corrected full run uses CORBANU_TEST_NO_NATIVE_KEYRING=1,
venv/system/Homebrew PATH, and unittest discover -v -s scripts/initiative_control -p test_*.py.
No Rust/native test or workspace formatter ran. The prime descendant hypothesis
was not borne out. A host-wide StartInterval failure would also be the wrong
conclusion: the user/Background comparison fired. The narrower GUI-domain failure
is explained by on-demand-only mode, not a literal non-Aqua session mismatch.

Full replay: **734 passed, 1 failed of 735**, 479.324s, exit1; no skips.
Log: /private/tmp/owner-recurrence-47-suite-replay.log. The remaining failure was
test_tmux_ingress_during_ack_preserves_native_final_fence: blank-pane startup timeout,
before the test's fence assertion. Its isolated replay passed 1/1 in 2.960s
(/private/tmp/owner-recurrence-47-ingress-replay.log). No code changed between runs.
The full-suite gate remains RED; TMUX source/tests are unchanged and outside scope.
Final checks: both governance checkers and git diff --check pass; scope is clean.
Changed lines (additions + deletions): 68 production + 111 evidence = 179 outside
tests; 109 unit-test + 148 diagnostic-test = 257 inside tests; 436 total.
