# PF-80-S01 bounded owner manager cycle

Original frozen, uncommitted worker handoff, September 13, 2026. Product initiative under
**Internal delivery control — TO BUILD**: “Use sequential sprints per initiative”
and the September 13 authorization for “fresh Fable 5.1 High management through
Corbanu/TMUX; durable event dispatch”. PF-80-S01 remains `in_progress` in the active
`docs/plans/active/initiative-delivery-control.md` plan; this unit does not close it.

Worktree `/Volumes/CorbanuDrive/Corbanu/worktrees/manager-cycle-20260913`, branch
`bootstrap/manager-cycle-20260913`, base `bca6485a2e60803393bbca0ee56d98953022ed12`.
The parent's canonical allocation is the **Event-to-manager driver allocation**
section of `docs/research/tasknode-integration/coordinator-bootstrap-20260913.md`
in `/Volumes/CorbanuDrive/Corbanu/worktrees/management-workstreams-20260911`.
Its active plan records these worker coordinates; both parent checkers passed.
Local plan/sprint copies remain at the base; shared records belong to the parent.
Only `manager_cycle.py`, `test_manager_cycle.py` and this receipt are added.
Frozen size: 799 total lines / 373 non-test (279 implementation, 426 tests, 94 QA).
Implementation SHA-256: `f1ed9c4f96ddd7d02cf8269a2f578423afc297af9ca682b9aa04fbeda4dd661a`.
Test SHA-256: `d78696bdc564ba0f71a9318e00a1cf5bb82e5160597c80ff1e943aea4ea28d61`.

## Contract and operation

`scripts/initiative_control/manager_cycle.py` exposes `run_cycle` and an explicit
`--run` CLI. Without `--run`, it returns OFF without inspecting paths. Help and
import do not open state, credentials, processes or network. Run requires explicit
`--state`, `--runs-dir`, `--binary`, `--auth-file`, `--owner-context`; timeout defaults
to 300 seconds. State must already be initialized and private. The small existing-
store adapter reuses Coordinator operations without its creating constructor;
readiness uses read-only SQLite and subsequent connections use `mode=rw`.
Paused, empty and already-owned stores are unchanged and do not read other inputs.

Owner context is a private, non-symlink, single-link JSON file capped at 8192 bytes:
`{"observed_at":"2026-09-13T00:00:00Z","context":{"authority":"owner supplied"}}`.
Supply real dated observations, approvals, unresolved blockers and review budgets
privately. This example supplies no production authority. Stored seed metadata is
explicitly historical; durable modes/allocations are not fresh external proof.

One meaningful pending batch is claimed through `begin_manager` (at most 24 events).
The briefing includes all three streams, ordered last-three action IDs, frozen
unconsumed scopes, pending actions, and recursively loaded original evidence keyed
by verified digest. Action records retain identity, kind, workstream, sprint, status,
rationale, allocation identity/digest, other lifecycle metadata and reference digests.
Terminal actions (`accepted`, `failed`, `cancelled`) omit inline inputs and expansion
of dispatch/ACK/result/verification/owner-failure/owner-cancellation references,
except when both in the last three and represented by a status transition in the
selected pending event batch. This exception supplies their exact inputs (or the
existing lossless allocation index) and recursively verified originals. Compacted
actions retain the core-stored result, verification, owner_failure and
owner_cancellation previews, each at most 400 characters, so later managers can
read partial outcome context after the transition batch is consumed. These previews
are untrusted partial context and do not replace the omitted full originals.
Allocations with literal `inputs.consumed == true` retain metadata and only
`{"consumed":true}` as inputs; active actions never index into compact allocation
inputs. Other allocation scopes and non-terminal action inputs remain exact.

Selected event originals and everything they reference always expand in full,
including evidence also referenced by compact history. Status transitions are read
from selected top-level event originals: returned/verified action IDs, exact
reconciled-dispatch IDs, owner completion/successor action IDs, owner allocation
cancellations and explicit matching action/status records. Nested evidence, unrelated
mentions and deferred events do not establish this exception. FIFO batch sizing
recomputes eligibility for each selected prefix before restricting the durable claim.

`evidence_omissions` identifies each affected action/allocation by `source`, `id` and
`reason`; `inputs_digest` hashes the exact omitted inputs and `evidence_digests`
lists reference roots whose originals were not expanded elsewhere. Unloaded roots
also identify their unexpanded descendants; they are not fetched just to enumerate
omissions. This is unavailable context, never a success claim. The untouched core
packet remains in `claim.json`. Derived previews are removed only at known reference
positions, except for the four compacted outcome fields above. Compacted action
references do not load originals; a shared reference still expands when required by
a selected event or another retained source. No original is shortened or summarized.
Missing/corrupt retained evidence,
excessive retained reference count, or a briefing exceeding the unchanged 65536
encoded bytes produces an owner hold before authentication/inference, with pending
events and the claim retained.
Rationales are strongly directed below 300 UTF-8 bytes; the unchanged core still
accepts at most 1000 bytes. Frozen inputs are never rewritten to fit acceptance.

A fresh private `m-<manager-token>/launches` directory binds exactly one synchronous
`fable_launcher.run_launcher` invocation. Use a short private runs root outside Git:
the reviewed launcher enforces its TMUX socket-path limit. Credentials are supplied
only by its explicit private auth-file interface. No credential adapter is added.
The injected callable is a trusted test/owner adapter, never model/event input.

Validation reads actual `receipt.json`, `manifest.json`, `launch.json`, copied
launcher/shell/packet, `candidate.json`, full `final.txt`, session rollout,
`process.json` and `stop`, using reviewed launcher parsing/evidence helpers.
It binds token, revision, exact briefing bytes, binary path/hash/argv, launcher hash,
fresh run/timestamps, Fable model/provider/high, session/thread/turn/response and
complete final decision. It requires the reviewed launcher's durable clean shutdown
record, session disappearance and owned-process binding. A caller boolean, PID guess
or elapsed deadline is not an alternate shutdown proof. No process is inspected or
killed by this driver. Existing Coordinator validates proposals transactionally.

Success consumes only selected events and returns durable `prepared_actions` for
the real host's claim/native-tool/ACK workflow. It performs no dispatch, integration,
Slack sends, scheduling, retry, shutdown reconciliation or initialization. Attempts
retain claim/brief/attempt/returned/validated/hold records as far as execution reached.
After an uncertain failure the owner inspects actual shutdown and core state before
using existing reconciliation. Never blindly call `fail_manager` on a timeout.
After an acceptance commit but before output, SQLite actions/audit are authoritative;
a missing `accepted.json` is not permission to repeat inference or dispatch.

## Verification and receiving handoff

Initial driver suite: 16 passed in 0.974s. Expanded system-Python suite: 80 ran,
79 passed, one existing collector test errored with `ModuleNotFoundError: slack_sdk`.
No code defect or live call was involved. Retained failure; reran using the existing
documented SDK interpreter: 80 passed in 1.934s; final frozen code: **80 passed in 1.893s**.
Counts: 21 driver, 37 Coordinator/owner-control, 7 launcher protocol, 15 collector.
Driver coverage includes 20 artifact/identity rejection subcases, recursive originals,
OFF/paused/empty/owned, stale/paused/deadline attempts, CLI, and five real child-process
exits at claim persistence, launcher entry, returned receipt, validated proof and
acceptance commit. Fixtures use real SQLite and actual artifact schemas, with an
absent credential file and no inference. Process identities are labeled synthetic.

Reproduce from this worktree with `PYTHONDONTWRITEBYTECODE=1`,
`PYTHONPATH=scripts/initiative_control` and the documented interpreter
`/Volumes/CorbanuDrive/Corbanu/.codex-work/slack-sdk-test.Ob3i5O/venv/bin/python -B -m unittest test_manager_cycle test_coordinator test_fable_launcher.Protocol test_slack_reply_poll`.
Both local governance checkers and whitespace checks pass. Existing dependencies
remain untouched. Parent owns Fable material review, receiving checks and actual
fresh-manager/native-host replay. No other agents or reviews were invoked here.
Internal-only engineering N/A follows the canonical allocation; this is not true-TUI,
TensorCash/Isometric Game, human, benchmark, release or isolated functional acceptance.
Those later applicable gates remain with the parent; recurrence stays unqualified.

## Scoped correction after original Fable review01

The original 799/373 receipt, hashes, test attempts and findings above remain
historical. Parent accepted P2 shared timeout as blocking and P3 as a narrow
read-only diagnostic correction. Original review is untouched at
`/Volumes/CorbanuDrive/Corbanu/.codex-work/bootstrap-cycle-review.WVYhMZ/review01.json`,
SHA-256 `b5a9c967e3ebabbd150964cf2203353f17285e063310cd872d16e6aafe0ffb1f`.
Parent owns material review02 under its corrective allowance, receiving tests and
actual fresh-manager/native-host replay; this worker invoked no agents or reviews.

`--timeout` now means total core claim seconds from claim creation, default300,
finite numeric range31..3600. At launch, subtract actual elapsed preparation and
a fixed30-second reserve for launcher pre-timer setup/version probe, shutdown and
validation. The launcher receives at most3570 seconds. Invalid values hold before
claiming; fewer than one remaining launcher second holds before inference and
retains the claim. A31-second window therefore needs zero preparation time to
launch; choose a larger budget for real operation. No rounding up or deadline
extension occurs. This is bounded margin, not a hard process-runtime guarantee:
overhead beyond the reserve still encounters the core's authoritative deadline
and retains ownership for reconciliation. Readiness short-circuits remain first.

Readiness remains `mode=ro`, including paused/empty/owned stores. SQLite error
code776 (`SQLITE_READONLY_ROLLBACK`) maps only to `sqlite_recovery_required`;
all other SQLite errors map to `sqlite_unavailable`. No raw exception, SQL, path
or arbitrary error name is exposed. Preflight holds have no attempt directory;
the returned fixed diagnostic is the evidence. OFF still performs no state read.

Owner recovery: first reconcile actual launcher/process state and verify the exact
existing private database path. Then deliberately open that existing store using
`c = Coordinator(Path("<verified-existing-state-directory>")); state = c.snapshot()`
in the supported owner Python environment. This existing rw API permits SQLite's
journal rollback; it is not automatic driver readiness or a new initializer.
Do not delete the journal or call `initialize`. Recheck readiness and inspect the
preserved manager/events; clearing an owned manager via existing `fail_manager`
still requires actual shutdown reconciliation, never just elapsed time. For
`sqlite_unavailable`, the owner checks locks/permissions/database health first.

Six new tests cover real core-clock late completion with12s preparation,5s launcher
setup,8s shutdown and4s validation; minimum/fractional/maximum/invalid budgets;
preparation exhaustion; real core deadline overrun; and fixed SQLite diagnostics.
The child-crash regression forces a synced hot DELETE journal via dirty-page spill
for paused/empty/owned/ready stores. Repeated probes plus OFF preserve database and
journal bytes; after the child is reaped, explicit owner Coordinator rw recovery
restores the exact pre-crash SQL dump and readiness. No live credentials/inference.

Correction verification: driver27 passed in2.422s; full86 passed in2.895s;
after adding the oversized-integer edge case, final code **86 passed in2.757s**
using the documented SDK venv command above (27 driver + original59 dependencies).
Frozen correction: **992 total / 439 non-test** (291 implementation,553 tests,148 QA),
above the800 total target, within450 non-test target and1050/600 hard ceilings.
Implementation SHA-256: `3e95db1d59c13173e31610c0914f1d1b3786b97f270c7763dfb2471c8890e411`.
Test SHA-256: `bc95c42ccc163d5bab5b297452534cda92e9dcc483b8746f6e60a5355403581f`.
Governance and whitespace checks pass; exactly three staged additions, no commit/push.

## Parent review, receiving and actual recovery rehearsal

Review02: Fable5.1High, helper exit0/no findings; original review01 retained.
Parent reproduced86 focused passes in2.663s, accepted the992/439 scope, committed
`b108c4ac81c10da2548878893e558b570754cdc9`, then received it with the actual
single-writer job at `3b25e9d9e4684df10cea505a2022db20eec04db4`.
All124 receiving tests passed73.157s, plus both governance checks and whitespace.
The durable receiving receipt is `.codex-work/bootstrap-cycle-review.WVYhMZ/receiving/receive-manager-cycle-b108c4ac8.json`,
SHA-256 `05f8f2807d455cc913814922d219a03415c85917623b426152ddcddbac003085`.

Parent ran the accepted driver against the existing real coordinator, not a fixture.
Private `/private/tmp/mc.CVU7SR/verification.json` records two complete fresh runs:
`f-h7ano0vl`, session `01a099cf-34d5-71d2-a7de-be41366923df`,34857-byte briefing;
`f-pbou3z1v`, session `01a099d0-b33e-77b1-aca9-de7888155190`,44795-byte briefing.
Both loaded `claude-fable-5-1-plan`/`claude-plan`/`high`, passed artifact validation
and core acceptance. Independent owner TMUX/ps probes confirmed both sessions
and their owned processes absent. Run homes contain credentials; never export them.

The first manager allocated a real zero-tool Astra High ACK agent, Godel
`01a099cf-b9e8-7413-9f7d-7d541daef60f`. Parent deliberately omitted the durable
dispatch recording after actual spawn, reproducing that crash window rather than
claiming the host coordinator itself crashed. After the real15-second deadline,
a fresh process recorded exactly one `dispatch_uncertain` watchdog event; another
watchdog changed nothing and a duplicate claim was rejected even when enabled.
Actual host spawn/ACK receipts reconciled the original identity and acknowledgment.
Actual close returned its completed status; the subsequent tool lookup returned
`not_found`, confirming removal (not the initially expected `shutdown` spelling).
The owner then recorded intentional failure/stop, never fabricated work completion.
The second fresh manager accepted a wait-only follow-up from that durable evidence.

At08:10:42Z, coordinator revision53: global and all three dispatch modes paused,
no manager claim; rehearsal action failed and follow-up wait prepared. No product
sprint advanced. This proves operator-invoked driver/native crash-window recovery,
not a deployed unattended controller, full Slack chain or isolated acceptance.

## September 14 briefing-size repair: scoped worker return

Action `briefing-size-repair-01`, bounded internal reliability fix under
**Internal delivery control — TO BUILD**: “Use sequential sprints per initiative”.
Assigned worktree `/Volumes/CorbanuDrive/Corbanu/worktrees/bootstrap-briefing-size-20260914`,
branch `bootstrap/briefing-size-20260914`, base
`c7ee093bc6a35c9fdd3f63d8f6bf49495cf61087`. The contract above describes the
implemented selection rules. Coordinator state, original evidence, claim limits and
authentication/inference boundaries are unchanged.

Synthetic measurement uses `BriefingSizeTests.large_packet` in
`scripts/initiative_control/manager_cycle_test.py`: one pending event, three
terminal actions with distinct dispatch/ACK/result/verification originals and one
consumed allocation with a referenced original. The base briefing holds at 65536;
its complete encoding measures **245123 bytes**, versus **7197 bytes** after this
repair. To count the rejected base output, only the base module's encoder ceiling
was bypassed in memory; production `BRIEF_LIMIT` remains **65536**. No original was
truncated. The new tests also cover terminal statuses, both recent-transition
conditions, nested/shared originals, consumed allocation input safety, omission
digests, missing/corrupt/size-mismatched retained evidence and FIFO batch restriction.

Verification commands and actual outcomes:

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s scripts/initiative_control -p '*_test.py'`:
  **12 passed** (new regression module).
- `PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=scripts/initiative_control /Volumes/CorbanuDrive/Corbanu/.codex-work/slack-sdk-test.Ob3i5O/venv/bin/python -B -m unittest test_manager_cycle test_coordinator test_fable_launcher.Protocol test_slack_reply_poll manager_cycle_test`:
  **120 ran; 117 passed, one failure, two errors**. The same existing focused
  modules using the base implementation loaded in memory pass **108/108**.
- Initial system-Python README discovery (`test_*.py`): **301 ran; six failures,
  twelve errors**, including missing `markdown_it`/`slack_sdk`, a subprocess
  `PYTHONPATH` failure and four launcher-environment failures. These are retained
  as failed evidence, not a passing full-suite claim.
- `PYTHONDONTWRITEBYTECODE=1 python3 docs/sprints/check.py` and
  `git diff --check`: passed.

**Initial scope blocker (resolved by the manager extension below):** the existing suite is actually
`scripts/initiative_control/test_manager_cycle.py`, outside this allocation's
explicit writable paths. Three tests still require terminal inputs inline:
`test_frozen_inputs_that_resemble_references_are_not_rewritten`,
`test_repeated_action_inputs_are_losslessly_indexed` and
`test_historical_or_nonidentical_action_inputs_stay_inline`. Their fixtures need
to use non-terminal actions to retain their original lossless-input coverage under
the new contract. That file is untouched; the existing suite is not green and this
return is not an acceptance claim. The manager must authorize the corrected test
path before those three fixture adjustments and a final rerun.

This is internal packet serialization, with no interactive product change. The
prior internal-only N/A scope applies; parent still owns actual fresh-manager
replay, evidence review and any later applicable functional/release qualification.
No live coordinator, credentials or inference were used, and no push was performed.

### Manager-authorized fixture adaptation

The manager extended writable scope to `scripts/initiative_control/test_manager_cycle.py`
for exactly the three conflicting fixtures above. Terminal cases now assert exact
`evidence_omissions` input/reference digests and unchanged claim inputs; running
cases retain the prior complete inline/indexed-input assertions. An AST comparison
confirms that exactly those three test methods changed. The 64 KiB hold,
`evidence_count_hold`, full-original and event-ordering tests remain unchanged.

The documented focused command above now passes **120/120** in **8.632s**.
Full discovery includes the README's `test_*.py` suite and the new `*_test.py`
module via the superset pattern `*test*.py`. Dependencies come from the existing
SDK interpreter and the existing initiative-control Markdown package directory;
no shared environment was modified. Exact command:

```sh
PYTHONDONTWRITEBYTECODE=1 \
PYTHONPATH=scripts/initiative_control:/Volumes/CorbanuDrive/Corbanu/.codex-work/initiative-control.oGQGyA/venv/lib/python3.14/site-packages \
/Volumes/CorbanuDrive/Corbanu/.codex-work/slack-sdk-test.Ob3i5O/venv/bin/python -B \
-m unittest discover -s scripts/initiative_control -p '*test*.py'
```

First dependency-complete full run: **494 ran; 487 passed, seven failures** in
**299.696s**, all in unchanged `test_fable_launcher.RealTmux` cases:
forced cleanup, late abort, management vocabulary, SIGTERM, interrupted-buffer
cleanup, timeout/partial final and two fresh runs. A diagnostic single-test replay
also failed and identified `subprocess.TimeoutExpired` in the synthetic binary's
five-second `--version` probe, before TMUX launch. Subsequent direct probes
succeeded; the unchanged two-fresh-runs case then passed **1/1 in 6.654s**.
No launcher code, timeouts, assertions or test exclusions were changed. These
failed attempts remain part of the evidence.

Final replay of the exact full-discovery command above: **494/494 passed in
276.132s**, with zero failures, errors or skips. This includes the unchanged real
TMUX launcher cases and all twelve new briefing regression tests. The test and
implementation tree was unchanged between the failed full attempt and this
passing replay. Whitespace checks pass. The scope blocker is resolved; the
synthetic briefing measurement remains **245123 → 7197 bytes** with the unchanged
65536-byte production limit. Parent still owns live-manager acceptance; no push
or release is claimed.

### September 14 terminal outcome preview repair

Action `briefing-previews-01`, allocation digest
`6557ec41aaf752d415cce0cd20dab53694e2d2e43891fae22b95489c86fa10e6`.
Bounded internal reliability follow-up to accepted compaction `673773188`, review P3,
under **Internal delivery control — TO BUILD**: “fresh Fable 5.1 High management
through Corbanu/TMUX; durable event dispatch”. Assigned branch
`bootstrap/previews-20260914`, worktree
`/Volumes/CorbanuDrive/Corbanu/worktrees/bootstrap-previews-20260914`, base
`a51602037877d7dfdcdd626fe39de26a6379b8d2`.

The four terminal outcome previews now survive compaction after a transition batch
is consumed. Owner failure/cancellation originals follow the same omission rules
as result/verification originals. Selected-event and other retained-source evidence
still expands fully, including shared references; omission digests describe only
unexpanded roots. The core packet is unchanged and previews are copied as stored,
without loading or summarizing omitted originals.

The unchanged `BriefingSizeTests.large_packet` measures **7197 → 9928 bytes**
(**+2731 bytes**, including the directive change) by loading the assigned base
module in memory and passing both versions the same synthetic packet. No encoder
ceiling was bypassed; production `BRIEF_LIMIT` remains **65536 bytes**.
Two new regression tests cover all three terminal statuses, all four outcome
fields, consumed-transition behavior, forbidden original reads and bounded
Unicode preview growth. Existing terminal/shared-evidence expectations are updated.
Against the base module, the two new tests produce three expected status-subcase
failures and one expected oversized-briefing error; against the repair they pass.

Focused SDK command: the full-discovery environment above with
`-m unittest manager_cycle_test test_manager_cycle`: **48/48 passed in 1.749s**.
Internal serialization only: the prior internal-only functional/TUI N/A applies.
The parent retains fresh-manager replay, independent evidence review and later
applicable functional/release qualification. No live state or credentials were
used; no push or release is authorized by this repair.

Final full-suite command:

```sh
PYTHONDONTWRITEBYTECODE=1 \
PYTHONPATH=scripts/initiative_control:/Volumes/CorbanuDrive/Corbanu/.codex-work/initiative-control.oGQGyA/venv/lib/python3.14/site-packages \
/Volumes/CorbanuDrive/Corbanu/.codex-work/slack-sdk-test.Ob3i5O/venv/bin/python -B \
-m unittest discover -s scripts/initiative_control -p '*test*.py'
```

**496/496 passed in 275.159s**, zero failures, errors or skips, including
`test_fable_launcher.RealTmux`. No replay was needed. Python emitted two cleanup
ResourceWarnings for synthetic HTTP 500/429 error fixtures; the suite exited zero.
All implementation/test edits preceded this run; only this result was appended
afterward. `git diff --check` passed.
