# owner-recurrence-35 — design stop before implementation

Outcome: STOP under the frozen brief's explicit instruction, “If the design
requires any of these, stop and report.” The required dashboard distinction
cannot be delivered through the allocated files. No implementation or scheduler
installation was attempted. This is a design finding, not a qualified candidate.

Allocation: `owner-recurrence-35`; claim
`8c6ecacc-9cce-44c0-9128-acf2edbf5cb7`; allocation digest
`03fd1f35a2083b3c118cb857e3e86ebe3fb27e83d7c7c196495621f0e2c648a7`.
The supplied brief was read first; `shasum -a 256` returned exactly
`fb2c7c388b0c2ca47eefb39a3101e2e34775312182329047cf57e5c1318ba3ed`.
Base/HEAD: `08db99fff46fd22c582fbea0240fa379a42242e7`.
Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/owner-recurrence-20260917`;
branch: `bootstrap/owner-recurrence-20260917`. Worker: GPT-6 Astra, high effort.

Classification for the proposed implementation: product initiative, active plan
`docs/plans/active/initiative-delivery-control.md`, PF-80-S01 (`in_progress`).
Product heading: **Internal delivery control — TO BUILD**; excerpts: “durable
event dispatch, acknowledgments and watchdog” and “Show blockers, rendered
sprints, human test plans, machines, run logs and freshness”. This allocation's
worktree is not yet listed in that plan; manager-owned record reconciliation is
also outstanding. No policy or plan files were changed.

## Blocking interface finding

- `owner_daemon.py:511` runs one tick and prints its result. Its durable `boots`
  rows start only after admission. Admission refusal can therefore produce the
  specified `owner_run_refused` HOLD without a boot row.
- `decision_manager.py:128` projects Slack listener disclosure, including the
  separately validated `supervisor_health`. Its health vocabulary, binding and
  five-second freshness rule describe that listener, not the owner scheduler.
- `decision_feed.py:275` explicitly constructs dashboard Slack health from
  selected fields. It does not read owner state, launchd state or an installation
  receipt. `control.py:441` publishes this projection, and `status.js:60`
  updates publisher freshness. None supplies owner-schedule freshness.
- `native_owner.py` is a bounded native handoff protocol, not a consumer of the
  dashboard projection. The existing `activate.py` installs the Linux publisher
  services; it does not provide a dashboard recurrence extension point.

A new local heartbeat alone would leave dashboard viewers unable to distinguish
running, stalled and never installed. Reusing Slack supervisor health would
mislabel another subsystem and overwrite its evidence. A generic run report
cannot establish the explicit never-installed state or current launchd presence.
A compliant design therefore needs an explicitly allocated recurrence projection
and rendering change, at minimum in the dashboard feed/rendering pipeline and
its tests. If attached to existing manager status, it also needs the prohibited
`decision_manager.py` change. Both routes exceed this assignment's scope.
The manager should allocate that interface separately or amend the scope before
implementation; no competing edits were made to the other worker's files.

## Proposed policy, frozen for manager review (not implemented)

1. Use `StartInterval=30`, `KeepAlive=false`, with an explicit initial run on
   installation. Thirty seconds matches the retained launch throttle and keeps
   observation delays below a multi-minute manager cycle. Three seconds fights
   the 30-second throttle and increases process/SQLite work without providing
   that cadence. Three hundred seconds delays ACK/RETURN and watchdog observation
   substantially and can consume short worker leases. Existing allocations allow
   leases as short as one second: this is a best-effort observation cadence, not
   a guarantee for all leases. Qualification must include representative leases.
2. Do not overlap ticks or kill one merely for exceeding an interval. The local
   `launchd.plist(5)` manual states that interval firings during an active job,
   and while asleep, are missed. The existing owner lock also refuses overlap.
   `ThrottleInterval=30` limits launches; `ExitTimeOut=30` is the SIGTERM-to-SIGKILL
   grace when stopping, NOT a tick runtime deadline. Record tick start before
   admission and completion afterward. At 90 seconds without completion, show
   stalled/overdue even if a PID survives. Do not replay missed ticks or infer
   recovery of an uncertain effect; require operator inspection of a stuck tick.
3. On global `owner_run_refused`, latch a durable schedule HOLD immediately.
   Continue only a cheap scheduled status probe every 30 seconds until explicit
   owner recovery; do not repeatedly enter the kernel. Preserve first/last
   refusal and reason, last success, skipped probes and recovery receipt. A
   per-action HOLD is different: preserve existing isolation and continue
   observing unaffected work without retrying held effects. Reinstallation must
   not clear a hold. If status persistence fails, emit a nonzero exit and private
   stderr; independent freshness expiry must still expose the failure.
4. Dashboard contract: running requires fresh installer/service observation and
   completed tick evidence (or a first tick still within its 90-second window).
   Stalled shows the last completion, current start/age, service presence and
   explicit hold/error. Never installed requires a fresh observation of absent
   service plus absent install receipt. An old receipt with absent service means
   stopped/uninstalled, not never installed; missing/unreadable observations mean
   unknown. Publisher refresh must never refresh the original tick timestamp.
   These are proposed states; today's dashboard does not implement them.

## Packaging and reversal contract (not implemented)

Resolve `@PINNED_PYTHON@` to an explicit absolute interpreter in a dedicated,
versioned runtime, with its executable digest and version recorded and verified
before bootstrap and ticks; never use ambient PATH or a moving Homebrew alias.
Pin its dependency environment as well; an absolute pathname alone is not an
immutable interpreter. Resolve `@PRIVATE_RUNTIME@`, `@PRIVATE_CONFIG@` and
`@PRIVATE_LOGS@` to caller-supplied absolute private paths using plist encoding,
not shell interpolation. Check owner UID, directory mode 0700, file mode 0600,
regular-file/link identity and no symlink components, following existing guards.
Record runtime/config/interpreter digests in a private installation receipt.

Proposed install/uninstall entry points belong in `activate.py`, independently
of its Linux publisher path. Install must preflight everything, atomically write
an owned plist/receipt, bootstrap only the exact label, and verify the loaded
job. Identical second install must be a verified no-op; differing pins or an
unrelated existing label must refuse. Interrupted installs need explicit
reconciliation, not overwrite. Uninstall must boot out that exact owned service,
verify absence, remove its plist, and retain a stopped receipt and audit logs.
It must not delete coordinator state, terminate unrelated jobs or erase holds.
No CLI, rollback proof, double-install proof or uninstall proof exists yet.

## Observations, checks and qualification limits

Read-only checks on 2026-09-17 around 01:36 UTC: `launchctl print` for
`gui/501/com.corbanu.initiative-owner`, `user/501/com.corbanu.initiative-owner`
and `system/com.corbanu.initiative-owner` each returned 113, service not found.
The live job was not loaded in those domains; this worker did not enable it.
No live coordinator store, profile or credential was read or changed. No Slack
message, service mutation, push or release occurred.

There is no disposable label/store or real scheduling observation: execution
stopped at design as the brief permits. Future qualification must use a disposable
label, synthetic armed store and the actual pinned package under launchd; observe
multiple distinct durable ticks without a caller loop, hold, overrun, interruption,
restart, identical reinstall and bootout followed by absence of further ticks.
Keep real scheduling proof separate from full independent functional acceptance.

`docs/development/test-isolation.md` was read. The read-only sprint checker passed:
`sprints: current 115; archived 127`. No automated test campaign was started;
no virtualenv was built; Python tests executed: **0**. Consequently the required
pinned-requirements PF-80-S01 suite and base-attributed TMUX cases are unrun,
not passed. Remaining test failure names: none observed (no test execution).
No human-test readiness, independent acceptance or recurrence qualification is
claimed. The manager retains those gates and plan/sprint reconciliation.

The brief's single-tick/template observations are correct. The missing assumption
is that its allowed files can deliver dashboard recurrence visibility. Increment
D's listener-supervision proof does not establish that connection: the relevant
projection lives in other files. Adding an interval also does not make
`owner_daemon.py` invoke Fable management or supervise Slack; its existing
`manager_enabled` path reports deferred, and those responsibilities remain
separate. The 30-second exit timeout must not be treated as a runtime cap.
