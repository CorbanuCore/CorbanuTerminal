# RETURN — owner-isolated-112

Allocation digest: `0a16457e983e02c4b76fb36be728abb8c0e3ab0002a3796efdb3fb7dbd535b2b`.
Claim: `fe1b4144-4152-446d-9be3-f494b26a73a3`.
Runtime: `gpt-6-astra`, effort `high`.
Base verified: `7d22801b942eb8b487e6c7cbc3ff7c44e0c297f1`.
Brief SHA-256 verified before other reads:
`85875adaf6e42405824a2f7cdbeefc02bc5854cb78a43ed0dd30d06480d2fe99`.

This is routine analysis and test evidence, not implementation, qualification,
or an authorization record. Product context: **Internal delivery control — TO
BUILD**, “durable event dispatch, acknowledgments and watchdog” and “authorizes
the bootstrap and bounded live qualification, not premature product sprint
resumption.” No plan/sprint state is changed. No VM connection, live journal or
coordinator inspection, qualification attempt, credential access, commit or push
was performed. The VM facts below are supplied by the manager, not verified here.

## 1. Minimum meaningful isolated transport and inference profile

The claim to establish is narrow: the exact candidate owner/transport can launch
an actual worker at the authorized provider/model/effort/policy, observe its
correlated ACK, send START once, and observe RETURN, while the qualification
actor and its descendants cannot borrow production state or privileged access.
It does not prove the owner is secure against a malicious same-UID manager.
`owner_tmux.BridgeReceiver` explicitly trusts that manager, TMUX and rollout files.

The minimum consists of these boundaries, not a new general-purpose security
platform:

| Boundary | Minimum sufficient separation |
| --- | --- |
| Qualification versus production | One disposable guest/run with its own coordinator, schedule, journal, toy worktree, publisher destination, private profile, dedicated TMUX server/socket and evidence. No production queues, Slack/Task Node writes, host shares, host automation sockets, SSH-agent forwarding or routable production control endpoints. A disposable coordinator is appropriate for different fixture resources; never clone the live coordinator to drive the live worktrees. |
| Blind executor versus implementation knowledge | Fresh separate executor sees frozen cases, neutral navigation, fixture task data and read-only exact binary/assets. No Corbanu checkout/history, prior findings, source-readable owner runtime, unrestricted host tools or credential stores. A fixture Git repository is allowed task data. Running owner Python helpers in the guest does not make them acceptable executor input: keep trusted orchestration outside the actor's enforced accessible surface. |
| Executor/worker tools versus trusted services | Confinement applies to actual tool calls and children, including shell, process/IPC, package writes and cross-run sockets. The existing worker policy is `--yolo`; that flag supplies no isolation. Use a verified external OS/capability boundary and remove guest privilege escalation. A restricted outer executor is insufficient if the worker it drives can read the host or trusted controller through unrestricted tools. |
| Inference versus credentials/ambient profile | Fresh per-run HOME, CODEX_HOME, CORBANU_HOME and PFTERMINAL_HOME; no inherited personal configuration, sessions, aliases, Keychain or readable live auth clone. Permit only the explicitly authorized model route through a trusted inference broker with credentials outside actor reach. The route must preserve actual provider/model/effort identity and record it; no silent fallback. No arbitrary fetch, tools or source retrieval through the broker. |
| Network versus unrelated services | Default deny for the actor and children; permit only the required mediated inference and declared fixture services. Deny host/LAN control access and alternate source/credential channels. The trusted administration channel is separate and not exposed as an executor capability. |

One VM, one per-run fixture, one private TMUX socket and a narrow broker can be
sufficient. No second physical machine, new provider account, GUI login, full
security-workstream implementation or broad Internet access is intrinsically
required for a terminal-only lifecycle case. Existing authorized inference can
be brokered if its account/spend authority covers this test and its credentials
remain inaccessible. A VM alone is not sufficient when it contains exposed
source, credentials, privileged guest tools or host integration.

Prove those boundaries with negative probes from the actual fresh executor AND
its children: source/history/prior-results access, synthetic credential canary,
symlink escape, other-run IPC, unapproved network, package modification and child
escape. Record the effective policy and denied identity, not just a missing file
or an unexplained nonzero. Positive controls must show frozen-packet reading,
exact package launch, PTY input and authorized real inference in that same lane.
Use synthetic canaries rather than touching actual secrets to test denial.

Then execute the frozen applicable success/failure/cancel/recovery/resume cases;
retain raw attempts and explicit dispositions. At least one real ACK, START and
RETURN must correlate to action ID, claim, allocation digest, binary/package,
provider/model/effort, session/turn and completed model-response provenance.
Pane text, a synthetic worker, an echoed ACK or an unverified send-key receipt
is insufficient. Send prompt text and Enter separately. Independent evidence
review must follow; the implementer and executor cannot review their own proof.
The gate's three counts of at least one are a floor, not complete case coverage.

Without isolation, a real run may demonstrate a lifecycle in the operator's
ambient environment. It fails to establish that success is reproducible from
the declared profile, independent of prior sessions, implementation knowledge,
shared sockets, hidden credentials, permissive networking or manual interference.
It also fails to establish that the exercise cannot affect production. Those
are precisely the additional claims required here, not a claim that every
non-isolated observation is worthless.

Two implementation constraints matter before allocating execution:

- `owner_tmux` starts the binary locally and checks local UID/PIDs, boot identity,
  sockets and rollouts. SSH access to a VM does not turn the host transport into
  a remote transport. Run the matching trusted owner/TMUX control plane in the
  guest with an enforced actor boundary, or qualify a separately authorized
  remote transport change. Do not put `ssh` in the binary field and call it the
  exact candidate.
- `TmuxAdapter.prepare` writes a fresh minimal profile and symlinks `auth_link`;
  it does not install a credential broker or arbitrary inference profile.
  `isolated_model_transport.py` is a separately pinned text/image inference
  helper with tools disabled, not a drop-in qualification of this TMUX worker.
  Verify an actual compatible mediated route before claiming readiness. If that
  needs an adapter/configuration change, allocate, review, pin and test it first.
  A readable real test-account auth file does not satisfy the stronger executor
  contract merely because the account is called a test account.

Authority for the definition is the existing root policy and
[isolated execution contract](../../code-blind-functional/isolated-execution.md),
not new requirements invented by this return. The older round-94 auth-link
wording must be read subject to that contract's non-readable trusted broker rule.

## 2. The existing macOS VM

**Suitable substrate, not yet a qualified profile.** The supplied machine is
`192.168.64.3`, account `agent`, UID 503, macOS 26.2, launchctl manager name
`Background`, key-based SSH, with GitHub firewalled. This can support the narrow
terminal/PTY/TMUX and user-domain launchd lane. Session-less operation is not a
reason to require a GUI login for these claims; it cannot qualify GUI-session,
interactive personal Keychain or desktop permission workflows by implication.
The existing launchd probe is supporting evidence, not this candidate's proof.

Still unestablished, and therefore required before execution:

1. A clean per-run guest state and inventory; no host shares, forwarded agents,
   imported personal profile/Keychain, accessible prior probe findings, unrelated
   services or escape-capable sudo/admin access. UID 503 and SSH alone prove none
   of this. Enforce isolation on actor tools/children even if orchestration uses
   the same UID for the transport's private-file and process checks.
2. Stage the exact candidate binary and matching owner helpers/runtime, an
   appropriate Python with pinned dependencies, TMUX, frozen cases and disposable
   target data. Record binary and owner package hashes, runtime/TMUX/launcher
   identity, architecture compatibility and fixture paths. Helpers needed by the
   trusted controller must not become source accessible to the blind actor.
3. Enforce and probe network confinement. Blocking GitHub alone leaves other
   hosts, Git mirrors, LAN/host endpoints and direct inference routes unaccounted
   for. Keep management SSH under the manager's control without forwarding the
   manager's key or shell capability into the actor.
4. Provide the authorized compatible inference broker and fresh run profile.
   Install no personal auth clone; record provider/model/effort, allowed endpoint,
   spend authority and broker scope without credential values. Verify PTY and
   inference positive controls and all required negative probes.
5. Freeze cases; allocate a separate blind executor and independent reviewer;
   retain candidate-bound receipts and every blocked/failed attempt. This worker
   is neither an independent acceptance executor nor that reviewer.

If this guest cannot enforce those boundaries or expose the needed PTY/model
route, use a clean disposable macOS guest with disabled host integration and a
restricted execution/broker service. Another VM with the same missing controls
would not solve the problem. A Linux lane alone would not establish the claimed
macOS launchd behavior. No VM installation or account authorization is performed
or presumed here.

## 3. Item 4: manager versus Travis

**Manager/integrator:** inventory/provision the already permitted VM and fixture,
choose and verify confinement, stage exact assets, reuse an already authorized
inference route within its existing spend/model/policy scope, arrange independent
cases/execution/review, inspect receipts and record accurate evidence. The
isolated-execution contract explicitly assigns remaining infrastructure to the
manager. Routine review extensions can be recorded under the existing integrator
delegation. This need not become a third product decision.

**Travis:** any named limitation replacing required proof; accepting an excluded
case or narrowing the required qualification; changing product scope/gates or
supplying missing promotion/go-no-go authority. A proposed limited path must
name exact candidate, allowed actions, missing proof and stop condition. It
remains limited, not qualified. Missing account/credential/spend permission must
come from its actual authority holder; escalate to Travis only where his product
or spending authority is actually needed. Do not infer permission from possessing
an SSH key, a model tool, or a credential.

The paused security workstream stays paused. This bounded qualification analysis
is not permission to resume it or borrow its unaccepted implementation. If a
proposed broker or boundary requires resuming that work, stop at that dependency
and obtain its authority owner's decision; use an existing permitted boundary
where possible. The brief does not supply the text of the two open decisions,
so this return cannot claim either already authorizes a new account or exception.

## 4. Legitimate quiescence, including the recurring owner

**Continuous dispatch must stop for the window.** Stop the manager's new
allocation/decision/dispatch submissions and raw key sends at their caller,
finish current sends and manager cycles, and pause all competing publication and
coordinator-ingest/reconciliation writers. Existing worker computation need not
be killed; its lifecycle writers must also be quiescent, and selected allocation
reservations still must satisfy item 2. Do not suppress or fabricate outcomes to
make resources appear free. Queue pending observations outside the coordinator
and reconcile them honestly after the window.

This operational pause does NOT require `set_enabled(False)`. Leave the
coordinator enabled, the owner armed fixture-only, its receipt installed, service
present and pins unchanged. Those are explicit preflight predicates. Disabling
coordinator dispatch fails item 2 (`dispatch_not_enabled`); disarming fails item
5 (`armed_fixture_status_required`); bootout fails service presence. Changing the
plist/timer/runtime also invalidates pins until properly repinned. Pausing the
manager's calls without changing these records does not invalidate the audit.

The scheduled owner is itself both a possible coordinator writer and a publisher.
An admitted fixture tick can replay operations, emit a fixture event, and report
an overdue watchdog event; even a tick that leaves the coordinator revision
unchanged writes owner/tick health and publishes after `scheduled_tick` returns.
Consequently “the revision stayed equal” does not establish publisher quiescence.
The watchdog can still mutate when coordinator dispatch is paused.

**Current-code limitation:** no complete maintenance fence is implemented in the
inspected entrypoint. Holding `owner-daemon.lock` can yield BUSY while the schedule
still writes/publishes. Holding `tick.lock` is also insufficient: its nonblocking
failure reaches the outer handler, which still attempts `publish_schedule`.
Publication is outside that lock. A SQLite write lock is not a substitute and
can create failures. Do not SIGSTOP a tick halfway through a journal/publication
write, falsify the receipt, remove a pending artifact, or silently weaken a gate.
The local launchctl manual says `disable` prevents loading and `stop` can be
followed by an on-demand restart; neither supplies a verified loaded-job timer
fence for this procedure. No such OS behavior was tested here.

There are two honest operational possibilities:

1. **A measured gap between invocations.** The service may stay loaded/armed and
   scheduled if the entire audit and both preflight reads, through the recipe's
   controlled disarm boundary, fit into an actually quiet gap after the previous
   scheduled CLI has finished publication. Stop all the other writers first.
   Record the actual process/tick/publication observations and revision around
   the gap; abort/re-audit if any invocation/writer overlaps or an observation is
   unavailable. A 30-second timer does not promise 30 usable seconds after tick
   completion, and a stable coordinator revision alone is inadequate evidence.
   This is a conditional observed quiet interval, not an enforced reservation.
2. **A guaranteed window requires stopping scheduled execution too**, at an idle
   boundary, including its publication path. Keep the installed/armed state and
   service-presence predicates valid. The current commands do not provide a
   proven way to fence all those effects while preserving these predicates.
   Allocate a narrowly scoped, reviewed maintenance mechanism covering tick AND
   publication, or a properly specified audit/cutover procedure change, before
   promising a repeatable window. Qualify that change and refresh its candidate
   evidence. Do not just recommend early disarm/bootout against the unchanged gate.
   This is a technical coordination gap for the manager to resolve within actual
   authority, not permission to waive item 5 or resume paused security work.

For an audit alone, pause from the last writer's completed effect through the
final audit/preflight observations. For promotion, retain the pause through the
second preflight, controlled disarm/uninstall, reconfiguration, ownership
handoff, reinstall/arm/recovery and verified postconditions; then restore only
the correct writers under the new ownership map. The maintenance mechanism must
hand control to the authorized cutover, not deadlock its own disarm/install.

There is no justified fixed duration such as “one tick” or “30 seconds.” It is
**drain time + measured audit/preflight time**, plus **cutover and verification
time** when promoting. The audit timestamp must be at most **300 seconds** old
at EACH preflight and its coordinator revision must still match. That is a
freshness ceiling, not permission to dispatch during the window, not an expected
runtime, and not a maximum total maintenance duration. If too old, collect a new
real audit while still quiet; if a writer runs, invalidate the old observation.
An abort after effects requires the recipe's recorded recovery, not blind resume.

## Corrections and limits in the brief/prior advice

- `qualification_candidate_mismatch` only means the supplied qualification's
  binary hash or five-module owner `package_digest()` differs (or is absent).
  It occurs BEFORE reference/isolation checks. It does not diagnose a bad VM or
  identify which hash is wrong. Do not edit old evidence hashes to manufacture
  a pass: collect proof for the actual candidate, or locate already authentic
  evidence for exactly those bytes. Full helper/runtime identity also belongs
  in the evidence; that five-module digest does not cover every installed asset.
- Prior item-1/item-3 passes and round-111 review are supplied history. They were
  not rerun against live state here. Any report that this worker freshly passed
  items 1, 3, 4 or 5 would be false.
- The limited branch returns detail `LIMITED: ...`, but `evaluate()` records its
  item status as `PASS`. Thus a green JSON `ok` can mean an accepted limitation,
  not full qualification. Preserve and inspect the detail and actual acceptance.
- “An isolated test credential” readable by the executor is weaker than the
  canonical non-readable broker requirement. A fresh auth symlink is not an
  enforced credential boundary, and `env -i` is not an OS sandbox.
- No honest continuously mutating audit exists. The current recipe's assertion
  of quiescence is not itself a scheduler fence. A short observed gap is possible;
  a guaranteed repeatable maintenance window needs the missing coordination.

## Verification and changed lines

Read `docs/development/test-isolation.md` before testing. Built a new disposable
Python 3.14.4 venv under `env -i` from exactly
`scripts/initiative_control/requirements.txt`: markdown-it-py 3.0.0, mdurl 0.1.2,
slack-sdk 3.44.1. Setup exited 0; its log and disposable root are retained beside
this report. Both test runs started at the repository root with an empty inherited
environment and these explicit settings:

```text
HOME=CODEX_HOME=CORBANU_HOME=PFTERMINAL_HOME=/private/tmp/owner-isolated-112.l89YCO
CORBANU_TEST_NO_NATIVE_KEYRING=1
PATH=/opt/homebrew/bin:/usr/bin:/bin
TMPDIR=/private/tmp
PYTHONDONTWRITEBYTECODE=1
PYTHONPATH=scripts/initiative_control
```

Commands used the venv interpreter with `-B`:

```text
python -B -m unittest discover -v -s scripts/initiative_control -p 'test_*.py'
python -B -m unittest -v test_owner_daemon test_owner_tmux test_decision_feed test_attention test_control test_decision_manager test_slack_transport
```

- [Full discovery](owner-isolated-112-suite.txt): **880 tests passed**, 505.623s,
  exit **0**, zero failures/errors/skips.
- [Focused run](owner-isolated-112-focused.txt): **483 tests passed**, 386.401s,
  exit **0**, zero failures/errors/skips. Breakdown: owner daemon 133, owner TMUX
  52, feed 39, attention 24 (including **14 DecisionRenderingTests**), control 39,
  decision manager 113, Slack transport 83. These 483 overlap full discovery;
  there are 880 unique suite tests, not 1,363 distinct tests.
- Failure names: **none**. Raw logs preserve fixture refusal output and existing
  ResourceWarnings. No native credential prompt or live-profile access observed.
- Top-level verification nonzeros: **0**. Incidental inspection nonzeros: **1**,
  exit **2**, from `rg` against a nonexistent guessed filename
  `qa/initiative-control/management-bootstrap/test_owner_promotion_94_preflight.py`.
  This was an unsuccessful file lookup, not a test failure. Expected negative
  subprocess cases are asserted within the passing tests; this accounting is
  for top-level commands, not an invented census of their child exit codes.
- No Rust tests, formatter or fix tool ran. `git diff --check` passed. Final
  `git status --short` contains only this allocation's five new QA files; no
  existing or executable source file changed.

Changed lines are all additions: this report and four raw evidence artifacts
(`owner-isolated-112-suite.txt`, `owner-isolated-112-focused.txt`,
`owner-isolated-112-venv.txt`, `owner-isolated-112-test-root.txt`). Exact final
line totals are reported in the final return. Existing/source lines changed: **0**;
deletions: **0**. No commit or push.

TUI/code-blind execution, native qualification, live-repository tests, human
acceptance and benchmarks are not claimed for this routine analysis. Its
non-user-facing applicability is submitted to the integrator; the actual item-4
qualification remains the later independent functional gate. Live promotions,
real qualification worker launches/ACKs/STARTs/RETURNs, VM connections and new
human approvals in this assignment: **0 each**.
