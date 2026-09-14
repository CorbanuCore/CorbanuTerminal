# Private initiative control: receiving PF-80 slice

Product initiative, PF-80-S01, in progress / awaiting manager review.
Product citation: **Internal delivery control — TO BUILD**, “Use sequential
sprints per initiative”. Internal operations only; no release or enablement.

This is the bounded port from recovery delivery control. Provenance and exact
receiving-tree test results are under `qa/initiative-control/pf-80-s01/`.
The recovery runbook described a deployed Linux service. The September 12 bounded
dashboard status/source repair has an explicitly authorized receiving-source
publication path; its exact deployment receipt is separate from native delivery
qualification. Source inspection and synthetic tests do not prove live
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
send. The separate guarded sender below requires batch posting to remain OFF.
The timer's existing delivery path requires literal `enabled: true` and local enrollment;
absent/false enablement remains OFF. Keep posting OFF throughout preparation.
The inherited enrollment/flush/activation entry points are retained for source
compatibility, but invoking them against live state is outside this slice.

## Single-event readiness candidate (not live-qualified)

The PF-80 readiness allocation adds `send --state STATE --event-id EXACT_CC_ID`
with exactly one of `--dry-run` or `--live`. This is internal implementation
evidence, not shipped guidance or authority to post. Dry-run validates the
immutable schema/hash and explicit PF-80-S01 mapping, returns a logical request
digest and gate observations, and performs no writes or credential reads.

Live mode additionally requires `--owner-activation-file FILE` and
`--credentials-file FILE`. It keeps `tasknode.enabled: false` (batch OFF),
requires local enrollment, a pending fresh observation, elapsed backoff, and
matching current workspace/mapping. It never calls flush, enqueue or retry and
never changes the outbox/index/config/enrollment. Exactly one request goes to
the fixed production events endpoint, with the immutable event ID as its
idempotency key. The inherited batch transport is unchanged.

The activation file must be a regular file owned by the executing OS user with
no group/other permissions. The manager supplies this exact schema:

```json
{
  "schema": 1,
  "owner": "NAMED_OWNER",
  "enabled": true,
  "event_id": "EXACT_CC_ID",
  "request_digest": "DIGEST_FROM_DRY_RUN",
  "workspace_id": "EXACT_WORKSPACE",
  "task_ids": ["EXPLICIT_PF80_TARGET"],
  "origin": "https://tasknode.postfiat.org",
  "expires_at": "OWNER_SELECTED_EXPIRY_WITH_TIMEZONE",
  "gates": {
    "identity": true,
    "entitlement": true,
    "enrollment": true,
    "target_lifecycle": true,
    "payload_review": true
  }
}
```

These are owner attestations, not independent server proof. Local files do not
authenticate the named human or fence credential rotation. Only the trusted
manager may provision activation/credentials after the remaining native identity,
destination/visibility, entitlement, lifecycle, payload and recovery gates pass.
No activation or real credentials were created in this allocation.

`send-receipts/CC_ID.intent.json` is exclusively created and fsynced before
transport. `CC_ID.result.json` is another create-once receipt with logical
request digest (excluding credentials), idempotency key, HTTP status and
allowlisted response facts: success boolean, matching event ID, deletion flag.
Raw responses, error bodies and credentials are never recorded. Receipts are
owner-only, append-only by this adapter; this is not protection against the OS
owner modifying files. Do not delete them, copy them across account scopes, or
enable batch delivery over these retained queue records.

A retry revalidates gates and returns the prior receipt without another POST.
A missing result, timeout, rejection, malformed response or crash stays uncertain
and requires manager reconciliation. Even an intent written before a crash that
preceded HTTP blocks retransmission. The server's existing account/event-ID
deduplication remains authoritative; the HTTP header alone is not delivery proof.
Dry-run creates no receipt.

`identity-check` takes no state or write flags and prints a redacted JSON receipt.
It resolves the installed helper while preserving inherited home/profile,
checks `tasknode --help` for profile support, then uses only `link status`,
`status`, and `task show` for the three documented personal targets. Missing
or conflicting scope, unsupported helper, mismatched origin/profile, command
failure, timeout or observed native credential prompt stops successor reads.
Never authorize a native prompt. Unknown task shapes/states remain unverified.
It does not invoke balance, wallet, acceptance, signing, enrollment or posting.
The current read-only CLI does not expose authoritative Tracker entitlement or
enrollment, so both remain explicitly unverified; successful status is not proof.

For this allocation use the SDK Python suite recorded in the
[readiness evidence](../../qa/initiative-control/management-bootstrap/tasknode-readiness-20260914.md).
Synthetic fixtures exercise delivery; do not invoke `--live` against real state.

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
`activate.py` is the manager-only Linux installer. It publishes once and refreshes
the private web service without enabling or disabling the publication timer.
New installations remain unscheduled until the operator separately authorizes
timer activation; a source update must not resume a paused schedule. Its synthetic
tests do not by themselves prove a real service update.
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

## Dashboard source maintenance

Manager configuration may list up to100 exact `reference_documents` Markdown
paths under `docs/research/` or `qa/` for allocation/context/evidence links.
Exporter and renderer use the same validated inventory; missing, unpinned,
unsafe or credential-like references fail publication. These are not human-test
cards, and links inside them never expand the export automatically. Managers
must inspect each selected document for private content before listing it.

The manager reconciles canonical plans/sprints, resolved decisions, human-test
cards and redacted worker reports before each ten-minute sync. The Luna Extra
High worker runs the existing private wrapper, now selecting the declared manager
receiving checkout, not recovery. Its nonblocking lock prevents overlapping syncs.
`sync-source.sh` exports the current HEAD and allowlisted file hashes, validates
and renders the bundle locally, rechecks branch/revision/content/configuration
after upload, then verifies the exact server publication. It never pulls, resets
or rebases an active checkout. Accepted upstream work and intentional source
moves must be reconciled by the manager and recorded in the same handoff.

The banner names checkout/branch/commit/content digest. Source older than 20
minutes is stale even if the server renders it again; worker notes have their own
45-minute stale threshold. Failed source sync keeps last-good pages and marks
failure through the existing private service when reachable. A failure marker
cannot be cleared by the backup 30-minute render timer; it needs a newer source
collection. If the server is unreachable, the unchanged collection timestamp
still ages and the manager reports the actionable failure.

Blocked sprint labels link to status context and full sprint records. In-progress
labels expose condensed latest reports on hover/focus and link to full notes for
click/touch. Missing notes say unknown; reports never override manager status.
Historical PF76 warnings remain held, not remapped or portrayed as PF80 blockers.
