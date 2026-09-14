# Owner daemon increment A — internal kernel

A crashed owner must not repeat an uncertain effect. This increment records
intent and receipts separately, recovers bookkeeping, and holds missing proof.
It is an internal fixture kernel, not an installed or qualified recurring owner.

Classification: product initiative, active plan
[initiative-delivery-control](../../../docs/plans/active/initiative-delivery-control.md),
PF-80-S01 (in_progress). Product heading: **Internal delivery control — TO BUILD**;
excerpt: “durable event dispatch, acknowledgments and watchdog”.
Allocation: owner-daemon-impl-01; manager disposition limits this to design
owner-daemon-design-01 Increment A. Governance baseline: 78164c613.
Worker: bootstrap/owner-daemon-20260914, worktrees/bootstrap-owner-daemon-20260914.
Only the four allocated files are changed; no service was installed or enabled.

## Entry and authority contract

Import, help and default invocation are read-only; the CLI reports OFF.
Explicit offline Python setup(config_path) creates schema version 1, OFF state
and locks beside an already initialized Coordinator. It never creates Coordinator.
No setup, arm, pause, install or transport-selection CLI is exposed.
Configuration has exactly coordinator, worktrees, package_digest, manager_enabled.
Worktrees must exist without symlink components. Private files require owner UID,
mode 0600 and one link; state directories require owner-only access.
The package digest binds owner_daemon, coordinator, manager_cycle and fable_launcher.
The eight design tables retain their specified columns; schema drift is refused.
SQLite uses DELETE journaling and FULL synchronization. Existing recovery sidecars
cause refusal. Runtime does not migrate schemas or repair files.

A fixture caller must explicitly inject FixedTestAdapter. No dynamic adapter,
command string, launcher or fixture mode can be selected from CLI/configuration.
Even an armed CLI --run refuses because no live transport exists.
An armed fixture tick also requires private activation.json with decision_id,
positive revision/generation, nonempty recorded authority, scope fixture-only,
config_digest and package_digest. The meta row must match its digest and pins.
The test harness supplies synthetic authority; it does not authorize live use.
No public activation writer exists yet; offline tests bind meta explicitly.

## One tick and recovery

The owner flock spans the tick; the admission flock spans authority checks and
bounded fixture work. Both validate existing lock-file identity and permissions.
Each admitted boot records actual host boot identity, PID and process start.
RECOVERING replays unfinished fixture receipts through Coordinator.event, whose
exact-content deduplication also handles a crash after the domain commit.
Missing receipts become durable holds; an uncertain observation is never repeated.
Requests, receipts, observations and holds have private fsynced artifacts.
A previous activation's unfinished operation holds rather than importing authority.
An owned manager defers ordinary replay; watchdog still checks its deadline.
Watchdog retains core stall semantics and never reconciles or replaces a worker.
READY means kernel recovery finished for this fixture scope only.
An enabled, unowned Coordinator admits ACTIVE and one synthetic observation/event
per activation generation, with intent persisted before invoking the fixed adapter.
Later idle ticks do not change Coordinator revision or invoke the adapter again.
Paused Coordinator suppresses new observations; draining/OFF refuses admission.
manager_enabled reports deferred readiness; it never invokes Fable.
Prepared/flagged work remains unchanged except for Coordinator watchdog flags.
No ACK, START, RETURN, worker dispatch, verification or acceptance is implemented.
Processes is reserved schema only; no synthetic process is presented as a worker.

## Limits and evidence

The noninstalled plist is disabled, with RunAtLoad/KeepAlive false. Later reviewed
packaging supplies recurrence and control; this one-tick kernel exits.
No live TMUX, Slack, Git writes, pushes, approvals, credentials, self-modification,
financial actions, manager inference, lifecycle transitions or publication exist.
Same-UID trusted-owner files are not malicious-worker isolation. Increment B adds
real transport/provenance; C adds effects; D adds Slack; E adds packaged supervision.
True-TUI, live TensorCash/Isometric workflows and independent confined acceptance
are deferred to those functional increments. Internal-only N/A requires integrator
acceptance; this return makes no human-test, release or recurrence-readiness claim.
Validation commands/results are recorded in the worker RETURN. The focused suite
uses real subprocess exits at intent, receipt and domain-commit boundaries, plus
activation, filesystem, locks, pause, watchdog, deduplication and disk-failure cases.

Validation September 14: focused 18 passed; full SDK suite 514 passed in 275.028s (HTTP-fixture cleanup warnings; no failures or retries).
The earlier 16-test pass exposed unclosed SQLite warnings; explicit connection cleanup was fixed before the final 18-test run.
Full command: `PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=scripts/initiative_control:/Volumes/CorbanuDrive/Corbanu/.codex-work/initiative-control.oGQGyA/venv/lib/python3.14/site-packages /Volumes/CorbanuDrive/Corbanu/.codex-work/slack-sdk-test.Ob3i5O/venv/bin/python -B -m unittest discover -s scripts/initiative_control -p '*test*.py'`.
Plan/sprint checkers passed (3 active plans; 116 current/126 archived sprints); plist lint and staged diff checks passed.
