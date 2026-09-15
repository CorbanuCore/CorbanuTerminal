# Owner daemon live qualification — September 15, 2026

## Outcome

**Unattended startup is not qualified.** The unmodified daemon launched the pinned
real Corbanu worker, but its fresh profile stopped at an update dialog and then a
directory-trust prompt. No ACK or START occurred in that lane. A separate,
explicitly provisioned fixture completed the real model lifecycle, including
recording a durable RETURN after the worker was killed. The four requested safety
demonstrations passed within the limitations below. Recurrence remains OFF;
activation is a separate manager decision requiring notification to Travis.

This receipt is routine qualification/evidence work for the existing product
initiative, with no implementation changes. Product heading: **Internal delivery
control — TO BUILD**; excerpt: “durable event dispatch, acknowledgments and
watchdog” and “initialize and rehearse all three workstreams before enabling
recurring operation.” Plan: `docs/plans/active/initiative-delivery-control.md`
(active); sprint: PF-80-S01 (in_progress). Shared plan/sprint allocation
reconciliation remains manager-owned.

- Action: `daemon-qualify-01`.
- Allocation digest: `614227786fedc3f6ba33c1618f448e2bfc3b7281505f04adca186b22edc1921c`.
- Claim: `1c4ec300-7a55-497f-b9e6-a2ef4732d929`.
- Worker: OpenAI `gpt-6-astra`, High.
- Branch: `bootstrap/daemon-qualify-20260915`.
- Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/bootstrap-daemon-qualify-20260915`.
- Frozen and tested source commit: `a7682bc5c01e8196457932b19668fda6bd7bb7e4`.
- Brief: `/private/tmp/fmgr.Q1SIYZ/briefs/daemon-qualify-01.json`.
- Verified brief SHA-256: `544bfa8221a1973d8e2a2358a982f617a24554a15e6ff51cae446aac2334eb5c`.

## Environment and production-target exclusion

Artifact root, abbreviated **Q** below:
`/Volumes/CorbanuDrive/Corbanu/.codex-work/daemon-qualify-20260915`.
The executable harness is [qualify.py](/Volumes/CorbanuDrive/Corbanu/.codex-work/daemon-qualify-20260915/qualify.py).
[commands.jsonl](/Volumes/CorbanuDrive/Corbanu/.codex-work/daemon-qualify-20260915/commands.jsonl)
records harness invocations, subprocess commands/environment, observations,
profile provisioning, fault injection, cleanup and revisions with timestamps.

Four fresh Coordinators were initialized at `Q/{live,provisioned,expiry,uncertain}/state`.
Each has its own config, activation, three workstreams, three in-progress fixture
sprints, three frozen allocations, one implement action and two prepared wait
actions. They were not copied from production. All worktrees are freshly
initialized, empty scratch Git repositories under the corresponding fixture
folder; there is no product source/history in them. Coordinator seed revision is
4 after fixture initialization, enablement, event and validated decision.
The owner seed/decision is harness authority, not a claim of manager inference.

Every configured target is checked before setup, each tick and subsequent state
access: its resolved path must equal `Q/<case>/state`, lie under Q, and not lie
under `/Volumes/CorbanuDrive/Corbanu/.codex-work/initiative-control.oGQGyA/state`.
Every recorded check says `production_targeted:false`. No production Coordinator
was constructed, queried or mutated; no production activation record was opened
or written. The SDK command includes only the preexisting **venv site-packages**
path under the similarly named parent, not its state directory. The evidence is
an auditable configuration/call trace; it is not an OS-wide access audit or an
isolation claim against a malicious same-UID worker.

Activation schema requires `scope:"tmux-workers"` for this real transport, so the
fixture limitation is explicitly stated in `authority`, fixture-specific
`decision_id`, and digest-bound config pointing only at the new Coordinator.
These records confer no production authority. Setup first created OFF state;
only disposable owner rows were armed offline. No service/timer was installed,
enabled or scheduled. Cleanup set all four disposable owner modes back to OFF,
retaining activation records as historical fixture evidence.

## Exact package and profile contract

- Binary: `/Volumes/CorbanuDrive/Corbanu/.codex-work/provider-reauth-health/macos-candidate-final9-20260910/bin/corbanu`.
- Observed version: **0.1.41** in the actual TUI.
- Binary SHA-256: `4a8eba7b10199ea49aee42a720687b63cb4b1b2194c2ce164d1b9f6f88ee510e`.
- Daemon package digest: `0b88edce04aad1e7ee5de08f51367a947c106a14388909057ee2131f8d6f0e57`.
- TMUX: `/opt/homebrew/Cellar/tmux/3.7c/bin/tmux`.
- Runtime: `gpt-6-astra`, provider `openai`, effort `high`, recorded `--yolo`
  policy expressed by the transport as `--sandbox danger-full-access --ask-for-approval never`.

The daemon transport created private `Q/r/w-*/home` profiles, dedicated sockets,
160×48 PTYs and remain-on-exit sessions. HOME and all three profile aliases point
at the new private home; its restricted environment does not inherit inference
credentials. Following the explicitly assigned manager contract, the transport
created the native auth-file symlink without the harness opening, copying,
hashing or printing its contents. This is authorized linked-auth inference,
not credential-free inference or isolated synthetic-credential qualification.
No native credential prompt was observed. The worker used only the file-writing
tool on its allocated `proof.txt`; no credential inspection was requested.

**Startup mismatch:** unlike the manager's launch contract, the transport does
not suppress update checks or provision project trust. The `live` pane offered
an update from 0.1.41 to 0.1.43. The harness sent Down and then Enter, selecting
Skip, solely to inspect the next checkpoint. No update ran. The next pane asked
whether to trust the scratch directory. No trust-prompt approval was sent.
Ticks remained `awaiting_ready`.

The separate `provisioned` lane wraps only `TmuxAdapter.prepare`: it first calls
the real implementation, then writes fresh-profile config before launch:

```toml
check_for_update_on_startup = false
[tui]
animations = false
[analytics]
enabled = false
[projects."/Volumes/CorbanuDrive/Corbanu/.codex-work/daemon-qualify-20260915/provisioned/scratch"]
trust_level = "trusted"
```

This trust applies to the harness-created empty scratch repository. It is a
fixture setup amendment, not an implementation fix. No real worker, model
response, rollout, ACK, RETURN or daemon receipt was mocked. All later ticks use
the unmodified adapter. This lane establishes lifecycle behavior after explicit
profile provisioning; it cannot qualify unattended default startup.
The exact transport also omits the test-tui skill's trace/log-dir launch settings;
raw panes, rollouts and daemon receipts are retained, but no added trace setting
or patched launcher is claimed.

## Per-tick results

All completed ticks include the exact output in `Q/<case>/tick-<n>.json` and a
full Coordinator snapshot in `snapshot-<n>.json`. Each output contains
`routing:"tmux-workers", fixture_only:false`; that flag identifies live transport,
not production activation. Every completed tick retained both prepared wait
actions byte-for-byte (`passive_unchanged:true`).

| Case / tick | Revision before → after | State | Action outcome |
| --- | --- | --- | --- |
| live / 1 | 4 → 5 | ACTIVE | live-delivery: awaiting_ready |
| live / 2 | 5 → 5 | ACTIVE | live-delivery: awaiting_ready |
| live / 3 | 5 → 5 | ACTIVE | live-delivery: awaiting_ready |
| provisioned / 1 | 4 → 5 | ACTIVE | provisioned-delivery: awaiting_ready |
| provisioned / 2 | 5 → 5 | ACTIVE | provisioned-delivery: awaiting_ack |
| provisioned / 3 | 5 → 7 | ACTIVE | provisioned-delivery: working |
| provisioned / 4 | 7 → 8 | ACTIVE | provisioned-delivery: returned |
| provisioned / 5 | 8 → 8 | ACTIVE | provisioned-delivery: returned |
| provisioned / 6 | 8 → 8 | ACTIVE | provisioned-delivery: returned |
| expiry / 1 | 4 → 5 | ACTIVE | expiry-delivery: awaiting_ready |
| expiry / 2 | 5 → 6 | HOLD | expiry-delivery: HOLD |
| expiry / 3 | 6 → 6 | HOLD | expiry-delivery: HOLD |
| uncertain / injected launch | 4 → 5 | process exit 75 | No completed tick output; real launch preceded exit |
| uncertain / recovery 1 | 5 → 5 | HOLD | uncertain-delivery: HOLD |
| uncertain / recovery 2 | 5 → 5 | HOLD | uncertain-delivery: HOLD |

The required completion/idempotence sequence is provisioned ticks **4, 5, 6**.
The second and third ticks in that sequence make no additional worker or
Coordinator mutation. Tick boot/health bookkeeping still occurs; they are not
byte-for-byte no-ops on the owner SQLite database. Startup and inference require
polling across ticks; no single immediate tick is claimed to await all inference.

## Exact real-worker ACK, START and RETURN

Worker run: `Q/r/w-3l01ggne`; PID **6960**; TMUX server **6959**.
Claim: `56a24e38-a14e-49ec-b1e9-50d7666f34f4`.
Session/thread: `01a0a426-7bf7-77a1-b659-0e8f82f8ff71`.
START work turn: `01a0a427-5e78-77d0-b061-4274b735bd27`.

Exact first completed assistant message:

```text
ACK provisioned-delivery 04783cadbd5904ef263f616a26760a37e4b0b60aab29c40e8aebe0f697904e23 gpt-6-astra high
```

The real transport compared this completed assistant message to the frozen ACK.
The harness independently asserted UTF-8 byte equality of the whole line and of
the digest token against the Coordinator action and recomputed allocation digest.
See [verification.json](/Volumes/CorbanuDrive/Corbanu/.codex-work/daemon-qualify-20260915/verification.json).

`prompt-intent.json`, `prompt-keys.json`, `start-intent.json` and `start-keys.json`
record the actual delivery. The transport sent bracketed-paste text and Enter in
separate TMUX calls. It sent **START** once. The correlated rollout user message
and matching runtime turn prove submission; the 08:20:39Z pane visibly shows
`START` and `Working (0s • esc to interrupt)`. Key delivery receipts alone retain
`accepted:false` and are not counted as proof of working.

Exact second completed assistant message:

````text
RETURN
/Volumes/CorbanuDrive/Corbanu/.codex-work/daemon-qualify-20260915/provisioned/scratch/proof.txt
```text
daemon qualification proof
```
````

The scratch file was independently checked as exactly
`b'daemon qualification proof\n'`. Completed-rollout SHA-256:
`e032aa9dcfc6070922d39e1b77fe1f8a117abd08a7057c9acac1dc78647b7356`.
Raw rollout:
`Q/r/w-3l01ggne/home/sessions/2026/09/15/rollout-2026-09-15T01-19-40-01a0a426-7bf7-77a1-b659-0e8f82f8ff71.jsonl`.
Selected real session, runtime, submission, response and completion events are
preserved in `Q/provisioned/correlated-rollout-events.json`.

The owner DB contains all eleven effects in `applied` phase: claim, prepare,
launch, prompt, ack, dispatched, acknowledged, start, working, return_observed,
returned. It contains no hold for this action. Coordinator status is `returned`,
not verified or accepted. Full tables and process provenance are retained in
`Q/provisioned/summary.json`.

## Four live safety demonstrations

> **Concurrency disclosure (independent review, September 15).** The 301-second
> SDK suite ran from 08:18:14Z to 08:23:15Z, overlapping every live tick
> (08:18:15Z–08:21:40Z) including the real, unmocked five-second lease deadline.
> The suite used a separate `Q/suite-home` and PYTHONPATH and nothing observed is
> invalidated — the expiry case was expected to expire — but a receipt that leans
> on a real 5 s deadline should record that the host was simultaneously running a
> full test suite. Future timing-sensitive lanes should run unloaded.

1. **Prepared non-worker actions remain untouched — PASS.** Every fixture has
   security/accounting wait actions. Across healthy ACTIVE ticks, including the
   completed lifecycle, their full Coordinator records remain identical and
   prepared. No worker run or operation is created for either. This live case
   covers `wait`; coverage of the other excluded kinds remains SDK evidence.
2. **Expired lease with surviving PID — PASS.** The expiry allocation has a real
   five-second deadline, with no clock mocking. PID **7161** remained alive with
   its original process identity. Tick 2 recorded `terminal_status:lease_expired`
   and a `lease_expired` hold; watchdog moved revision 5→6. Tick 3 remained HOLD
   at revision 6, with one launch/run. Transport liveness correctly still says
   `alive`; the expired lease is dead for dispatch purposes and never renewed.
   See `Q/expiry/summary.json` for the simultaneous alive observation and expired
   owner process record. No ACK or inference was needed for this timeout case.

   **Scope of this demonstration (independent review, September 15).** This lane
   used the unmodified adapter, so its worker was still sitting at the same
   update/trust startup checkpoint documented for the `live` lane: the summary
   records `ready:false` and the same generic `pane_digest` (`56442b5d…`) as the
   never-started `uncertain` worker, with no session, thread or ack fields. What
   is therefore proven is **"the lease deadline elapsed while the worker was
   never ready"**, not "the lease expired on a worker that was actually running a
   turn". The daemon behaviour observed is real and is the behaviour we want, but
   expiry against a genuinely mid-turn worker is **not yet demonstrated** and is
   added to the startup-fix follow-up below.
3. **Kill after durable RETURN — PASS in provisioned lane.** At revision 7 the
   Coordinator still said `running`, while the real completed rollout already
   contained RETURN. At 08:21:27Z the harness sent SIGKILL to the identified PID
   **6960**. The next observation showed `pane_dead:1`, `liveness:crashed`, no
   survivors and the same correlated RETURN. Tick 4 recorded returned at revision
   8; ticks 5/6 preserved it. `before-kill.json`, `after-kill.json`, `tick-4.json`
   and the correlated rollout retain the ordering. No post-death keys or new
   process were used to recover the result.
4. **Uncertain effect holds rather than retrying — PASS.** A dedicated harness
   mode calls the real `Worker.launch`, records actual PID **7201** and then
   exits the owner process with code 75 before the daemon's launch receipt can
   be written. Transport intent/client/process artifacts survive; the daemon
   operation has an unreceipted intent. Recovery tick 1 records `effect_uncertain`
   and HOLD at revision 5; recovery tick 2 retains HOLD and the same revision.
   There is exactly one worker run, no prompt/START receipt and no relaunch.
   The empty owner process table is expected: the crash preceded the daemon
   observation insert. Actual TMUX/process evidence lives under
   `Q/r/w-3hl3tty1`; see `Q/uncertain/summary.json` and the fault log.

## Commands, artifacts and cleanup

Preflight first ran `shasum -a 256` and read the named brief, then read
`owner-daemon.md`, `owner-tmux.md`, `owner_daemon.py`, `owner_tmux.py`, relevant
Coordinator/test helpers, root policy, product heading, active plan/sprint,
`docs/development/test-isolation.md`, and the development/TUI skills. Read-only
`git status --short`, `git rev-parse HEAD`, and `git branch --show-current`
confirmed a clean frozen checkout and the assigned branch. Manager launch
contract was read from `/private/tmp/fmgr.Q1SIYZ/dispatch.py`; it was never run.
No production-state inspection was used to obtain its configuration.

Q was created mode 0700, followed by `Q/r` and `Q/suite-home`. The disposable
harness was created and amended only under Q; this receipt is the only repository
file changed. It records commands and exact Python API operations in its source.
Every invocation below used `PYTHONDONTWRITEBYTECODE=1 python3 -B Q/qualify.py`:

```text
init live
suite                         # separate guarded-environment Python SDK process
checks
 tick live; inspect live      # notation lists sequential harness invocations
# Direct fixture TMUX navigation: send-keys Down, then send-keys Enter (Skip).
inspect live; tick live; tick live
init provisioned; tick-provisioned provisioned
inspect provisioned; tick provisioned
init expiry 5; tick expiry
init uncertain; tick-crash uncertain  # actual process exit 75, retained
 tick provisioned; inspect provisioned; tick expiry; tick uncertain
 tick expiry; tick uncertain; inspect provisioned
kill-return provisioned
 tick provisioned; tick provisioned; tick provisioned
summary expiry; summary uncertain; summary provisioned
verify
summary live
cleanup provisioned; cleanup live; cleanup expiry; cleanup uncertain
```

The chronological, timestamped command journal resolves this compact inventory;
semicolon notation above is descriptive, not a claim that all calls shared one
shell command. Raw outputs were additionally checked using `tail` on the SDK log.
Every tick has a target guard and before/after revision. Inspection saves separate
`inspect-<time>.json` and `pane-<time>.txt` attempts, including startup failures.
Snapshots, summaries, eleven operation request/receipt chains, transport manifests,
rollout and original failed/crashed attempts remain under Q.

Cleanup qualified all four fixture sessions as `clean:true`: the RETURN worker
was already killed by its test, and the three startup-blocked fixture PIDs were
sent SIGTERM without typing into any update/trust prompt. Transport close then
confirmed dead pane, no observed surviving children, server exit and failed
session probe. No global TMUX kill was used. Auth symlinks were unlinked without
following them, and fixture owner rows set OFF. See each `cleanup.json` and
`commands.jsonl`. This is observed-process cleanup, not hostile-child containment.

## Automated validation

**591 tests passed in 301.254 seconds**, exit 0 (301.651 seconds wall time).
Only the previously documented HTTP 500/429 fixture-cleanup ResourceWarnings
appeared. No failure, retry or native credential prompt was observed. Raw output:
[sdk-suite.log](/Volumes/CorbanuDrive/Corbanu/.codex-work/daemon-qualify-20260915/sdk-suite.log).
The log's synthetic publication messages belong to tests; they do not indicate a
production publication or activation by this qualification harness.

The full suite uses the established SDK interpreter, `-B -m unittest discover
-s scripts/initiative_control -p '*test*.py'`, from the assigned checkout, with
`TMPDIR=/private/tmp`. Exact argv/environment/output location are in the command
journal. HOME and all profile aliases point to `Q/suite-home`; the constructed
environment excludes inherited credentials. No Rust tests were needed or run.

Governance checks passed: **3/3 active plans**, **116 current / 126 archived
sprints**. `git diff --check` passed after receipt authoring. Source and test files
remained identical to the frozen commit throughout validation; only this receipt
is committed. There was no code formatting/fix pass or Rust test campaign.

## What still blocks recurrence

- Fix and review the fresh-profile startup contract, then repeat an unassisted
  exact-package live run. Current default launch encounters update/trust prompts;
  harness profile provisioning is not a shipped fix or an unattended pass.
- Manager review/receiving of this receipt and allocation reconciliation, plus
  disposition of the provisioned-lane limitation. No approval is inferred.
- The existing combined functional handoff still requires fresh code-blind cases,
  independently confined execution and evidence review, including enforced
  filesystem/process/IPC/network/credential boundaries and negative probes.
  This code-aware implementation-worker run is supporting engineering evidence,
  not independent acceptance. Any internal-stage N/A requires integrator acceptance.
- Outstanding manager/receiving/verification, Slack decision/ACK, packaged
  supervision and broader recovery evidence remain governed by PF-80-S01 and
  the owner-daemon/owner-tmux records; this bounded worker-only run does not close
  them. No TensorCash/Isometric, benchmark, release or named-human acceptance
  evidence was produced here.
- Production activation requires the manager's separate explicit decision and
  notifying Travis. This assignment authorizes neither that activation nor
  launchd installation. No recurrence was activated, no Slack message sent and
  no branch/release pushed.
