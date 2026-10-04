# owner-recurrence-41 — increment E

Frozen brief SHA-256 verified: `f9ae1bc1f0c301dfa10f899a1622e30eb7ae69f8327efb6dbcac176fdf1e0999`.
Base: `a6c471a351cc35c4417fdef75e0797ca4ea6be98`, already descends from
`529454cf7`. Product initiative in active plan `initiative-delivery-control`,
PF-80-S01 (`in_progress`). Product heading **Internal delivery control — TO BUILD**:
“durable event dispatch, acknowledgments and watchdog”; “Show blockers, rendered
sprints, human test plans, machines, run logs and freshness”. The worktree and
branch are registered; the sprint checker passes (115 current, 127 archived).

## Design frozen before implementation

1. Thirty-second launchd StartInterval, explicit initial kickstart, retained 30-second
   ThrottleInterval. Three seconds conflicts with that throttle; 300 seconds
   delays ACK/RETURN/watchdog observation. This is best-effort cadence, not a
   guarantee for leases shorter than the interval.
2. No overlapping ticks and no catch-up. launchd skips firings while running;
   a separate nonblocking file lock also fences direct invocations. A durable
   start precedes kernel admission, and completion follows it. Ninety seconds
   without completion means stalled. ExitTimeOut=30 is shutdown grace, not a
   runtime deadline. Interrupted ticks require explicit recovery.
3. Global refusal durably latches HOLD. Scheduled probes continue indefinitely
   but do not enter the kernel until explicit recovery with recorded evidence.
   Retain first/last refusal, last success and skipped-probe count; reinstall
   does not clear the latch. Per-action holds retain existing isolation.
4. Separate owner recurrence from Slack-supervisor health. Running requires
   fresh service observation and tick evidence. Stalled exposes age, service
   absence or HOLD/error. Never installed requires verified service absence
   and no installation receipt. Unavailable/stale observation is unknown with
   a reason. Publishing does not reset observation timestamps.

Activation resolves all four template placeholders with plist encoding, pins an
explicit canonical Python executable by supplied SHA-256 and version, disables
ambient Python environment/site imports, and pins the private runtime files and
configuration. Check owner UID, modes and no symlink components. Install writes
an intent receipt before bootstrap; identical reinstall verifies the loaded job
and does not clear health state. Uninstall verifies the exact owned service,
boots it out, verifies absence and removes the plist while preserving audit and
HOLD records. Test only a disposable label/store; never enable the live job.

Dashboard transport uses an optional separately validated owner projection in
the existing pinned snapshot. `decision_manager.py` remains untouched, respecting
the brief's explicit prohibition despite its presence in the writable list.
This is implementation/scheduling evidence, not independent functional acceptance
or an unqualified human-test handoff. The manager retains the independent
functional gate and plan/sprint ledger updates, outside this worker's scope.

## Activation contract and evidence

Entry points: `activate.py --owner install --root S --runtime R --config C
--python P --python-sha256 SHA`; `activate.py --owner uninstall --root S`.
S and R are existing private directories; R contains the reviewed flat runtime
files, C is the private existing owner configuration. P is an explicit canonical
interpreter, checked against supplied SHA before version invocation. The receipt
pins version, executable, runtime files and config; `-E -s -S -B` excludes ambient
Python environment, user/global site packages and bytecode writes. Standard-library
files are not independently hashed. The four placeholders resolve to P, R, C
and S/logs by parsed plist serialization, including paths containing XML symbols.
Private paths use existing UID/mode/no-link guards; logs are 0600.
The GUI-session job is explicitly bootstrapped, not installed for automatic login.
A lost job requires inspection/uninstall/reinstall; no automatic effect retry.

`owner_daemon.py --schedule S --recover "inspection evidence"` records recovery;
it does not resolve operation holds or change kernel authority. `--observe
--schedule S --publish-state PUBLISH_STATE` exports separate owner health for
the normal pinned dashboard transfer. The publisher must refresh this observation;
unavailable/stale input remains unknown. The rendered page labels its snapshot,
observation time and 90-second expiry; it does not implement a browser-side timer.
Running exposes fresh service/tick evidence, stalled exposes age and hold/error,
never-installed requires verified absence without a receipt, unknown gives a reason.
Slack-supervisor health and decision_manager remain unchanged.

**Real scheduling qualification FAILED; do not enable the live job.** The final
Background trial used label
`com.corbanu.initiative-owner.test-owner-recurrence-41-x1v69nk9`, store
`/private/tmp/owner-recurrence-41-x1v69nk9/state`, interval/throttle 2 seconds,
and synthetic fixture-only authority. Its `qualification.json` in the parent
directory records one initial completion, no second tick after 105 seconds,
and actual dashboard `stalled / tick-overdue / age 105`. Identical double-install
preserved the receipt. Double-uninstall removed the plist, verified service
absence, and observed unchanged tick state for five seconds (`uninstall_reversed:
true`). HOLD/recovery unattended execution was not reached; unit evidence only.

Raw harness: `/private/tmp/owner-recurrence-41-qualification.py` (originally executed
from this evidence directory). Earlier retained fixture roots with the same
`/private/tmp/owner-recurrence-41-` prefix: `c865c7up` (35-second timeout),
`d5iody4z` (180-second Background timeout), `86fuo04z` (Standard timeout),
`ho_lowss` (Interactive timeout and preserved original/corrected health probes).
`qy7asi_g` failed before bootstrap on an underscore in the fixture label; the
harness sanitized it. All loaded disposable jobs were removed. Process-type
experiments did not fix recurrence; Background is retained. launchd reported
`pended nondemand spawn = interval`; no root cause is asserted. The final changes
after that trial only reject missing/unsafe installation receipts and orphaned
tick state; successful final-tree scheduling remains explicitly unqualified.
Read-only checks found the live label absent in gui/501, user/501 and system.
No live job, coordinator, Slack, credential, push or release action occurred.
The manager owns the next scheduler investigation and functional acceptance.

## Automated evidence

Read test-isolation guidance before tests. Created a new venv with
`python3 -m venv /private/tmp/owner-recurrence-41-venv`, then installed only
`scripts/initiative_control/requirements.txt`: markdown-it-py 3.0.0, mdurl 0.1.2,
slack-sdk 3.44.1. Final campaign uses an empty environment, fixture HOME and all
three profile aliases, TMPDIR=/private/tmp, PYTHONDONTWRITEBYTECODE=1, and absolute
PYTHONPATH to this checkout's scripts/initiative_control; venv/bin leads PATH.
Command: `python -m unittest discover -s scripts/initiative_control -p 'test_*.py'`.
The final tree collects 727 tests; completion is recorded below. All 11 added
tests pass in a focused run. The unmodified detached base at
`/private/tmp/owner-recurrence-41-base-wt` passes all 48 owner-TMUX tests.
Earlier logs are retained under `/private/tmp/owner-recurrence-41-*`: the archive
baseline lacked Git metadata; initial campaigns omitted PYTHONPATH/TMPDIR;
an interrupted campaign was contaminated by its cancellation handler. Those runs
are not qualification. No existing tests were removed. Governance and diff checks pass.

Final suite: **727/727 passed in 441.798s**, exit 0; remaining failure names: none.
Log: `/private/tmp/owner-recurrence-41-qualified-env-suite.log`.
Per-user-domain diagnostic `7zqoaazl` failed bootstrap with code 5; cleaned up,
not retried with elevated authority. Log: `/private/tmp/owner-recurrence-41-user-domain-probe.log`.
Diff: 225 test lines; 397 outside tests (277 code/template, 120 evidence); below hard 400.
