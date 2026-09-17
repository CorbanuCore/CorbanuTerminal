# owner-recurrence-43 — corrective worker return
Base: `46b555f2d673deb42e875b59ef36dda12a877f9b`; frozen brief SHA-256:
`115a2d6db7dc83575456378a14b2e7ee117e368d00eb83a5f56045fcbe6f96d9`.
Product-initiative revision within active initiative-delivery-control, PF-80-S01
(in_progress). **Internal delivery control — TO BUILD**: “durable event dispatch,
acknowledgments and watchdog”; “Show blockers, rendered sprints, human test plans,
machines, run logs and freshness”. Explicit worker allocation owns this checkout;
shared plan/sprint ledgers remain manager-owned. Both governance checkers pass.
This is implementation evidence, not independent functional/human acceptance.

Install now requires an explicit label; the exact live label additionally requires
`--confirm-live`. Bare install refuses before touching installation state or launchd,
including reinstall with a pre-existing receipt. No live job was enabled.
Install also requires `--publish-state` pointing to the publisher's private state.
The scheduled CLI writes owner-recurrence.json after every attempt, held probe
and recovery, and uninstall publishes verified absence. The existing pinned export
captures it. This is a wired producer, not an additional publisher or timer.
The dashboard says recurring-at-observation only with two successful completions
0.5–1.5 configured intervals apart and a success within three intervals at observation.
One kickstart remains stalled/recurrence-unproven. Published observations expire
after 3,600 seconds (two existing 30-minute publication periods); rendering explicitly
labels this coarse observation, never current liveness. No browser timer is claimed.
Explicit LaunchError/coordinator Rejected refusals latch until recovery; other
exceptions report ERROR and retry next firing. Total/consecutive errors and the
exception class are persisted and projected; exception contents are not exported.
Interrupted ticks still latch. Uninstalled receipts plus absent service produce
never-installed, even when old tick/HOLD history remains.

Qualification harness: test_owner_recurrence_43_schedule.py (synthetic authority,
ten measured real Kernel.tick calls, one install/kickstart, wait for four ticks,
always uninstall). It never accesses credentials or starts inference workers.
First corrected trial: /private/tmp/owner-recurrence-43-btnfbk46/qualification.json.
Measured kernel maximum 0.021015250s; full scheduled tick 0.172513962s.
Interval 30s, throttle 0s (launchd reports minimum runtime 1s). Only kickstart
2026-09-17T03:21:14.719414Z occurred in 140s. No interval-driven firings.
Launchd still reported pended nondemand spawn = interval, runs=1, exit=0.
Uninstall verified absence, removed plist and no later tick. Live GUI label absent.
The assertion that a tick exceeds two seconds is contradicted by this fixture's
measurements. Equal throttle/interval is not a sufficient diagnosis: zero throttle
still failed. Thirty-second recurrence is unqualified on this host/mechanism;
there is no workaround or claim of three consecutive firings.

Final-tree schedule rerun: /private/tmp/owner-recurrence-43-aag6blg0/qualification.json.
Ten kernel calls max 0.015793709s; scheduled tick 0.164659977s, only kickstart
2026-09-17T03:23:56.529398Z; zero interval firings in 140s, same pending reason.
The automatic projection was stalled/recurrence-unproven; uninstall produced
installed=false/service=absent and verified cleanup. These observed fixture maxima
are not an upper bound on arbitrary live workloads. Both raw attempts are retained.
Initial focused run:
13/14 passed; the remaining assertion still expected the superseded 90-second
snapshot expiry. Corrected to check the actual 3,600-second budget. Final focused
run: 15/15 passed in 0.937s, including explicit coordinator rejection.
Disposable venv /private/tmp/owner-recurrence-43-venv contains only requirements.txt
dependencies (markdown-it-py 3.0.0, mdurl 0.1.2, slack-sdk 3.44.1).
Runs use env -i, synthetic HOME/all three profile aliases, bytecode disabled,
TMPDIR=/private/tmp, fixture keyring denial and checkout-only PYTHONPATH.
First full attempt: 730 tests, 58 failures/2 errors, 408.589s; retained at
/private/tmp/owner-recurrence-43-suite.log. The restricted PATH omitted Homebrew,
hiding node and tmux. Two node errors were independently reproduced in
/private/tmp/owner-recurrence-43-path-diagnosis.log; TMUX tests refused missing
real TMUX, and launcher fixtures returned launcher_failure. This is a harness
failure, not a passing suite. Final run adds /opt/homebrew/bin:/usr/local/bin
after the venv and system directories, with dependencies/profile isolation unchanged.
One added coordinator-refusal test brings final discovery to 731 cases.
No Rust tests, workspace formatter, external message, push or release occurred.
Manager retains independent functional execution/evidence review and final receiving
gates; this worker return does not qualify a human-test candidate or complete S01.

Final complete suite: **731/731 passed in 459.950s**, exit 0, log
/private/tmp/owner-recurrence-43-suite-final.log. No failures or skipped tests.
Final command: venv python -m unittest discover -v -s scripts/initiative_control -p test_*.py.
Diff scope: production 89 changed lines, unit tests 107, qualification harness 99,
and this evidence 74. Tests/harness total 206; other lines 163. Counting the harness
conservatively outside unit tests gives 262, above target 200 but below hard 320.
