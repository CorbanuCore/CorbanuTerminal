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
The briefing includes all three streams, their last three actions, frozen scopes,
pending actions, and recursively loaded original evidence keyed by verified digest.
Original references and previews remain attached; previews never replace originals.
There is no summarization or truncation fallback. Missing/corrupt evidence, excessive
reference count, or a briefing exceeding 65536 encoded bytes produces an owner hold
before authentication/inference, with pending events and the claim retained.
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
