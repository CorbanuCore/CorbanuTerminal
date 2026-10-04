# PF-80-S01 owner-controls candidate

Uncommitted internal engineering candidate for parent review. Product initiative:
**Internal delivery control — TO BUILD**, “Use sequential sprints per initiative”.
The corbanu-terminal-development skill routes this work through the active
initiative-delivery-control plan and in-progress PF-80-S01; the exact four-file
[allocation](../../../docs/research/tasknode-integration/coordinator-bootstrap-20260913.md)
overrides historical broader scopes. Plan/sprint ledgers remain parent-owned.

Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/coordinator-bridge-20260913`.
Branch: `bootstrap/coordinator-bridge-20260913`.
Implementation base: `84f16cceec87292296ff94a9d431a86d23a9fe52`.
Allocation HEAD: `875613aab6c160b6ebf12f53b8b33152a7986c32`.
Prior reviewed code and core reviews01–05 were inspected; this unit adds no
opinion reviews. Parent owns one Fable material review and receiving integration.

## Owner CLI contract

Use the existing `python3 scripts/initiative_control/coordinator_cli.py OPERATION
--state PRIVATE_DIRECTORY` entry point with bounded JSON on stdin. Every new
operation requires integer `expected_revision` from `snapshot` and a nonempty
owner `evidence` object. Stale requests fail atomically, including after restart;
each successful command appends an audit record and meaningful event. An owner
mutation invalidates an in-flight manager decision; use existing `fail_manager`
only after launcher shutdown/reconciliation, then begin a fresh manager cycle.

| Operation | Additional JSON fields | Effect and conditions |
| --- | --- | --- |
| `put_allocation` | `allocation_id`, `allocation`, `replace` boolean | Explicit add or replace. Uses the existing frozen allocation shape. Replacement refuses dispatching/uncertain/dispatched/running/returned reservations. Cancels prepared actions; retains old/new allocation evidence and action history. Duplicate content and missing/existing target mismatches fail. |
| `set_stream_mode` | `workstream`, `mode`: `enabled` or `paused` | Changes one stream. Does not release active reservations or override global pause. Repeating the current mode fails. |
| `complete_sprint` | `action_id`, `gates` | Requires an accepted `complete_sprint` action, accepted receiving proof, all mandatory owner-verified gates, dependency archives, an active/blocked reservation, no pending actions, and no pause. Records completed, unarchived state. |
| `archive_sprint` | `sprint` | Requires this controller's verified completion record, completed/unarchived state, archived dependencies, no pending actions and no pause. Records the archive flag only. |
| `activate_successor` | `action_id` | Requires an accepted `prepare_successor` action for a draft in the same stream, the current predecessor among its completed/archived dependencies, exact predecessor receiving base, no pending predecessor/successor actions and no pause. Transfers the reservation and stream pointer. |

Allocation fields remain `sprint`, `kinds`, `resources`, `scope`, `inputs`,
`timeout_seconds`. Owner allocations containing `complete_sprint` must freeze
these input fields before the manager prepares the action:

```json
{
  "receiving_action": "OWNER_SELECTED_ACCEPTED_ACTION_ID",
  "receiving_commit": "EXACT_40_CHARACTER_LOWERCASE_SHA",
  "mandatory_gates": ["OWNER_SELECTED_CANONICAL_GATE_IDS"]
}
```

This is a schema illustration, not executable evidence. The owner must enumerate
every applicable canonical gate; the module does not read or interpret policy
documents. The mandatory list must be nonempty and unique. Manager/worker input
cannot alter it. `gates` must have exactly those keys, each containing
`owner_verified: true`, `status: passed` or `not_applicable`, and a nonempty
`evidence` object. N/A also requires an explicit `reason` and owner verification.
This records an authorized disposition; it grants no authority to waive policy.

The receiving action must be `integrate` or `verify_integration` in the same
sprint, with the exact Integrator assignment frozen at `inputs.assignment`.
Its owner `verify` evidence must contain `receiving_receipt`: the inspected
Integrator receipt, status `verified`, exact matching assignment/receiving commit,
and every required test with matching argv, integer exit code zero and no timeout.
Worker-returned claims alone never qualify. Completion and receiving actions can
be retrieved from older SQLite action history, but their allocation digests must
still match. The successor preparation action freezes `inputs.base` to that
receiving commit. Accepted actions alone never advance sprint lifecycle.

## Verification and limits

Focused commands: `python3 -m unittest discover -s scripts/initiative_control -p
'test_coordinator.py' -v` and the same command with `test_integration.py`.
All 25 original coordinator tests are retained. New CLI cases cover add/replace,
active/uncertain/returned refusal, stale claims and proofs, stream/global pauses,
restart, history retrieval, competing revisions, completion/archive/successor,
missing/unverified/N/A gates, wrong/failed/incomplete receiving evidence, exact
successor base and draft/reservation denials. Rejection checks assert unchanged
state and event/audit/history/evidence row counts. Every receipt in the new tests
is synthetic; no live native dispatch, receiving acceptance or lifecycle change
is claimed. Existing integration tests exercise only disposable Git repositories.

Internal CLI/state engineering has reasoned internal-only N/A for GUI/TUI and
TensorCash/Isometric acceptance in this unit, for parent acceptance. Later combined
dashboard/Slack and native workflows retain independent functional/live gates.
No production store, credential, scheduler, shared policy or sprint document was
changed. Existing stores reopen without schema migration or reinitialization;
legacy completed records without verified completion cannot be auto-archived.
Prepared passive wait/pause/cancel/question/successor-preparation keeps the prior
pause exemption; active work and all lifecycle transitions enforce pause.
Already dispatched work still requires explicit native shutdown/reconciliation.
The CLI remains trusted-owner-only, never a worker/Slack/web command endpoint.
It validates persisted structure/bindings, not the truth of supplied artifacts;
the parent must inspect actual receipts, gate applicability and evidence freshness.
Security/accounting/product resumption, recurring enablement, actual document
archive, live integration and full PF-80-S01 acceptance remain with the parent.

## Final local receipt

`PYTHONPATH=scripts/initiative_control python3 -m unittest test_coordinator
test_integration -v`: **50 passed in 4.252s**, exit 0 (37 coordinator, including
25 unchanged originals and 12 new tests; 13 unchanged integration tests).
Plan checker: active 3/3. Sprint checker: current 115, archived 126. CLI help
and `git diff --check` pass. No failed local test attempts in this unit.
The parent's later launcher/socket fixture results are separate from this tree.

Literal delta: coordinator.py +181/-12; coordinator_cli.py +1/-0;
test_coordinator.py +275/-0; this new receipt +105/-0. Total **574 changed
lines**, **299 non-test including this receipt** (194 implementation-only),
within target600/350 and hard950/550. No commit/push or live-state mutation.

Parent acceptance: owner-controls Fable 5.1 High review01 completed with exit0
and no actionable findings. Exact private result is retained in
`.codex-work/bootstrap-owner-review.33HBPR/review01.json`. The integrator accepts
this bounded internal increment and its reasoned N/A; actual artifact inspection,
document archive, native/event wiring and full-loop qualification remain open.
Worker closed after handoff. Parent owns the following commit/receiving job.
