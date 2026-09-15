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
