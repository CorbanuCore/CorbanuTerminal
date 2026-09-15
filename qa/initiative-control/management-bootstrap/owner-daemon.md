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

## Increment C — admitted worker lifecycle (owner-daemon-impl-04)

This section supersedes A/B's deferred worker-routing statements only. Product
initiative under active initiative-delivery-control / PF-80-S01 (in_progress).
Product heading: **Internal delivery control — TO BUILD**; excerpt: “fresh Fable
5.1 High management through Corbanu/TMUX; durable event dispatch, acknowledgments
and watchdog”. Frozen base: `94ea8948e801b9ee9ab723af23e6154841dce5fd`.
Worker branch: `bootstrap/owner-daemon-c-20260915`; worktree:
`/Volumes/CorbanuDrive/Corbanu/worktrees/bootstrap-owner-daemon-c-20260915`.
Allocation digest: `0ad7b4e7a50ce49900b5dd956f6c4e9671957a64e53a1943bf748735811a0ebb`.
The dispatched brief hash was verified:
`9494143ad87e0863d39d049fd3fc73d40deaa4a8f16f5badde4279dde98b791b`.
Manager owns reconciliation of these coordinates into plan/sprint records;
this worker has no write allocation for those files.

### Established by internal tests

- Explicit TMUX routing still requires OFF-to-armed offline state and private,
  digest-bound activation with recorded authority. Its scope must now be exactly
  `tmux-workers`; fixture-only authority cannot launch a worker. There is no
  activation writer, service installation or automatic enablement.
- Prepared worker actions require frozen `inputs.worker` with exactly `model`,
  `provider`, `effort`, `worktree`, `policy`. Worktree must be in the pinned
  daemon config and policy exactly `--yolo`. The existing transport expresses
  that policy as danger-full-access / never; no permission prompts are approved.
  Worker kinds match NativeOwner's work kinds; passive waits and integration,
  acceptance, publication and sprint transitions are outside this increment.
- The assignment preserves kind/workstream/sprint/scope/inputs/timeout, adds fixed
  owner constraints, and uses B's private home, dedicated socket and explicit
  auth-link contract. The daemon never reads the linked authentication file.
  Its own code performs no Git push, Slack send, source edit or credential access.
  Prompt restrictions are instructions, not enforced malicious-worker isolation.
- One bounded pass advances every eligible action as far as current evidence
  permits; later ticks poll waiting workers. Startup requires the matching loaded
  model/effort pane. Exact completed assistant ACK is compared byte-for-byte;
  prompt echoes never qualify. Dispatched and acknowledged use existing core APIs.
  Core's existing `running` means ACK received: only the separate `working`
  operation proves START reached a correlated user-message and configured turn.
  Accepted key delivery alone leaves `awaiting_working`, never started.
- Completed RETURN still requires B's standalone marker and correlated second
  completed rollout turn. Pane/tool/partial output does not qualify. The daemon
  records `returned`; it never calls acceptance/verification on the work result.
- Claim, prepare, launch, prompt, dispatch, ACK, START, working observation and
  returned steps persist requests/intents before effects and receipts afterward.
  A receipt persisted before the owner DB commit can resume bookkeeping.
  A crash with a missing receipt becomes HOLD, including after a core commit;
  no launch/key/core-mutation retry is inferred. Existing foreign claims,
  changed activation, binding/receipt drift and failures stay held.
- The claim's fixed deadline is the worker lease. Polling a PID never extends it;
  a surviving PID at expiry is recorded as lease-expired. Missing ACK and START
  without work get specific holds. TMUX additionally rejects zombie process state.
  Pause/manager ownership defers effects and does not hide an expired lease.
- Each action has its own exception/hold boundary; another eligible action can
  finish in the same tick. A tick with unresolved holds still reports HOLD.
  Global admission/storage failure remains fail-closed. No automatic hold
  resolution, worker replacement, forced termination or cleanup is inferred.

### Validation and live-tick gaps

Initial focused run: 31 tests, 11 failure reports from an incorrect process-table
placeholder count (including cascading subtest assertions). Fixed the 20/19-column
mismatch; the next run passed all 31. Expanded focused run passed 64 tests
in 22.923s, including actual private TMUX with harmless shell fixtures and a real
Python process exiting between launch effect and owner receipt. These are
synthetic worker/model/activation records, not live model proof. The first full
SDK run passed 586 tests in 311.407s (HTTP-fixture cleanup ResourceWarnings only).
Final review then restored READY reporting for paused/manager-owned TMUX ticks
and added explicit assertions; the final-tree SDK replay is recorded below.

Daemon/test file counts: 507/678 against the sprint's documented 660/780 limits.
Transport/test file counts: 362/533; no separate per-file ceilings were specified
in the received transport contract. Full SDK command/result is recorded below
and in the worker RETURN.

**Live-tick qualification and activation remain manager/Travis gated.**
Before a qualified live tick the manager must receive/review this exact tree,
reconcile allocation records, pin the actual Corbanu binary and qualify its
startup pane and rollout/session/runtime formats, supply approved frozen worker
allocations and isolated auth transport, and provide the explicit activation
record/authority through the separately controlled activation process. This
worker created no activation outside disposable tests and ran no live tick.

Combined functional handoff still needs fresh code-blind cases, independently
confined exact-package execution (including negative filesystem/process/IPC/
network/credential probes), raw actual-key success/failure/recovery/resume
evidence, independent evidence review, and applicable live-repository proof.
Internal-only N/A is proposed for this implementation fixture stage because no
operator workflow was qualified; integrator acceptance and that later combined
functional gate remain required. Same-UID metadata/private homes and process
sampling do not enforce containment against an arbitrary --yolo worker.

Manager inference, result verification/receiving integration, worker shutdown
qualification, Slack activation/decision ACK, packaged recurring supervision,
named-human acceptance, benchmarks and release evidence remain separate work.
No actual model, live credential, Slack, launchd, TensorCash/Isometric, human-test,
recurrence-readiness or release qualification is claimed.

Increment C initial final-tree full SDK: **586 tests passed in 298.911s**, with only the retained
HTTP-fixture cleanup ResourceWarnings and no failures or native credential prompt.
Requested isolated TMUX replay: **27 tests passed in 19.788s**; actual class is
`test_owner_tmux.TmuxTests`. Its command uses the identical environment/interpreter
below with `-B -m unittest test_owner_tmux.TmuxTests`. No TMUX flake reproduced.
Final governance: plan checker 3/3 active; sprint checker 116 current/126 archived;
`git diff --check` passed. Only the five dispatched paths changed.
Exact full SDK command (also used for the review corrections below):

```sh
env -u CODEX_HOME -u CORBANU_HOME -u PFTERMINAL_HOME PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=scripts/initiative_control:/Volumes/CorbanuDrive/Corbanu/.codex-work/initiative-control.oGQGyA/venv/lib/python3.14/site-packages /Volumes/CorbanuDrive/Corbanu/.codex-work/slack-sdk-test.Ob3i5O/venv/bin/python -B -m unittest discover -s scripts/initiative_control -p "*test*.py"
```

## Increment C review corrections (owner-daemon-impl-05)

Classification: bounded fixes to the existing increment C contract, under product
heading **Internal delivery control — TO BUILD**; requirement excerpt: “durable
event dispatch, acknowledgments and watchdog”. Existing initiative/sprint context
and manager-owned record reconciliation remain as recorded above. Frozen base:
`ef0091cb0c6432f5d88eab64a39da7cca6d4792b`; branch/worktree unchanged.
Allocation digest: `17469381469a636d37bbe1501d584e23456df91a5e7e90dc279b64bcb9835d5f`.
Verified brief SHA-256:
`d0b61cd7b994c61e69f67c5dd113e4cb4ce71664a43150ea88cadf182ea681f7`.

- **Finding 1, P2, corrected:** selection uses the same explicit worker-kind set
  as dispatch validation. Prepared waits and other manager-owned action kinds
  remain untouched and unclaimed; they create no daemon hold or operation.
  `test_non_worker_actions_remain_unclaimed_across_healthy_ticks` covers all
  eight excluded kinds alongside a supported worker action across two healthy ticks.
- **Finding 2, P3, corrected:** a pause or manager takeover observed by the gate
  after intent commit records a `deferred` phase before unwinding. This phase
  means the external effect was never attempted. On resumption it revalidates
  authority/request bindings, commits intent again, and checks the gate before
  performing. Existing launched workers retain their process/operation records;
  resumption continues without launching or sending a completed delivery again.
  `test_pre_effect_pause_or_manager_ownership_defers_and_resumes_without_relaunch`
  covers launch, prompt and START windows for both pause and manager takeover.
  `test_deferred_retry_crash_restores_uncertain_intent_and_never_relaunches`
  verifies a crash during the resumed effect remains an uncertain, fenced intent.
  A crash before the deferral commit also retains the existing uncertain-intent
  refusal; no general missing-receipt retry or hold-resolution path was added.
- **Finding 3, P3, corrected:** a valid correlated durable RETURN permits result
  bookkeeping after pane death; missing/invalid RETURN still requires an alive
  worker. `test_daemon_records_durable_return_after_pane_is_killed` writes a
  synthetic completed rollout in a private shell fixture, kills that actual pane,
  and proves two daemon ticks record/preserve `returned` without sending keys.
  Further key delivery to the dead pane remains refused. Identity, ACK, START
  correlation, receipts and the fixed claim lease retain their existing checks.

No existing safety refusal is weakened: default OFF and recorded activation
remain required; no ACK, START, RETURN or verification is inferred; uncertain
effects, binding drift, dead workers without RETURN and expired leases still
hold. No live profile, native credential access, live tick, activation outside
fixtures, push, Slack, service installation or release was performed.
The transport implementation needed no change.

This is internal regression evidence. The later combined code-blind functional
and exact-package/live-tick gates above remain open; internal-only N/A still
requires integrator acceptance. No human-test or release readiness is claimed.

Validation attempts are retained separately: the first focused run executed 39
cases in 7.343s with two fixture errors (missing completion-allocation metadata
and cleanup ordering). The durable RETURN assertions passed before cleanup
failed. The test-owned private TMUX server was identified by its exact fixture
worktree and stopped; its process exit was confirmed. After repairing both
fixtures, the same 39-case command passed in 7.511s. The retry-crash regression
was then added before the final full SDK run.

Current per-file counts: owner_daemon.py **531/660**, test_owner_daemon.py
**745/780**, owner_tmux.py **362** (unchanged), test_owner_tmux.py **586**.
No separate transport/test ceiling was specified.

Final full SDK run: **589 tests passed in 301.548s**, using the exact command
above. Only the existing HTTP-fixture cleanup ResourceWarnings (500/429) appeared;
there were no failures, TMUX load flakes, or native credential prompts.
No isolated TMUX retry was needed. The sprint checker passed (116 current,
126 archived), and `git diff --check` passed. The focused command used the same
environment/interpreter with `-B -m unittest test_owner_daemon
test_owner_tmux.TmuxTests.test_daemon_records_durable_return_after_pane_is_killed`.
Only the four allocated files listed by the final diff changed.

## Receiving verification — September 15, 2026

Increment C and its corrections were received together at `572356137`. The first
verification attempt failed for two independent reasons, both resolved before
acceptance:

- A manager-side regression in `manager_cycle.briefing`. `manager_cycle_test.BriefingSizeTests`
  encodes the contract that `evidence_omissions` enumerates every reference whose
  body was not expanded; a filter that skipped digests already visible on the
  retained record broke six of those cases. Reverted in `d0d93c75a`: the list
  reports what the manager cannot read, not which strings happen to appear.
- The receiving harness ran with the default macOS `TMPDIR` under `/var/folders`,
  whose length exceeds the 104-byte Unix socket path limit, so 27 owner-daemon
  lifecycle cases failed with `socket_path_too_long`. Workers always run under a
  short private `TMPDIR`, which is why the worker observed 589 green on the same
  code. **Receiving test commands must pin `TMPDIR=/private/tmp`.**

Re-verified on the merged tree: **591 passed in 299.284s**. The daemon remains OFF
with no activation record; live-tick qualification stays manager and Travis gated.

## Startup determinism follow-up — owner-daemon-impl-06, September 15, 2026

Product initiative revision within the existing PF-80-S01 (in_progress), active
initiative-delivery-control plan. Product heading: **Internal delivery control —
TO BUILD**; excerpt: “durable event dispatch, acknowledgments and watchdog” and
“initialize and rehearse all three workstreams before enabling recurring
operation.” Exact assignment base: `1e33bb7bfa4a74d9081e96a5e65ed4d40014d995`;
branch/worktree remain `bootstrap/owner-daemon-c-20260915` as recorded above.
Manager owns reconciliation of these assigned coordinates into the shared
plan/sprint ledgers; this revision does not edit them.

The real transport now provisions mode-0600 `home/config.toml` before launch,
disabling startup updates, TUI animations and analytics and trusting only the
exact binding worktree. TOML quoting preserves spaces, dots, quotes, backslashes
and Unicode without adding project entries. Launch passes the manager's three
matching `-c` startup overrides. Sandbox and approval arguments still come
unchanged from the binding; the existing auth link and prompt refusal remain.
No credential/authentication/permission prompt is answered or suppressed.

The dated follow-up in
[the live qualification receipt](owner-daemon-live-qualification-20260915.md#follow-up--september-15-2026-owner-daemon-impl-06)
records the exact source/package pins, unassisted ACK/START/working/RETURN,
natural fixed-lease expiry during an actual running tool, preserved PID and
HOLD/no-relaunch evidence, separate final-tree SDK run and fixture cleanup.
The original startup failure and narrower expiry demonstration remain intact.

This is internal implementation and code-aware live qualification, not a
combined human-test or recurrence handoff. Independent review/receiving,
plan/sprint reconciliation, confined code-blind functional acceptance and the
remaining integration/Slack/supervision gates still apply. Internal-stage N/A
requires integrator acceptance; no activation approval is inferred.
