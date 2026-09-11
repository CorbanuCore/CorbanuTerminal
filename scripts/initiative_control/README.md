# Private initiative control: receiving PF-80 slice

Product initiative, PF-80-S01, in progress / awaiting manager review.
Product citation: **Internal delivery control — TO BUILD**, “Use sequential
sprints per initiative”. Internal operations only; no release or enablement.

This is the bounded port from recovery delivery control. Provenance and exact
receiving-tree test results are under `qa/initiative-control/pf-80-s01/`.
The recovery runbook described a deployed Linux service; this receiving port
has not been deployed. Source inspection and synthetic tests do not prove live
entitlement, task ownership, enrollment, grants or successful delivery.

## Offline preparation

Install `requirements.txt` in a disposable venv. From the repository root:

```sh
python -m unittest discover -s scripts/initiative_control -p 'test_*.py'
python scripts/initiative_control/tasknode.py prepare --state SYNTHETIC_STATE --event-id EXACT_CC_ID
python scripts/initiative_control/tasknode.py preview --state SYNTHETIC_STATE --event-id EXACT_CC_ID
```

`prepare` and `preview` are the same read-only operation. They select exactly
one existing outbox event by immutable `cc-` ID, validate its hash/schema and
return its deliberately limited goal payload plus local gate observations.
They do not scan the batch, load credentials, call a transport, enqueue, retry,
write a receipt or change configuration. Live/report flags are rejected. Missing
IDs fail without selecting another event. Output always says
`network_writes: false` and `send_authorized: false`, even with local gates ON.
Enrollment receipts are only local workspace-bound evidence, not current
account identity or server entitlement. Stale observations remain visibly held.

Payloads omit transcripts, tool arguments, local paths and credentials. Reports
must be intentionally redacted before submission; pattern checks cannot detect
all private text. Human review must examine the exact payload before delivery.
Do not redact or rewrite a queued event in place: that changes its immutable
identity. Report/queue files are owner-only metadata, not encrypted storage.

The inherited `flush` remains a **batch operation, up to 20 eligible events**.
`flush --event-id` is explicitly rejected; it cannot masquerade as a one-event
send. No live single-event sender is introduced in this slice. The timer's
existing delivery path requires literal `enabled: true` and local enrollment;
absent/false enablement remains OFF. Keep posting OFF throughout preparation.
The inherited enrollment/flush/activation entry points are retained for source
compatibility, but invoking them against live state is outside this slice.

## Source and history boundaries

Recovery PF-76-S01 means delivery control; main PF-76-S01 means provider
persistence. This adapter holds every ambiguous raw PF-76-S01 report out of
main initiative joins, rejects new enqueue/retry for that ID, and leaves old
PF-76 outbox records untouched during a batch flush. Preparation can inspect
one historical event while reporting reconciliation as a blocker. There is no
automatic PF-76 → PF-80 alias and no rewrite of IDs, payloads or receipts.

New PF-80 reports represent newly observed work and require explicit PF-80
task mappings. They must not be regenerated from old PF-76 reports to evade
reconciliation. The manager owns the per-record cutover inventory and all
decisions about historical events; see `docs/research/tasknode-integration/`.

## Retained machinery

`control.py` validates immutable worker reports, collects canonical plans and
sprints, renders sanitized private HTML and retains the last good publication
on failures. Stale source and worker ages remain separate. `export.py` exports
declared document/tooling paths with hashes; it never scans agent homes.
`activate.py` is the inherited manager-only Linux installer. It is tested with
synthetic directories and mocked service calls, not executed on a real service.
`tick.py` publishes and queues explicitly mapped reports; OFF prevents network
delivery but does not prevent local report collection or queue preparation.
`delivery.py` checks candidate/evidence hashes, native flag contracts and human
enablement records; a merge check does not authorize enabling or releasing.

Native `/tasknode` account linking, profile isolation and Campaign Tracker are
the authority to reuse. The CLI currently lacks an arbitrary-event transport;
the native capture method generates new IDs and sync processes a batch. Do not
feed historical events through capture or use Sync now for one-event approval.
The manager must allocate a reviewed native transport boundary before live use.
No new auth store, credential provisioning flow, scheduler or Rust changes are
introduced here. Workers cannot self-accept reports, close sprints or release.
