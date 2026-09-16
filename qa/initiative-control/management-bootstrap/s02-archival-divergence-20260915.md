# The document ledger and the coordinator disagree about PF-60-S02

**Fable, 2026-09-15.** Recorded while awaiting an owner decision, so the state
is not discovered later by someone who trusts one side of it.

## What diverged

| | Says |
| --- | --- |
| Documents (`fc315cb6b`) | PF-60-S02 `status: completed`, moved to `docs/sprints/archive/`, all obligations transferred to S03 |
| Coordinator state | PF-60-S02 `status: blocked`, `archived: false`, `source_path` still `docs/sprints/current/...` |

PF-60-S03 does not exist in coordinator state at all. The only PF-60 sprints it
knows are S01 and S02.

## How it happened

I performed the documentary half of an archival — moving the file, setting the
status, transferring the obligations — without first confirming the control
plane could perform its half. The governance checkers passed, because they only
read the documents, so nothing objected until the S03 dispatch was refused with
`unknown allocation sprint`. That refusal was the coordinator working correctly.

This is the same predicted-versus-verified mistake recorded in
[manager-cycle.md](manager-cycle.md) about worker scopes, one level up: I checked
the thing I was editing and not the thing that had to agree with it.

## What is actually possible, established by reading the code

- `complete_sprint` requires a prepared action of that kind referencing a
  **verified receiving action**. Those exist for S02: `82f9dde0e` and
  `533a16077`. So a correct S02 completion is achievable.
- `activate_successor` requires the successor to already exist **as a draft in
  coordinator state**.
- Sprints are created **only** by `initialize`, which is a one-time seed:
  it inserts the state row and refuses a second call with *"already initialized;
  never erase prior state"*.

So **S03 cannot be registered through the existing coordinator API.** That is a
structural limitation, not an oversight in this session.

## What was deliberately not done

The S03 implementation unit was not dispatched under the `PF-60-S02` sprint id,
which the coordinator would have accepted. That would have produced real
implementation work filed against an archived sprint and a receipt that
misstates which sprint it belongs to. A single dispatch is not worth a false
provenance record.

## Options, for the owner

1. Drive the proper lifecycle: prepare `complete_sprint` against the verified
   receiving proof, then `archive_sprint`, so both sides agree about S02. S03
   registration remains blocked pending a coordinator change.
2. Revert the documentary archival so the ledger matches the control state, and
   treat S02 archival plus S03 registration as one unit of work.

Nothing else is affected. No worker is running against either sprint, the
integration branch is clean, and the dashboard is accurate about everything
except this.

## Resolved, 2026-09-16

The owner chose option 1 and it is executed. PF-60-S02 is `completed, archived: true` in coordinator state with `receiving_commit 533a160778e258f8e11863e26b4c6ea1a395102c`, matching the documents.

One correction to what this document originally said. It claimed `complete_sprint` was achievable because "verified receiving actions exist". They did not: there were **no `integrate` or `verify_integration` actions in the coordinator at all**, only the Integrator's on-disk receipts. A `verify_integration` action had to be created and independently verified first — a separate worker confirmed the receiving commit is an ancestor of the tip, the merged source is present, every changed file lies inside the frozen scope, and all eight receiving tests recorded `exit_code 0` with `timed_out false`, returning PASS with no discrepancy. The recommendation was right; the reason given for it was wrong.

Four owner-verified gates were supplied: receiving tests, governance checks, independent review, and the internal-stage functional gate as `not_applicable` with its reason and an explicit note that it is deferred to S03 and S04 rather than discharged.

**Still true:** `PF-60-S03` does not exist in coordinator state, sprints are created only by `initialize` which refuses a second call, and the accounting workstream still points at the now-archived `PF-60-S02`. No S03 work can be dispatched until that is addressed at the coordinator level.
