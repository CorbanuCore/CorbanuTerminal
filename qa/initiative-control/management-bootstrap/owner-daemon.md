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

## Increment D — listener supervision and disclosure, September 15, 2026

Allocation `owner-daemon-supervision-01`; Astra High implement worker.
Allocation digest:
`a9e01168b465d696444b8e8a85e04059dc12d2c2c64385628828c1b39c56266a`.
Frozen brief SHA-256 verified before implementation:
`1fbff736e084948ddbe9b18a988efa7749047fae79a9b9e61b9eea77f842cf05`.
Base `68e07dc8e19e12cad4b88a81392056b55b256aaa`, branch
`bootstrap/owner-daemon-c-20260915`, worktree
`/Volumes/CorbanuDrive/Corbanu/worktrees/bootstrap-owner-daemon-c-20260915`.

Classification: bounded reliability fix restoring the already-authorized
watchdog/disclosure contract. Product heading: **Internal delivery control —
TO BUILD**; requirement excerpt: “durable event dispatch, acknowledgments and
watchdog” and “Show blockers, rendered sprints, human test plans, machines, run
logs and freshness”. Existing initiative context remains active
initiative-delivery-control / PF-80-S01 (`in_progress`); shared plan/sprint record
reconciliation belongs to the manager. No new posting authority is granted.

### Incident and implementation

At 12:22 UTC on September 15, the manager-owned listener died and remained down
for roughly two hours. Its supervisor noticed the child exit but only stopped
the process. The fence retained 45 callback marks against 42 journal ingresses:
three callbacks never reached the journal. Ordinary projections did not explain
the gap. The missing-fence inspection could not apply because the fence existed.

1. **Supervision:** the foreground supervisor records each unexpected exit in
   durable `transport.listener_events`, including return code, observation time,
   fence and ingress counts, exact difference (or unknown when unreadable),
   lifecycle epoch, retry count and pending/held disposition. Status and dashboard
   health expose `listener_exits` and `last_listener_exit`. Up to three restarts
   follow nonblocking 1/2/4-second backoff per explicit start. Reaping still uses
   the owned process handle. A restart preserves the finite run deadline and
   never clears a transport hold. Exhaustion, missing fence, nonzero gap, stopped
   session or changed binding/lifecycle stops retrying visibly. The parent
   rechecks the captured binding/lifecycle before spawn; the child rechecks under
   the transport lock immediately before creating its new session, closing the
   intervening stop/rebind window. Explicit stop, repair quiescence and owner EOF
   retain their stop behavior.
2. **Disclosure:** `project_status`, `project_slack` and dashboard health carry
   exact `fence_gap`; a nonzero gap reports `held`, including when the cached
   assessment is stale. Projection never changes ingress, the fence, unknown
   arrivals or qualification. Existing optional-field status records remain
   readable. `inspect_fence_loss` and missing-fence recovery are unchanged.
3. **Pending pointers:** status/feed/health expose `pending_pointers`. Once a
   supervised pass is admitted, it retries only an existing pending follow-up
   notice with a retained request and sent parent/details. It does not create a
   notice for a follow-up that has no retained request. The retry rechecks
   identity/cancellation and requires the reconstructed request to equal the
   retained request before entering the existing transport exchange. A recorded
   receipt is reused; a recorded attempt without receipt becomes uncertain and
   cannot produce another Slack message. Holds retain the pending request.

### Discriminating regressions and mutation receipts

All six new cases are in `test_decision_manager.ManagerTests`. They use private
stores, synthetic SDK/web/socket factories and loopback fixture endpoints. The
incident and restart-budget cases kill and reap actual dedicated listener
children; deterministic clocks control only backoff. These are implementation
regressions, not independent functional qualification.

Mutation method: load the real module source, assert the mutation target occurs
once, compile the modified source into that module's namespace, execute the
named unittest, then compile the original source back into the same namespace
and run the named unittest again with fresh fixtures. Require mutant failure
and restored success. Dedicated SDK children remain routed through the existing
synthetic factory. No on-disk production mutation, credential access, external
Slack endpoint or live store is involved.

| Named case (prefix `test_`) | Mutation | Broken outcome | Restored outcome |
| --- | --- | --- | --- |
| `listener_incident_records_exit_and_three_unknown_arrivals` | Replace `self.record("child-exit", code)` with `pass`. | ERROR: `KeyError: listener_events`; durable exit absent. | PASS; actual SIGKILL is recorded as -9, fence 45, ingress 42, gap 3; held, no restart, missing-fence inspection still refuses. |
| `listener_restarts_with_backoff_and_exhausts_after_three` | Change retry eligibility `self.restarts < 3` to `< 0`. | FAIL: next retry is None instead of 1. | PASS; new owned sessions after 1/2/4 seconds, four distinct sessions total, then held. |
| Same restart case | Change retry eligibility `< 3` to `< 4`. | FAIL: retry remains scheduled after the third restart. | PASS; no fourth restart. |
| `listener_restart_rechecks_stopped_epoch_and_binding` | Skip parent `s.restart_allowed(...)`. | FAIL in all four subcases: unsafe restart attempted after stopped phase, rebind, epoch change or new gap. | PASS in all four subcases. |
| Same restart-pin case | Skip `restart_allowed(...)` inside `Session.__init__`. | FAIL in all four subcases: expected Invalid not raised. | PASS; child-side check refuses and leaves lifecycle unchanged. |
| `gap_projection_and_dashboard_health_preserve_exact_count` | Set projected `fence_gap` to zero. | FAIL: held/0 differs from held/3. | PASS; exact count and unchanged journal. |
| Same gap case | Remove `project_disclosure` from `project_slack`. | FAIL: feed reports gap 0 instead of 3. | PASS; feed reports held/3. |
| Same gap case | Replace dashboard health's gap with zero. | FAIL: health reports gap 0 instead of 3. | PASS; fresh and stale health preserve held/3. |
| `supervised_pending_pointer_retry_is_admitted_and_exactly_once` | Set projected pending-pointer count to zero. | FAIL: 0 differs from 1. | PASS; pending count is 1 before admitted retry and 0 after. |
| Same admitted retry case | Remove the supervised call to `retry_pending_pointers`. | FAIL: retained slot remains pending instead of sent. | PASS; hold blocks send; admitted retry reuses receipt; uncertain recorded attempt never reposts. |
| Same admitted retry case | Disable existing transport `if attempt in value["posts"]` deduplication. | FAIL: six Slack fixture messages instead of five. | PASS; exactly five messages, including one pointer. |
| `supervised_pointer_posts_only_retained_approved_slot` | Iterate no pending slots in the retry helper. | FAIL: retained slot remains pending instead of sent. | PASS; no request means no new notice; retained approved request sends once. |

Initial focused run: **6 passed in 5.526s**. All twelve mutation variants failed
as expected and their restored named runs passed. Retained mutation attempts:
the first child-side mutation exposed a fixture-only leaked flock after its
expected stopped-session assertion failure, causing three secondary lock errors.
The fixture now releases an unexpectedly constructed session before failing;
a fresh replay produced four clean assertion failures, then passed restored.
The admitted retry case was extended to cover an uncertain recorded attempt and
replayed broken/restored successfully. The incident case was replayed after
adding latest-exit disclosure and again failed broken/passed restored.

### Validation and deliberately open work

Final-tree full suite: **663 tests passed in 383.406s**. Only the retained
HTTP-fixture cleanup ResourceWarnings for synthetic 500/429 responses appeared;
no failure or native credential prompt occurred. Plan checker passed (3/3 active,
0 available); sprint checker passed (116 current, 126 archived).
Final `git diff --check` passed. Six allocated files changed.
Command (short TMPDIR is required for the Unix socket fixtures):

```sh
env -u CODEX_HOME -u CORBANU_HOME -u PFTERMINAL_HOME TMPDIR=/private/tmp PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=scripts/initiative_control:/Volumes/CorbanuDrive/Corbanu/.codex-work/initiative-control.oGQGyA/venv/lib/python3.14/site-packages /Volumes/CorbanuDrive/Corbanu/.codex-work/slack-sdk-test.Ob3i5O/venv/bin/python -B -m unittest discover -s scripts/initiative_control -p "*test*.py"
python3 docs/plans/check.py
python3 docs/sprints/check.py
git diff --check
```

Recurrence remains OFF; no activation is recorded. No live Slack message, real
operator store access, credential read, prompt handling, push or release was
performed. Gap repair remains deliberately open: these arrivals are unknown,
and neither counter advancement nor fabricated intake is an acceptable repair.
Restart after a store-stopped epoch is deliberately refused even when the
process exit was unexpected. Hold review/requalification stays owner-controlled.

This is an offline implementation return to the manager, not a human-test,
recurrence or release handoff. Independent review/receiving and the later confined
code-blind exact-package functional gate remain open. No true TUI, live
TensorCash/Isometric, named-human acceptance or benchmark result is claimed;
this internal test stage exercises Python supervision and synthetic Slack
delivery, not a packaged product workflow. The integrator must accept this
stage's limited evidence and arrange applicable functional qualification before
declaring the combined operator workflow ready.

## Increment D review correction — owner-daemon-supervision-02, September 15, 2026

This round preserves the original increment D record above. Its candidate
`c9bb3f3ea4e82368e5f674d49b17e8bfc6e58e90` received an independent Opus 5.0 High
review with overall confidence 0.60 and verdict “patch is incorrect”: two P2
blockers and two P3 findings. The worker read the complete frozen review at
`/private/tmp/fmgr.Q1SIYZ/odsup-review.json` before editing.

Allocation digest:
`02e228a24f440c1cabe51a766a1f00f840d6f3f9b16aa71c25d213700abba66f`.
Claim: `465d86ff-492a-4341-8805-2999573e0932`.
Brief SHA-256 verified with `shasum -a 256`:
`f71e7e9ce19f0fc9929667c1593ff60e35ef02f76f5304668354af3b7ca291a5`.
Worker: gpt-6-astra / high. Branch and worktree remain
`bootstrap/owner-daemon-c-20260915` and
`/Volumes/CorbanuDrive/Corbanu/worktrees/bootstrap-owner-daemon-c-20260915`.

Classification: bounded reliability correction to the existing increment D
contract. Product heading: **Internal delivery control — TO BUILD**; excerpts:
“durable event dispatch, acknowledgments and watchdog” and “Show blockers,
rendered sprints, human test plans, machines, run logs and freshness”.
Existing initiative context: active initiative-delivery-control / PF-80-S01
(`in_progress`); shared plan/sprint reconciliation stays with the manager.

### Per-finding corrections

- **P2-1 — watchdog exception escape:** `tick` catches operational store/lock,
  validation and subprocess timeout failures. An observed death stays in
  `pending_event` until its write succeeds, before the handle is reaped.
  The observed handle is retained separately so a reap timeout cannot append
  another exit on the next tick. Restart failure stores a pending refusal
  before trying to write it; a failed write retries on the next tick. Explicit
  start first flushes retained evidence. Errors remain sanitized; no raw SDK or
  process exception is published.
- **P2-2 — stale pending disclosure:** the latest listener event is published
  even when it is a refusal; `listener_exits` still counts actual child exits.
  Validation accepts a refusal only with a null return code and held restart.
  Every refusal now ends retry eligibility, including a start failure with an
  otherwise unchanged binding and lifecycle. Status, feed and health are
  checked for stopped session, rebind, epoch change, new fence gap and start
  timeout.
- **P3-1 — idle contention:** the helper reads the atomic alerts file before
  taking any lock. An empty approved-pending set returns immediately. A nonempty
  hint is rechecked under the store lock before transport admission. The
  supervisor compares the alerts file's inode, nanosecond mtime and size:
  unchanged idle stores get no repeated scans or transport admission calls.
  Existing pending work retains bounded retry polling and existing notice/
  transport checks. No callback locking or financial/posting authority changes.
- **P3-2 — growing restart frame:** the pin contains binding plus the digest of
  the complete lifecycle. Parent and child compare that same bounded identity.
  No history is pruned; a history edit still invalidates the pin. A real child
  restart with 200 retained historical sessions crosses the existing bounded
  Stdio channel successfully and leaves 201 history entries.

A permanently unavailable journal cannot record evidence; the foreground
supervisor retains its observation in memory and keeps retrying. This does not
claim crash durability for evidence whose first write has never succeeded.

### Fresh regressions and mutation evidence

Four new cases extend `test_decision_manager.ManagerTests`; the existing
restart-pin case now also checks all three disclosure consumers. Focused run:
**8 passed in 8.149s**. Each mutation below was compiled into the real module
namespace in a separate fixture test process; its unique source target was
checked before replacement. The named case ran broken, the original source was
restored in memory, and a fresh instance of the named case ran again. Production
source was never mutated on disk. All fixtures use private stores and synthetic
Slack endpoints; dedicated children use the existing injected fixture launcher.

Case names below have the `test_` prefix and `ManagerTests` owner.

| Mutation | Named case | Broken result | Restored |
| --- | --- | --- | --- |
| Re-raise the operational exception at the tick boundary. | `listener_record_failures_retry_observation_and_reap_once` | ERROR: Invalid from real transport-lock contention. | PASS |
| Remove the already-observed-process check. | Same record/reap case | FAIL: 3 exit records instead of 1 after the failed reap. | PASS |
| Remove TimeoutExpired from restart's inner catch. | `listener_restart_timeout_retries_refusal_write_then_stays_held` | FAIL: last event remains child-exit/pending. | PASS |
| Record restart refusal directly without retaining pending intent. | Same timeout/refusal case | FAIL: failed disk write loses refusal; child-exit/pending remains. | PASS |
| Permit a refusal to satisfy restart eligibility. | Same timeout/refusal case | FAIL: restart-refused/pending instead of restart-refused/held. | PASS |
| Project the latest child-exit instead of the latest listener event. | `listener_restart_rechecks_stopped_epoch_and_binding` | Four FAILs: stale pending event for stopped, binding, epoch and gap cases. | PASS |
| Call gate before the empty-pointer hint. | `idle_pointer_watch_does_not_acquire_callback_lock_or_rescan` | ERROR: BlockingIOError on the actual held callback lock. | PASS |
| Unconditionally rescan at each pointer interval. | Same idle case | FAIL: 21 helper calls instead of 2 (including direct probe). | PASS |
| Replace lifecycle digest with the full lifecycle in the shared pin builder. | `listener_aged_history_restarts_through_bounded_control_frame` | FAIL: no restarted process; oversized frame refused. | PASS |
| **Replacement 1:** suppress the actual ingress fence append (`os.write`). | `gap_projection_and_dashboard_health_preserve_exact_count` | FAIL: last-verified/0 instead of held/3 after three real mark requests. | PASS |
| **Replacement 2:** route stale health ahead of the held-gap state. | Same gap/health case | FAIL: stale/3 instead of held/3; count remains intact. | PASS |
| **Replacement 3:** select sent notice slots instead of pending slots for retry. | `supervised_pointer_posts_only_retained_approved_slot` | FAIL: approved retained notice remains pending instead of sent. This case does not assert a projected count. | PASS |
| Suppress retaining the unexpected child exit. | `listener_incident_records_exit_and_three_unknown_arrivals` | ERROR: missing listener_events after actual killed child. | PASS |
| Change retry budget from <3 to <0. | `listener_restarts_with_backoff_and_exhausts_after_three` | FAIL: retry_at None instead of 1. | PASS |
| Change retry budget from <3 to <4. | Same restart-budget case | FAIL: retry_at 15.0 instead of None after three retries. | PASS |
| Skip parent restart_allowed. | `listener_restart_rechecks_stopped_epoch_and_binding` | Four FAILs: unsafe start attempted. | PASS |
| Skip child Session restart_allowed. | Same restart-pin case | Four FAILs: Invalid not raised; fixture releases unexpected session. | PASS |

The three bold replacement rows supersede the earlier “set projected value to
zero” mutations as discriminating evidence. The earlier receipts remain above
as historical attempts. Incident, both budget boundaries, parent and child
restart checks were retained and rerun. The four other earlier mutations are
also replayed in this round:

| Retained mutation | Named case | Broken result | Restored |
| --- | --- | --- | --- |
| Remove feed's project_disclosure call. | `gap_projection_and_dashboard_health_preserve_exact_count` | FAIL: held/0 instead of held/3. | PASS |
| Suppress the supervisor's retry helper call. | `supervised_pending_pointer_retry_is_admitted_and_exactly_once` | FAIL: retained slot remains pending. | PASS |
| Disable the transport's existing-attempt deduplication. | Same admitted retry case | FAIL: 6 fixture Slack messages instead of 5. | PASS |
| Iterate no pending slots. | `supervised_pointer_posts_only_retained_approved_slot` | FAIL: retained slot remains pending. | PASS |

**21/21 mutants failed as intended; all 21 fresh restored runs passed.**
The full mutated and restored unittest outputs were retained in this allocation's
tool transcript. No mutant survived, no unexpected fixture failure required a
retry, and no native credential prompt occurred. After correcting a comment
escape, the three pointer cases passed again: **3 tests in 2.779s**.

### Validation and remaining gates

Full suite: **667 tests passed in 391.447s**, using the exact pinned
`TMPDIR=/private/tmp`, profile-alias-unset SDK command recorded for increment D
above. Only the known synthetic HTTP 500/429 cleanup ResourceWarnings appeared;
there were no suite failures, retries or native credential prompts.
Both governance checks passed: plans **3/3 active, 0 slots available**; sprints
**116 current, 126 archived**. Final `git diff --check` passed. Only five allocated
files changed: decision_manager, decision_alerts, slack_transport, the manager
tests and this evidence record.

Recurrence stays OFF. This allocation adds no activation,
service installation, live Slack message, real-store/credential read, push,
release or approval. It is an offline implementation return for manager
review/receiving. Gap repair, shared plan/sprint reconciliation, integrator
acceptance of internal-stage N/A, and later confined code-blind exact-package
functional acceptance remain open. No combined human-test readiness, true-TUI,
live TensorCash/Isometric, named-human acceptance or benchmark pass is claimed.

## Increment D final bounded correction — owner-daemon-supervision-03, September 15, 2026

The two earlier supervision rounds above remain historical evidence. This round
addresses only P3-1 and P3-2 from the independent Opus 5.0 High review of
`2335d1eb95c3b639399226a58bccf5b038cf9f06`: verdict “patch is correct”,
confidence 0.72, no blocking findings. Frozen review:
`/private/tmp/fmgr.Q1SIYZ/odsup2-review.json`.

Allocation digest:
`43620288609c27c48b4448dbd843f160da3fd523d8abd2fc6807944196a6d531`.
Claim: `f82499fe-bf01-48fb-83bf-b3ce904deb91`.
Brief SHA-256, verified before other reads:
`11abf9ca7652da384d2a454ac72cc397bffa1bdcb41f9a148d3ec5b1bed2d8fa`.
Worker: gpt-6-astra / high. Branch:
`bootstrap/owner-daemon-c-20260915`. Worktree:
`/Volumes/CorbanuDrive/Corbanu/worktrees/bootstrap-owner-daemon-c-20260915`.

Classification: bounded reliability correction to the authorized increment D
watchdog. Product heading: **Internal delivery control — TO BUILD**; excerpts:
“durable event dispatch, acknowledgments and watchdog” and “Show blockers,
rendered sprints, human test plans, machines, run logs and freshness”.
The manager's frozen brief explicitly requests the health disclosure and bounded
retention correction. Existing context remains active
initiative-delivery-control / PF-80-S01 (`in_progress`); shared governance
records stay with the manager.

### Finding dispositions and retention contract

- **P3-1 addressed:** operational failure inside an event flush increments a
  consecutive-failure counter and returns to the tick. Reaping continues despite
  persistent disk/lock/validation failures, and a running listener's pointer
  checks continue. Automatic and explicit starts wait for the pending event's
  successful write, so a restart refusal cannot overwrite unrecorded evidence.
  Once storage recovers, recording and the existing bounded restart sequence
  resume. A successful flush clears the counter.
- The foreground `status` response includes `supervisor_health`: explicit
  `healthy`/`unhealthy`, `event_flush_failures`, and `pending_events`.
  Any failed flush reports unhealthy immediately; enabled status is held.
  This is live process observation, not a claim that an unavailable journal
  recorded it. Separate file-only status/feed consumers cannot observe an
  unwritten failure; the foreground status command can. The pending observation
  remains volatile until the write succeeds, as disclosed in prior rounds.
- `listener_events` retains at most **128 newest records**, pruning more oldest
  records if needed to fit the Store envelope within `MAX_BYTES`. Pruning and
  appending share one atomic journal write. The newest record is never pruned.
  Other journal sections are untouched; if even the newest event and summary
  cannot fit, the flush stays pending and unhealthy.
- Pruning deliberately discards each removed event's individual kind, timestamp,
  return code, restart count, fence/ingress/gap counts, epoch and restart state.
  A durable cumulative `listener_events_pruned` summary retains the number of
  discarded events, number of discarded child exits, and timestamp of the most
  recently discarded record. Status exposes the discarded-event count, and
  `listener_exits` includes discarded child exits, so pruning cannot hide their
  occurrence. Full details survive only for retained events.
- **P3-2 addressed:** projection normalizes the known legacy
  `restart-refused/pending` event to `restart-refused/held` in a copy. The
  unreleased predecessor could write that contradictory shape; a refusal must
  not imply restart eligibility. The journal is not rewritten and the validator
  remains strict for current records. Regression covers status, feed and health.

### Regressions and mutation receipts

Five new cases belong to `test_decision_manager.ManagerTests`; case names below
omit the `test_` prefix. The original record/reap regression now explicitly
injects reap timeouts while journal operations fail, preserving its once-only
exit assertion while permitting successful reaping when storage is unavailable.

Initial focused run: **7 passed in 10.341s**, including the two prior error-path
cases. The persistent-failure case was then extended through the real foreground
pipe command loop, and the pointer case through explicit-start refusal.
Every mutation runs compiled production source in memory, then restores that
source and runs a fresh fixture. No production file is mutated on disk.
Dedicated SDK children use the existing synthetic launcher and private stores.

| Mutation | Named case | Broken outcome | Restored |
| --- | --- | --- | --- |
| Re-raise the event flush error. | `persistent_flush_failure_reaps_reports_and_recovers_restart` | FAIL: killed child handle remains instead of being reaped. | PASS |
| Suppress the failure counter increment. | Same persistent/reap case | FAIL: healthy/0 instead of unhealthy/1. | PASS |
| Route foreground status directly to file-only projection. | Same persistent/reap case | ERROR: missing `supervisor_health` in the actual pipe response. | PASS |
| Return from the tick when flush fails. | `persistent_flush_failure_still_checks_pointers_without_overwriting_event` | FAIL: zero pointer helper calls instead of one. | PASS |
| Remove the pending-evidence automatic-restart guard. | Same pointer case | FAIL: retry budget consumed while evidence is still pending. | PASS |
| Remove the explicit-start successful-flush requirement. | Same pointer case | FAIL: attempts manager.start before durable evidence. | PASS |
| Raise retention limit from 128 to 100000. | `listener_event_pruning_keeps_newest_and_accounts_for_discarded_exits` | FAIL: four extra old events retained instead of newest 128. | PASS |
| Suppress discarded child-exit accounting. | Same retention case | FAIL: zero discarded exits instead of two. | PASS |
| Disable byte-budget pruning. | `listener_event_pruning_makes_room_at_store_byte_limit` | ERROR: actual Store.write rejects the oversized envelope. | PASS |
| Disable legacy refusal normalization. | `legacy_pending_restart_refusal_is_normalized_without_rewriting_journal` | ERROR: validate_status rejects legacy pending refusal. | PASS |

**Ten distinct mutation variants failed and their fresh restored runs passed.**
Raw broken/restored unittest output is retained in this allocation's tool
transcript. The first retention mutation also enlarged the fixture because it
used the mutated constant, failing in fixture setup at Store.write; that attempt
is retained but excluded as discriminating proof. The fixture now independently
creates 131 records. Fresh replay failed on the retained-event assertion and
passed restored; discarded-exit accounting was also replayed on the fixed
fixture. No native credential prompt occurred.

### Validation and remaining gates

Full final-tree suite: **672 tests passed in 397.039s**, with the exact
profile-alias-unset SDK command recorded for increment D above, including
`TMPDIR=/private/tmp` and `PYTHONDONTWRITEBYTECODE=1`. Only the known
synthetic HTTP 500/429 cleanup ResourceWarnings appeared; there were no suite
failures, retries or native credential prompts. The suite retained synthetic
transport evidence at `/private/tmp/isolated-transport-tests-mta9ki7k`.
Both governance checkers passed: plans **3/3 active, 0 slots available**;
sprints **116 current, 126 archived**. Final `git diff --check` passed.
Exactly four allocated files changed: decision_manager, slack_transport,
test_decision_manager and this evidence record.

Recurrence remains OFF. No activation, installation, live Slack message,
real-store/credential access, push, release or approval is performed or claimed.
This is an internal implementation return for manager review, not a combined
operator human-test handoff. Integrator acceptance of this stage's internal-only
N/A and the later confined code-blind exact-package functional gate remain open,
along with prior gap repair and shared plan/sprint reconciliation. This stage
changes Python supervision/error reporting and uses synthetic Slack fixtures;
it does not qualify a packaged interactive workflow. No true-TUI,
TensorCash/Isometric, named-human acceptance or benchmark pass is claimed.

## Increment D file-reader and exit-observation correction — owner-daemon-supervision-04, September 15, 2026

All three earlier supervision rounds remain historical evidence. This round
addresses only findings A/B in the frozen brief and
`/private/tmp/fmgr.Q1SIYZ/odsup3-review.json`.
Base: `04a1670e8bc26775a3e28b2fc70a21eb354c0b44`; branch/worktree unchanged.
Allocation digest:
`358a91c3d4652c3baa776fba8c80fc6750b6247c2a25b6541e921008247f2011`.
Claim: `354cb989-ddf0-483e-ad12-7d5c0ca1472b`.
Brief SHA-256 verified before other reads:
`efb8484944e616f9aa5ad3ed8051383b9c7ad0b3e46e738eaf392bfb00e65ef7`.
Worker: gpt-6-astra / high.

Classification: bounded fixes restoring the authorized increment D supervision
and disclosure contract. Product heading: **Internal delivery control — TO BUILD**;
excerpt: “durable event dispatch, acknowledgments and watchdog”.
Existing initiative-delivery-control / PF-80-S01 context and manager ownership of
shared governance records remain unchanged.

### A — supervision health reaches file-only readers

The supervisor writes a separate private, checksummed `supervisor.json`
observation using the existing Store atomic replacement and fsync path. This
small file has its own write path and is independent of the transport journal's
lock, validation and size failures. It binds the observation to the Slack
identity and includes `observed_at`, health state, pending-event count,
consecutive flush failures and a fixed reason. It contains no credentials or
raw exception text. Pending evidence is published before the journal attempt;
a failed attempt publishes `event-flush-failed`, and successful flushing
publishes healthy with zero pending events.

`project_disclosure` reads that observation; both `project_status` and
`decision_feed.project_slack` include it. The latter writes the existing
`decision-slack-status.json` cache, and `decision_feed.slack_health` carries
the observation into the dashboard health payload. Unhealthy observation holds
the projected status without inventing a journal hold or exit record.

The regression uses a separate Python interpreter for status, then reads the
actual projected cache and dashboard health. During the first failed flush all
surfaces report unhealthy / event-flush-failed / one pending event while durable
`listener_exits` remains zero, `last_listener_exit` remains null and the
transport journal remains unchanged from its synthetic no-hold baseline.

Missing, invalid or mismatched observations report unknown. A healthy observation
older than five seconds becomes unknown when projected; an observed failure is
not aged into apparent health. This is an observation cache, not restart
authority or a second durable event journal. A whole-filesystem outage can also
prevent its write: then a prior healthy observation becomes unknown on its next
projection after the five-second freshness window. Already published dashboard
snapshots still require the existing projection/publication cycle. Unavailable
storage cannot preserve new evidence, and file consumers cannot see an
unpublished observation instantly.

### B — retain the failed-start survivor's exit

A retry that fails after spawning retains the exact surviving process handle as
an unexpected-exit candidate, even if a successfully recorded refusal later
disables normal restart options. One additional pending-exit slot preserves
that child's observation behind the earlier pending refusal. Starts remain
blocked while either event awaits recording. After recovery the refusal is
followed by the child exit; the two facts never overwrite each other. Handle
identity prevents repeated observations. Reaping proceeds during storage failure.

The regression runs a real harmless Python child through ManagedListener.start,
injects handshake failure and all cleanup wait timeouts while suppressing fixture
terminate/kill effects, then restores real process methods, kills and reaps the
owned child. It covers storage recovery both before the child's exit and while
the refusal is still unflushed at exit. Both cases preserve exactly one refusal
and one subsequent child-exit record with return code -9 and fence/ingress
counts, keep restart held and consume no additional restart attempt.

### Regression and mutation evidence

New cases are in `test_decision_manager.ManagerTests`; names below omit
`test_`. Each mutation compiled altered production source in memory, ran the
named case, restored the original source and ran a fresh fixture. Production
files were never mutated on disk. Raw broken/restored outputs are retained in
this allocation's tool transcript.

| Production mutation | Named case | Broken outcome | Restored |
| --- | --- | --- | --- |
| Suppress independent observation publication. | `first_unflushed_exit_reaches_file_only_status_and_dashboard` | FAIL: unknown/zero instead of unhealthy/one. | PASS |
| Suppress disclosure's observation read. | Same file-reader case | ERROR: projected cache lacks supervisor_health. | PASS |
| Drop supervision health from dashboard payload. | Same file-reader case | FAIL: null observation. | PASS |
| Disable healthy-observation expiry. | `supervisor_observation_missing_stale_or_unwritable_is_not_healthy` | FAIL: healthy instead of unknown after both write paths fail. | PASS |
| Reinstate the pending-event guard on exit observation. | `failed_start_survivor_exit_is_recorded_after_pending_refusal` | FAIL: surviving child's handle never observed while refusal is pending. | PASS |
| Forget the failed-start process handle. | Same survivor case | FAIL: exit lost after refusal disables options. | PASS |
| Discard the queued child exit after recording refusal. | Same survivor case | FAIL: only two journal events instead of three. | PASS |

**Seven mutations failed as intended; all seven fresh restored runs passed.**
The initial five-case focused run passed four cases and failed the file-reader
fixture's no-hold precondition: existing fixture startup already records
outage-gap. The test now explicitly establishes a synthetic no-hold baseline
before injecting the first failed exit write. The corrected case passed, as did
all its later restored runs. No native credential prompt occurred.

### Final validation and retained limits

Initial full run: **675 tests in 401.149s, three errors**, all from existing
pointer fixtures whose stand-in managers omitted the binding present on real
ManagedListener instances. Those three fixtures now supply the synthetic PIN;
the focused replay passed **3 tests in 2.727s**. Initial run retained evidence:
`/private/tmp/isolated-transport-tests-mrs3epbg`.
Final full-suite replay: **675 tests passed in 401.610s**. Only the known
synthetic HTTP 500/429 cleanup ResourceWarnings appeared; there were no errors,
failures or native credential prompts. Final retained synthetic evidence:
`/private/tmp/isolated-transport-tests-uelaxaed`.
Both governance checkers passed: plans
**3/3 active, 0 slots available**; sprints **116 current, 126 archived**.
The full suite uses the existing SDK command above with profile aliases unset,
`TMPDIR=/private/tmp` pinned and bytecode writes disabled. Final `git diff --check`
passed. The commit is recorded in the worker RETURN.

Recurrence stays OFF. This allocation performs no activation, service install,
live Slack message, credential/profile read, push, release or approval.
This is an internal implementation return to the Fable manager. Integrator
acceptance of the internal-stage N/A and the later confined code-blind
exact-package functional gate remain open; no packaged operator workflow,
true-TUI, live-repository, human-test, benchmark or release qualification is
claimed. Only the five allocated implementation/test/evidence files are changed.

## Increment D heartbeat and publication correction — owner-daemon-supervision-05, September 15, 2026

All four earlier supervision rounds remain unchanged as historical evidence.
This round addresses only P2/P3 from the frozen brief and
`/private/tmp/fmgr.Q1SIYZ/odsup4-review.json`.
Base: `d02b460296e57c12fd4686f4f8587c59d99a4656`.
Branch: `bootstrap/owner-daemon-c-20260915`; worktree:
`/Volumes/CorbanuDrive/Corbanu/worktrees/bootstrap-owner-daemon-c-20260915`.
Allocation digest:
`97422a45854ee7edc49297786f3493fb6ff7ff50f658a1ee628258341a22cb53`.
Claim: `f15949d2-9824-4b08-b1f9-b99b95ee2555`.
Brief SHA-256 verified before other reads:
`c0fb2c798407a5a77b57df9ca6522cfa49629884a29b736f691dd263fa0b6e26`.
Worker: gpt-6-astra / high.

Classification: bounded fixes restoring the existing supervision/disclosure
contract. Product heading: **Internal delivery control — TO BUILD**;
requirement excerpts: “durable event dispatch, acknowledgments and watchdog”
and “Show blockers, rendered sprints, human test plans, machines, run logs
and freshness”. Existing initiative-delivery-control / PF-80-S01 context
and manager ownership of shared governance records remain unchanged.

### Finding dispositions

- **P2 addressed:** remember the last successfully published observation and
  skip an identical payload. The second-resolution timestamp still advances
  the heartbeat every second; health, pending-count and failure-count changes
  publish immediately within the same second. Failed writes do not advance
  the remembered observation, allowing a same-second recovery retry.
- **P3 addressed:** projection and `slack_health` publication now share
  `assess_supervisor_health`. Publication independently rechecks the original
  observation timestamp: healthy through age five seconds, then
  unknown / observation-stale. The Slack cache's separate 900-second rule is
  unchanged. Missing/future observation times cannot become healthy, observed
  failures remain unhealthy, and reading does not rewrite the cached snapshot.

### Bounded-write and mutation proof

Focused run: **3 tests passed in 7.122s**, including the existing
missing/stale/unwritable observation regression.

New P2 case:
`test_decision_manager.ManagerTests.test_idle_supervisor_bounds_durable_writes_and_keeps_fresh_transitions`.
With no listener ever started, 300 real supervisor ticks at a simulated 10Hz
across 30 seconds perform **30 actual Store writes and 60 actual fsync calls**
(one file and one directory fsync per write). Every tick reads the independent
observation and verifies healthy state with the current second's timestamp.
Pending, failed and recovered observations then add three immediate writes
within the same second. An identical tick adds zero writes. A failed heartbeat
is retried successfully at the same timestamp; a further duplicate adds zero.
After six seconds without a heartbeat, the reader reports observation-stale.
The bound applies to unchanged idle health; changing failure evidence still
publishes immediately.

New P3 case:
`test_decision_feed.FeedTests.test_published_supervisor_health_expires_before_slack_cache`.
A synthetic cached observation is exported, activated with service calls
mocked, and passed through actual local publication into `health.json`.
Ages 0 and 5 remain healthy; ages 6, 899 and 901 become unknown/stale.
The outer cache remains off through 899 seconds and becomes stale at 901.
The case also verifies unchanged input, missing/future timestamp rejection
and retained unhealthy observations.

Each mutation compiled changed production source in memory, ran its named
case, restored the original source and ran a fresh fixture. No production file
was mutated on disk. Raw attempts remain in this allocation's tool transcript.

| Mutation | Named case | Broken outcome | Restored |
| --- | --- | --- | --- |
| Remove identical-observation suppression. | New P2 case above | FAIL: 300 writes instead of 30. | PASS |
| Freeze the heartbeat timestamp at its first publication. | New P2 case above | FAIL: observed timestamp stays at second 0 when second 1 is expected. | PASS |
| Suppress all same-second observations, including transitions. | New P2 case above | FAIL: 30 writes instead of 32 after pending/failure transitions. | PASS |
| Remember the observation before its write succeeds. | New P2 case above | FAIL: 34 calls instead of 35; same-second retry skipped. | PASS |
| Copy cached health without publication reassessment. | New P3 case above | FAIL: healthy at ages 6, 899 and 901; missing timestamp also stays healthy. | PASS |
| Widen shared healthy validity from five to 900 seconds. | New P3 case above | FAIL: healthy at ages 6 and 899. | PASS |

**Six mutations failed as intended; all six fresh restored runs passed.**

### Final validation and retained limits

Initial full suite: **677 tests in 419.934s, one failure** in the existing
`test_fable_launcher.RealTmux.test_normal_management_vocabulary_round_trip`.
Its synthetic launcher returned `launcher_failure` with `launched: false`
after approximately five seconds; this is consistent with the existing
five-second binary-version probe timeout, but the receipt does not establish
the precise cause. No allocated production module is involved in that launch.
The unchanged focused replay **passed in 3.711s**. No launcher/test code was
changed. Initial suite evidence:
`/private/tmp/isolated-transport-tests-wjqsmcvo`.
Final complete unchanged-code replay: **677 tests passed in 408.776s**.
Known synthetic HTTP 500/429 fixture cleanup ResourceWarnings appeared;
there were no failures, errors or native credential prompts in the replay.
Final retained synthetic evidence:
`/private/tmp/isolated-transport-tests-pn5u8bz6`.
Both suite runs used the exact SDK command recorded above with profile aliases
unset, `TMPDIR=/private/tmp` pinned and bytecode writes disabled.
Both governance checkers passed: plans **3/3 active, 0 slots available**;
sprints **116 current, 126 archived**. Final `git diff --check` passed.
The evidence file's entire pre-round contents were verified byte-for-byte
against the base commit; this round only appends to the four earlier rounds.

Recurrence stays OFF. This allocation performs no activation of real services,
installation, live Slack message, credential/profile read, push, release or
approval. The activation exercised above belongs only to disposable test
fixtures with service subprocess calls mocked.
This is an internal implementation return to the Fable manager. Integrator
acceptance of the internal-stage N/A and the later confined code-blind
exact-package functional gate remain open. Python heartbeat I/O and cached
health publication are the scope of this correction; no packaged operator
workflow, true-TUI, live-repository, human-test, benchmark or release
qualification is claimed. Five allocated implementation/test/evidence files
are changed; the commit is recorded in the worker RETURN.

## Functional qualification fixes — owner-daemon-qualify-fixes-01, September 15, 2026

Allocation digest:
`b02cfbb95498a095e54bb732af10b4c290b7b60e6438e4146db2bbb2e6e874f1`.
Claim: `cda98ad1-b6e9-4702-8d60-b29693fae3fd`.
Worker: gpt-6-astra / high.
Base: `948454162e584d4516c9ce3e39dc7d2663904929`.
Branch: `bootstrap/owner-daemon-c-20260915`; worktree:
`/Volumes/CorbanuDrive/Corbanu/worktrees/bootstrap-owner-daemon-c-20260915`.
The brief was read first after verifying SHA-256
`0ff985919d6f60505f54e7fa2a32a254d0bac3fa36575b356d8b37440b3cce21`.

Classification: bounded fixes restoring the authorized watchdog and disclosure
contract. Product heading: **Internal delivery control — TO BUILD**; excerpts:
“durable event dispatch, acknowledgments and watchdog” and “Show blockers,
rendered sprints, human test plans, machines, run logs and freshness”.
The initiative-delivery-control / PF-80-S01 context and manager ownership of
shared governance records remain unchanged.

These defects were found by functional qualification **after 675 unit tests
and five independent review rounds had passed**, as reported in the frozen
brief. The subsequent heartbeat round separately recorded 677 passing tests
above. Neither earlier result qualifies these failed workflows.
Original qualification report:
`/Volumes/CorbanuDrive/Corbanu/.codex-work/functional-odsup.20260915/RETURN-odsup-qualify-01.md`.
It records 7 passing and 6 failing cases across 39 variants, including broken
harness prerequisites. This worker does not change those original results or
the separate harness work.

### Tests written before implementation

All names below belong to `test_decision_manager.ManagerTests`.
The first five-test run on unchanged production code ran in **10.432s** and
failed with **16 failures (including subcases) and one error**, reproducing every allocated
product defect. Raw output is retained in this allocation's tool transcript.

| Defect | New test | First observed failure | Disposition |
| --- | --- | --- | --- |
| D03 unknown gap | `test_missing_fence_discloses_unknown_on_every_projection` | Status, feed and dashboard returned zero for a missing fence. | Fixed: disclose null and held; the test also checks the persisted status cache and proves projection does not recreate the fence or rewrite the journal. |
| D03 refusal accounting/retry | `test_child_admission_refusal_is_not_an_exit_or_another_retry` | Stopped, binding, epoch, gap and missing-fence hazards each became another child exit; epoch also attempted another restart. | Fixed: pre-admission refusal has a dedicated child exit code, recorded as held restart-refused with null returncode. Existing admission checks remain enforced. |
| D09 retained request | `test_refused_pointer_different_request_keeps_retained_bytes` | No physical post, but the retained alerts bytes changed. | Reproduced; implementation blocked by writable scope. |
| D10 event time | `test_exit_timestamp_survives_delayed_journal_flush` | Durable exit time shifted from 12:00:00 to 12:00:10 after failed writes. | Fixed: pending observations retain the time of first observed exit; retries use that timestamp. Reaping proceeds during write failure and restart backoff begins after durable flush. |
| D14 dashboard counter | `test_dashboard_discloses_cumulative_discard_count_after_reopen` | Dashboard raised KeyError for listener_events_pruned while status/feed correctly reported four discards. | Reproduced; implementation blocked by writable scope. |

The first post-change five-test run still failed the binding hazard's two
assertions: runtime ownership validation precedes the transport gate. The
correction classifies its pre-admission refusal too, without bypassing that
validation. The focused D03/D10 replay then passed **3 tests in 7.730s**.
The unchanged successful-restart/backoff and failed-start-survivor cases also
passed in the first post-change run.

### Mutation proof for completed fixes

Each mutation compiled altered `decision_manager.py` source in memory,
ran its named test, restored the original module source and ran a fresh fixture.
No production file was mutated on disk. Raw attempts remain in the tool
transcript.

| Mutation | Named test above | Broken result | Restored result |
| --- | --- | --- | --- |
| Replace unknown gap null with zero. | Missing-fence projection | Four failures: status, feed, persisted cache and dashboard all show zero. | PASS, 0.514s |
| Count the dedicated refusal exit code as child-exit. | Child-admission refusal | Eleven failures: wrong kinds/counts across five hazards and an extra epoch retry. | PASS, 6.471s |
| Assign the flush time instead of retained event time. | Delayed-journal exit timestamp | One failure: timestamp moves forward ten seconds. | PASS, 0.723s |

All three mutations failed as intended and all three restored runs passed.
D09/D14 have no fix or mutation-success claim: their regressions remain failing,
without skips or expected-failure annotations.

### Scope blockers and retained limits

D09's source is `scripts/initiative_control/decision_alerts.py`:
`notice` overwrites the retained request and writes the sending state before
`retry_pending_pointers` compares the reconstructed request in its exchange
callback. A transport-only change cannot undo that already-durable rewrite.

D14's source is `scripts/initiative_control/decision_feed.py`:
`slack_health` explicitly constructs the dashboard dictionary and omits
`listener_events_pruned`. Manager-side status already contains the counter.

Neither source path is in this allocation's explicit five-path writable scope.
Both files remain unchanged. No indirect monkey-patch or weakened fence is used
to conceal the scope conflict. The manager must extend/reallocate these two
paths to finish the requested four-defect repair. Harness D04, D06, D09 positive
receipt and D10 reader-assertion problems remain assigned elsewhere.

Recurrence stays OFF. No real service activation, installation, live Slack
message, credential/profile read, native credential prompt, push, release or
approval occurred. Tests use disposable synthetic stores and loopback SDK
fixtures, with all profile aliases unset and TMPDIR pinned to /private/tmp.
This is an incomplete internal implementation return, not functional acceptance
or an unqualified human-test handoff. Independent confined exact-package replay
and evidence review remain manager-owned gates after all four fixes.
No true-TUI, live-repository, human acceptance or release qualification is claimed.

### Final validation

Full suite: **683 tests in 430.989s; 681 passed, one failure and one error**.
The failure is the new D09 retained-bytes regression. The error is the new D14
dashboard-counter KeyError. No other case failed, no tests were skipped or
marked expected failures, and no retry of the full suite was needed.
Known synthetic HTTP 500/429 fixture cleanup ResourceWarnings appeared.
Retained synthetic evidence:
`/private/tmp/isolated-transport-tests-e4e2bl8s`.

Exact command:

```sh
env -u CODEX_HOME -u CORBANU_HOME -u PFTERMINAL_HOME TMPDIR=/private/tmp PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=scripts/initiative_control:/Volumes/CorbanuDrive/Corbanu/.codex-work/initiative-control.oGQGyA/venv/lib/python3.14/site-packages /Volumes/CorbanuDrive/Corbanu/.codex-work/slack-sdk-test.Ob3i5O/venv/bin/python -B -m unittest discover -s scripts/initiative_control -p '*test*.py'
python3 docs/plans/check.py
python3 docs/sprints/check.py
git diff --check
```

Both governance checkers passed: plans **3/3 active, zero slots available**;
sprints **116 current, 126 archived**. Final `git diff --check` passed.
The evidence file's pre-round contents match the base commit byte-for-byte;
all four changed files are within the five allocated writable paths.
The commit is recorded in the worker RETURN. **HOLD: D09 and D14 remain unfixed,
and the full suite is not green.** Their fixes and mutation proofs require
allocation of the two source paths identified above.

## Functional qualification fixes — owner-daemon-qualify-fixes-02, September 15, 2026

Allocation digest:
`5e76fad67bc52fbeb1ef90f49ba3e35078b841724b53395a0abcedf9ca316c7d`.
Claim: `cef39218-840b-4c85-98df-c4159b562aad`.
Worker: gpt-6-astra / high.
Base: `791bb79cdffa47a6d4ab8b0e6f9ee4678b72d73b`.
Branch: `bootstrap/owner-daemon-c-20260915`; worktree:
`/Volumes/CorbanuDrive/Corbanu/worktrees/bootstrap-owner-daemon-c-20260915`.
The brief was read first after verifying SHA-256
`b771c0342bceea46ff0261a735d3b1a0ef3f54481707cf0afec6a94d6abc0cdc`.

Classification: bounded fixes restoring the existing retained-request and
pruning-disclosure contracts. Product heading: **Internal delivery control —
TO BUILD**; requirement excerpts: “durable event dispatch, acknowledgments and
watchdog” and “Show blockers, rendered sprints, human test plans, machines, run
logs and freshness”. Existing initiative-delivery-control / PF-80-S01 context
and manager ownership of shared governance records remain unchanged.

**All four defects (D03, D09, D10 and D14) came from functional qualification
after unit tests and five review rounds had passed.** The prior allocation
records the original 675-test result and qualification provenance. This round
continues from its D03/D10 fixes without reimplementing them, and resolves the
two explicitly expanded source-scope blockers. Earlier failed attempts and
qualification results above remain intact.

### Test-first reproduction and fixes

Both regressions were written before implementation in the prior allocation.
The unchanged-base replay here ran **2 tests in 2.782s**, with the same D09
retained-bytes failure and D14 dashboard `KeyError`.
Before editing production code, this round strengthened both cases and observed
**2 tests in 2.789s: three D09 subcase failures and one D14 error**.
All named cases below belong to `test_decision_manager.ManagerTests`.

- **D09 — `test_refused_pointer_different_request_keeps_retained_bytes`:**
  repeated supervisor/direct-notice/supervisor calls must preserve the complete
  alerts file byte-for-byte, perform no store writes and invoke no exchange when
  reconstruction differs from the retained pending request. The original code
  rewrote the request before the exchange callback refused it. `notice` now
  compares reconstruction with the retained request before changing state or
  writing; mismatch returns the unchanged pending slot for manager recovery.
  Existing request equality, admission, cancellation, identity, uncertain-send
  and receipt fences remain enforced. Matching requests retain the existing
  admitted-send path.
- **D14 — `test_dashboard_discloses_cumulative_discard_count_after_reopen`:**
  status, projected feed, persisted feed cache and dashboard must disclose four
  discarded events, then five after another pruning pass and reopen, while the
  total child-exit count stays 131. The original dashboard omitted the key and
  raised `KeyError`. `slack_health` now forwards `listener_events_pruned`, using
  zero for the legacy status shape that omits it, consistent with the existing
  optional-counter contract. The regression checks that compatibility case too.
  Journal pruning and cumulative accounting are unchanged.

The fixed focused run passed **4 tests in 5.034s**, including both strengthened
regressions and the existing
`test_supervised_pending_pointer_retry_is_admitted_and_exactly_once` and
`test_supervised_pointer_posts_only_retained_approved_slot`. These positive
controls prove matching retained requests still send once and uncertain
transport attempts do not repost.

### Mutation proof

Each mutation compiled altered module source in memory, ran its regression,
restored the original module source in a `finally` block and ran a fresh fixture.
No production file was mutated on disk. Raw attempts are in the tool transcript.

| Defect | Mutation | Broken result | Restored result |
| --- | --- | --- | --- |
| D09 | Remove the retained-request comparison and early return from `notice`. | Three retained-byte subcase failures, 1.064s. | PASS, 1.063s. |
| D14 | Remove `listener_events_pruned` from `slack_health`. | `KeyError: 'listener_events_pruned'`, 1.741s. | PASS, 1.747s. |

Both mutations were detected; both restored runs passed.

### Retained limits

No fence was weakened and no out-of-scope change was needed. D03/D10 production
code, transport authorization, journal retention policy and other qualification
harness defects were left unchanged. This worker has no allocation to change
shared governance records or perform the separate functional acceptance replay.

**Recurrence stays OFF.** No live Slack message, real service activation,
installation, live credential/profile read, native credential prompt, push,
release or approval occurred. Python tests use disposable synthetic stores,
private shell/TMUX fixtures and loopback SDK fixtures, with profile aliases
unset and `TMPDIR=/private/tmp`. No Rust test campaign is in scope.
This return provides internal regression evidence, not functional acceptance
or an unqualified human-test handoff. Independent confined exact-package replay
and evidence review remain manager-owned gates; no true-TUI, live-repository,
human acceptance or release qualification is claimed.

### Final validation

Full suite: **683 tests passed in 432.306s**, with no failures, errors, skips,
expected failures or full-suite retry. Known synthetic HTTP 500/429 fixture
cleanup ResourceWarnings appeared. Retained synthetic transport evidence:
`/private/tmp/isolated-transport-tests-lbgol48a`.

Exact commands:

```sh
env -u CODEX_HOME -u CORBANU_HOME -u PFTERMINAL_HOME TMPDIR=/private/tmp PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=scripts/initiative_control:/Volumes/CorbanuDrive/Corbanu/.codex-work/initiative-control.oGQGyA/venv/lib/python3.14/site-packages /Volumes/CorbanuDrive/Corbanu/.codex-work/slack-sdk-test.Ob3i5O/venv/bin/python -B -m unittest discover -s scripts/initiative_control -p '*test*.py'
python3 docs/plans/check.py
python3 docs/sprints/check.py
git diff --check
```

Both governance checkers passed: plans **3/3 active, zero slots available**;
sprints **116 current, 126 archived**. Final `git diff --check` passed.
The four changed paths are within this allocation, and the prior evidence file
contents remain byte-for-byte intact. The commit hash is in the worker RETURN.
D09/D14's implementation-scope blockers are resolved; the functional acceptance
and recurrence gates above remain separate and open.

## Sprint registration repair — coordinator-sprint-registration-01

The [S02 archival divergence](s02-archival-divergence-20260915.md) exposed a
missing owner operation: a documented successor could not enter coordinator
state after initialization. Unknown-sprint dispatch correctly refused it.
This bounded reliability fix restores sequential sprint bookkeeping under
PF-80-S01 (in_progress), active plan **initiative-delivery-control**.
Product heading: **Internal delivery control — TO BUILD**; excerpt:
“durable event dispatch, acknowledgments and watchdog”.
It retains the existing owner, activation, pause and reservation boundaries.

Frozen assignment: Astra High, branch `bootstrap/owner-daemon-c-20260915`,
worktree `/Volumes/CorbanuDrive/Corbanu/worktrees/bootstrap-owner-daemon-c-20260915`,
base `55130a9b577529338b17e6a69adcb91a5cbc375c`.
Allocation digest:
`f98b606c6f5c56a2c640d6ca938cf38a15294bd94a381bf5763b771fe43b3fff`.
Brief SHA-256 verified before reading:
`5575f6dea21bc09fa8041053a079ffaa1787ba14d52e6613db2d5d92d78f6947`.

### API and refusal contract

Owner Python API (the CLI allowlist is outside this allocation):

```python
Coordinator.register_sprint(
    sprint_id, workstream, dependencies, status, source_path,
    expected_revision, evidence, repo, replace=False,
)
```

`source_path` is repository-relative and must match
`docs/sprints/**/*.md` with no absolute prefix, backslash, `..` segment or
empty component; it is read under `repo` and the resolved file must still lie
inside the resolved `repo`, which rejects a symlink pointing out of the tree.
`repo` must be a real checkout, proved by the presence of the `plan_file` the
document itself names, not by the caller asserting it. The **relative** path is
what the row stores, so a sprint row never pins one worktree.

`replace=True` re-registers a sprint that is still an unstarted registered
draft, which is how a row recorded with a bad `source_path` is corrected through
the audited API rather than by hand. It refuses if the sprint is archived, is no
longer `draft`, was never registered, already has any allocation or action, or if
the call would change the workstream, status or dependencies: the only thing a
re-registration may move is the document reference.

The operation uses `owner_mutation("owner_register_sprint", ...)`, atomically
adds one draft/unarchived row, increments revision, emits an event and audit,
and returns a retrievable evidence reference. Registration evidence pins the
repository-relative source path, exact document SHA-256, row, revision and
owner evidence. It neither changes the workstream's current sprint nor supplies
successor activation authority. Existing records, allocations and modes remain.

Repository sprint front matter identifies its workstream by `plan_file`;
the three existing mappings are security → p0-security-levels, accounting →
portfolio-agent-cost-accounting, delivery → initiative-delivery-control, under
`docs/plans/active/`. An optional explicit `workstream` must agree too.
The file read must exist and be regular, have bounded UTF-8 scalar front matter,
and match the supplied ID, workstream, status and ordered dependency list.
Missing `depends_on` is refused; `none` represents no dependencies.

Exact refusal messages include:

- `sprint already registered` for every existing ID when `replace` is false,
  including completed and archived rows; no overwrite, reopening, reparenting
  or unarchiving.
- `unknown sprint` when `replace` is true and the ID was never registered.
- `only an unstarted registered draft can be re-registered`,
  `re-registration may only correct the document reference`,
  `sprint already has allocations or actions`, `explicit add/replace required`.
- `sprint document id mismatch`, `sprint document workstream mismatch`,
  `sprint document status mismatch`, `sprint document dependencies mismatch`,
  and `sprint document dependencies missing`.
- `sprint source_path required`, `sprint source_path must be repository-relative
  under docs/sprints`, `sprint source_path escapes the repository`,
  `sprint source_path must be a regular file`, `sprint repo must be a directory`,
  `sprint document too large`, or `sprint source_path unreadable` for an absent
  or non-UTF-8 source.
- `sprint plan file missing from repository` when `repo` is not a checkout
  holding the plan the document names.
- `sprint front matter required`, `invalid sprint front matter`,
  `duplicate sprint front matter key`, or `invalid sprint scalar`.
- `invalid identifier`, `invalid sprint dependencies`,
  `unknown sprint workstream`, `unknown dependency`, `dependency cycle`.
- `registered sprint must start as draft`, `owner revision and evidence required`,
  `stale owner revision`, `dispatch/workstream paused`.
- Existing reservation refusals: `three-reservation limit`,
  `workstream already reserved`, `reservation differs from current sprint`.

Every tested refusal compares all six SQLite tables before/after. Registration
does not provide a manager action or a path around unknown-allocation refusal.

### Regression and mutation evidence

All 12 new `SprintRegistrationTests` passed. Each new test was exercised against
altered module source compiled only in memory, restored in `finally`, then
replayed with fresh synthetic state. Production source was never mutated on disk.
The tool transcript retains all attempts. Two harness assertions stopped at
nonunique source fragments (revision, then evidence); subsequent runs targeted
the owner block/first occurrence. Those interruptions are not counted as passes.

| Test suffix (`test_register_`) | Deliberate mutation | Broken result; restored |
| --- | --- | --- |
| existing_id_refused | Remove existing-ID refusal | 3 failures (draft, reserved, completed/archived); PASS |
| document_mismatch_refused | Separately remove ID, workstream, status, dependencies comparisons | 5/4/2/1 failures respectively; all PASS |
| missing_file_refused | Substitute asserted metadata for the document read | 1 failure, Rejected not raised; PASS |
| unknown_dependency_refused | Silently drop unknown dependencies | 1 failure, Rejected not raised; PASS |
| cycle_refused | Drop self-edge; separately allow revisiting a graph node | 1 failure each, Rejected not raised; both PASS |
| stale_revision_refused | Remove owner revision comparison | 1 failure, Rejected not raised; PASS |
| evidence_required | Remove owner evidence validation | 1 failure, Rejected not raised; PASS |
| paused_refused | Remove registration pause check | 1 failure, Rejected not raised; PASS |
| reserved_status_refused | Remove draft-only check | 1 failure, wrong reservation refusal; PASS |
| reservation_limit_refused | Remove three-reservation limit | 1 failure, wrong duplicate-stream refusal; PASS |
| malformed_front_matter_refused | Remove required-front guard; separately remove duplicate-key guard | 1 TypeError / 1 wrong-ID-refusal failure; both PASS |
| draft_is_durable_audited_and_does_not_activate | Replace owner_mutation with bare mutation | 1 missing-audit TypeError; PASS |

Total: **17 detected mutations, 17 restored passes**. Mismatch subcases after
an illicit insert can additionally fail on the now-existing ID; these are
retained cascade failures, not extra mutation experiments. The draft/reservation
and malformed-document mutations demonstrate exact guard discrimination; later
guards or parser errors still prevented registration in those mutants.

### Qualification and live registration

Final full suite: **695 tests passed in 438.625s**, no failures, errors or skips.
Synthetic HTTP 429/500 fixture cleanup ResourceWarnings were retained.
Synthetic transport artifacts: `/private/tmp/isolated-transport-tests-v1y16a94`.

Exact full-suite command:

```sh
env -u CODEX_HOME -u CORBANU_HOME -u PFTERMINAL_HOME TMPDIR=/private/tmp PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=scripts/initiative_control:/Volumes/CorbanuDrive/Corbanu/.codex-work/initiative-control.oGQGyA/venv/lib/python3.14/site-packages /Volumes/CorbanuDrive/Corbanu/.codex-work/slack-sdk-test.Ob3i5O/venv/bin/python -B -m unittest discover -s scripts/initiative_control -p '*test*.py'
python3 docs/plans/check.py
python3 docs/sprints/check.py
git diff --check
```

Governance passed: **3/3 active plans**, zero available slots;
**115 current, 127 archived sprints**. Diff whitespace check passed.

After the suite passed, the real owner API registered **PF-60-S03** from the
receiving checkout's document, whose bytes matched this checkout. Database:
`/Volumes/CorbanuDrive/Corbanu/.codex-work/initiative-control.oGQGyA/state/coordinator/coordinator.sqlite3`.
The running allocation digest and claim were checked before mutation.
Revision **1542 → 1543**, audit sequence **1770**, event sequence **747**,
event ID `owner_register_sprint:1542`.
Registration evidence digest:
`e8e622456186cfd3cf7b2edcf0f65c313f4907043bd0ae1378fa4650f31b8904`.
Document SHA-256:
`b275150d6a5ee027cfb25d7ddd3f46e48d5d5efbc501ad9379da10d760357dbd`.
Stored source (as first recorded, an absolute worktree path):
`/Volumes/CorbanuDrive/Corbanu/worktrees/management-workstreams-20260911/docs/sprints/current/portfolio-agent-cost-accounting/pf-60-s03-inspectable-run-and-campaign-totals.md`.
That path was the first P2 in review: it pins one worktree and rots when the
worktree is removed. It is repaired by re-registering PF-60-S03 with
`replace=True` against the relative path, which is why the replace lever exists.

Result: workstream **accounting**, status **draft**, archived **false**,
dependencies **[PF-60-S02]**. Accounting's current sprint remains **PF-60-S02**;
its mode remains enabled. Assertions verified every prior sprint, workstream,
allocation, action, manager and global enable value was preserved. No activation
occurred. The historical S02 source_path is still not repaired: `replace` is
confined to unstarted drafts, and S02 is completed and archived. Correcting a
finished sprint's recorded path is a separate decision, not a registration.
Python tests use disposable synthetic state, unset profile aliases,
`TMPDIR=/private/tmp` and no native credential access. No Rust tests are in scope.
True-TUI, live-repository and code-blind functional acceptance are not claimed:
this is an internal owner API repair, with no Terminal interaction change.
Reasoned internal-stage N/A is submitted for integrator disposition; the existing
independent confined execution/evidence and recurrence gates remain open.
No activation, service enablement, external messaging, release or push is authorized
by this repair. The brief separately authorizes registering PF-60-S03 only.
