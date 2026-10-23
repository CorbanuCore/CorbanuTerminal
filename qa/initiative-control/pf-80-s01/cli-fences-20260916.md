# Scoping the Task Node CLI's recovery, input, retention and restart fences

**Fable, 2026-09-16.** The delivery sprint lists this as manager work before a
CLI allocation, and explicitly not an unanswered product question. It has been
sitting unmade, which is latency rather than caution, so here it is. Everything
below is read off the current `scripts/initiative_control/tasknode.py` and
`control.py`, not proposed in the abstract.

## What exists today

- The queue is guarded by `control.locked()`, which is
  `fcntl.flock(stream, fcntl.LOCK_EX)` with **no** `LOCK_NB` and no timeout.
- `flush` increments `attempts`, and on a transient failure sets
  `next_attempt_at` to `min(3600, 30 * 2**min(attempts, 7))` seconds out, marking
  the record failed at eight attempts.
- `retry` resets `status="pending"`, `attempts=0`, `next_attempt_at=now()`.
- `preview` reads exactly one immutable record with no credentials, locks,
  writes or transport, and reports a `blockers` list.
- The module's own comment on recovery: *"Reconciliation is external; deleting
  receipts is not recovery."*

## The fences I am freezing

**Cross-process.** The lock stays exclusive, but a CLI must not inherit a
blocking-forever wait. Any CLI entry point acquires with a **bounded wait**, and
on expiry exits non-zero with "another Task Node operation is holding the queue"
naming the lock path. A human at a terminal must never see a hang they cannot
explain. `locked()` keeps its current blocking behaviour for in-process callers;
the bound belongs at the CLI edge, so no existing caller changes semantics.

**Restart.** A process that dies between the HTTP send and the receipt write
leaves a record whose delivery is genuinely unknown, and no local inspection can
resolve that. The CLI therefore **never** auto-retries such a record on restart
and never treats a missing receipt as a failed send. It reports the record as
uncertain and requires an explicit `retry` naming the event id. An uncertain
delivery retried automatically is how a duplicate gets posted.

**Recovery.** Read-only by default. The CLI may inspect, and may `retry` a named
record. It may **not** delete a receipt, edit a payload, rewrite an id, clear the
outbox, or reset `attempts` in bulk. The existing comment already says deleting
receipts is not recovery; the CLI must not offer the operation at all rather than
offering it with a warning.

**Input.** No credential may be passed on the command line or through an
environment variable the CLI reads itself — the existing `--credentials-file`
path stays the only route, and the CLI never echoes its contents. No free-form
event payload: events are built by `event_for` from a validated run record, so a
CLI that accepted arbitrary JSON would bypass every field and byte check. Event
ids are accepted only as selectors for records that already exist.

**Retention.** Outbox records and their receipts are retained until an explicit
owner action, not aged out. This is deliberate and it is the opposite of the
accounting store's policy, for a reason worth writing down: accounting retention
exists to bound storage of high-volume telemetry, whereas an outbox record is
evidence of whether something was sent to an external service. Aging one out
silently would destroy the only local record that a post happened. If a retention
policy is ever wanted, it needs its own decision and an archival destination, not
a default.

**Accessibility.** Every refusal names the blocker in words rather than a code,
and the reason is on the same line as the refusal, so a screen reader reaches it
without navigating. Status output stays one JSON object per line, which is both
machine-readable and linearly readable. No output depends on colour or on
terminal width, and nothing is conveyed by position in a table alone.

## What this does not decide

Posting stays OFF and nothing here turns it on. Credential scope and
destination, enrollment, task lifecycle and the first payload are still Travis's
to decide — they are a separate Remaining item and I am not folding them in. This
scoping covers only how a CLI may behave once those exist.
