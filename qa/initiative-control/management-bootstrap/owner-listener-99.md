# RETURN — owner-listener-99

Brief SHA-256 verified:
`9960bbcab14a2b8c259050cc538cd6b3e1d1e01aa1b36aabeb753c155c00e93a`.
Base `55dd447f41fb1c00b73d74d03bcbed8a22dd580c`; runtime gpt-6-astra / high.
Allocation digest `e7e587efa25cf0f5f335ab841e24b1dea7ad8a42031a843e98d8de250770a7cc`;
claim `fcbe5752-9bc2-4c75-a5fc-e171c90d1366`.

Bounded reliability fix to already-authorized Slack intake and supervision.
Product-spec heading **Internal delivery control — TO BUILD**:
“durable event dispatch, acknowledgments and watchdog”; “actual Slack
reply/decision/agent acknowledgment”; “No arbitrary Slack reply grants
operational authority.” The existing initiative-delivery-control / PF-80-S01
context is unchanged; this worker does not advance its acceptance or edit its
plan/sprint. New audit fields remain in the existing private transport store.
Identity checks, reply attribution, dispatch authorization, runtime ownership,
qualification and outage-review rules remain enforced.

This is implementation and synthetic regression evidence, **not a human-test
readiness or deployment claim**. Independent code-blind design/execution/evidence
review and applicable live interactive qualification remain the integrator's
later gates. No independent acceptance or N/A approval is invented. No live
listener, live journal, coordinator or operator profile was operated on, no real
Slack messages were sent, and no operator auth/token contents were read. No commit or push.

## 1. Failure evidence now survives child death

The dedicated child emits one bounded JSON failure frame on its existing owned
stdout pipe. stderr remains discarded. The supervisor reads at most 1,025 bytes,
accepts at most 1,024, validates the entire fixed schema, and retains the result
across reaping and failed startup. It never persists arbitrary stdout, stderr,
exception text, tracebacks, URLs, message content or credentials.

The record is nested beside the existing exit fields:

```json
{
  "failure": {
    "reason": "transport-busy",
    "stage": "session-renew",
    "callbacks": 0,
    "disconnect_marks": 3,
    "quarantined": 0
  }
}
```

It surfaces at:

- `transport.json` body's `listener_events[-1].failure`;
- `decision_manager.project_status(...).last_listener_exit.failure`;
- the existing dashboard feed projection's
  `slack.status.last_listener_exit.failure` and `decision_feed.slack_health`
  result's `last_listener_exit.failure`.

The regression carries a real child failure through persisted state, projection
and feed-health validation. No HTML presentation change is claimed. Existing
128-event retention and pruned-event accounting still apply. A failed journal
write leaves the event pending and supervisor health unhealthy; it does not
silently discard the failure frame.

Fixed reason codes:

| Reason | Meaning |
| --- | --- |
| `transport-busy` | A BlockingIOError, such as the nonblocking transport flock refusal. |
| `transport-io` | Another OS I/O failure. |
| `validation-failed` | A rejected value/shape or missing key/attribute. The stage distinguishes credential lookup from callback validation. |
| `sdk-failed` | Other exception; no exception text or SDK response exported. |
| `restart-refused` | Restart admission refused before a session. |
| `shutdown-timeout` | Watchdog exit 72 without a usable child frame. |
| `child-signalled` | Negative process return code without a usable child frame. |
| `child-unreported` | Other proved exit without a valid bounded child frame. |

Fixed stages are `bootstrap`, `runtime`, `credentials`, `sdk-init`,
`session-start`, `connect`, `session-renew`, `callback-fence`,
`callback-envelope`, `callback-store`, `callback-ack`, `disconnect`,
`shutdown`, and `supervisor`.

A transport record preserves the **first observed failure in that child**, with
counters sampled at final reporting; it is not a claim that the last exception
was the first failure. Counters are per child and saturate at MAX_BYTES.
`quarantined` counts successful durable quarantine writes, not failed attempts.
Fallback records and failures before construction of the transport use **null**
counters: unknown is not zero. SIGKILL, startup import failure and a hung teardown
cannot be expected to emit a frame; the fallback distinguishes those limits.
Early bootstrap frames are cached even when they replace the startup handshake.

Adopt supervisor and child code together in an authorized manager-controlled
cutover. An old strict validator cannot understand the added fields. This worker
did not perform that cutover.

## 2. The “three” is reproducible without three events

The final [reproduction](owner-listener-99-reproduction-observed.jsonl), driven by
[the disposable harness](owner_listener_99_reproduce.py), executes the exact base
versions of both modules fetched with `git show`, then the revised versions.
All stores and SDK fixtures are disposable. There is no real Slack connection.

A fixture thread holds the **actual transport flock** while the listener tries
to publish its connected session. Session.update raises BlockingIOError and
releases lifetime ownership. The code then appends:

1. one fence mark in the exception path's `disconnected()`;
2. one in the outer finally, before SDK close;
3. one in the nested finally, after SDK close.

Both base and revised children exit **1**, with **fence 3 / ingress 0,
zero SDK callbacks and zero message events**. The revised child additionally
reports `transport-busy / session-renew / callbacks=0 / disconnect_marks=3`.
The runtime unit regression separately injects a lifecycle-write BlockingIOError
and proves the same signature.

Thus the fence is an arrival/disconnect barrier, **not a callback counter**.
106/103, 109/106 and 112/109 alone cannot establish that any messages were lost.
Real lock contention is a proved cause of this exact signature, not a proved
diagnosis of the live incident. The live child left insufficient diagnostics to
choose among causes retrospectively. No live restart was performed to obtain
new evidence. Lock-contention failure remains fail-closed; this change does not
silently forgive a missed renewal or claim it fixed every possible exit.

The earlier fault-injection reproduction attempts are retained in
[the first record](owner-listener-99-reproduction.jsonl) and
[the final-source replay](owner-listener-99-reproduction-final.jsonl);
the [first flock run](owner-listener-99-reproduction-lock.jsonl) is also retained.
The final replay additionally instruments actual callback invocations in both
versions; its zero-callback result is measured, not merely inferred from the
fixture construction.

## 3. A separate poison-event bug is reproduced and repaired

The base transport receives a synthetic unknown-delete event followed by a
valid reply from the pinned human in a bound decision thread. Result:
active false, fence 2 / ingress 0, **zero acknowledgments and zero recorded human
events**. normalize() rejects the unknown deletion; callback() catches the
exception and disables intake; the following reply fails the active check.

The revised transport gives the same sequence **two acknowledgments, fence 2 /
ingress 2, one quarantined delete, one recorded human reply, active true**.
A regression also drains that reply into the reply ledger.

Authenticated, bounded envelopes that fail normalization now receive a durable
metadata quarantine before acknowledgment. The journal's sibling `quarantine`
object retains a total count and the newest 128 records, each containing only:

- fixed `reason`: `unknown-delete`, `unbound-human-thread`,
  `nonhuman-author`, `unsupported-subtype`, or `invalid-message`;
- local numeric `ingress` offset;
- `author_hint`: `pinned-human`, `other`, or `unknown`;
- `shape`: `message`, `edit`, `delete`, or `other`.

These are metadata receipts, not stored raw callbacks or replayable content.
The author hint is derived from the payload's author field; it is not decision
authorization or proof that a deletion was performed by that human. A replay of
the same rejected callback produces another receipt and ACK without starving
following intake. Pruning retains the total count.

The cases cover another user, unknown deletes, edited-author mismatch,
unsupported subtype, an unbound human thread, null/list events, null edited
message, list subtype, and text rejected by normalized-message limits.
Outbound echoes and irrelevant events retain their existing behavior.

Quarantine preserves `ingress-held`: a malformed possible answer still needs
owner review. Intake and draining continue, but qualification/dispatch authority
does not silently resume. No rejected callback is attributed to a decision.
An invalid SDK session, wrong team/app, oversized outer envelope, duplicate
event-ID conflict, queue exhaustion, fence failure, lock failure or durable-write
failure retains the existing fail-closed path. Failed quarantine writes produce
no ACK and leave the fence uncovered. This is intentionally narrower than
catching all transport failures as benign poison.

## 4. Why restart cannot clear the hold on its own evidence

No automatic hold clearing was added. A connected replacement and covered local
fence prove neither that Slack delivered all messages during the disconnected
interval nor that an edit/deletion was not missed. This listener has no durable
remote replay cursor or complete history reconciliation. Its send-reconciliation
history lookup is bounded and matches outgoing post attempts; it is not an
ingress completeness witness. Inventing a gap-review receipt would bypass the
existing authorization boundary.

The existing exact recovery regression remains required and passes only when
it proves: connected old session; actual child kill; covered local fence and
pending restart; replacement with new session/epoch; `outage-gap` still held;
explicit qualification with exact gap evidence; then hold cleared. A stopped
session is not auto-restarted by the supervisor. A missing/uncovered fence also
prevents automatic restart; explicit start is a manager operation.

The operator/manager must use the existing reviewed recovery path:

1. Obtain an actual connected listener in an authorized operation. If automatic
   restart was refused, explicitly start it through the supervisor; do not edit
   journal/fence counters. Investigate any failure using the new fixed record.
2. Review channel/decision-thread history across the outage, plus the retained
   quarantine metadata. Recover any affected answer/edit/delete through the
   existing verified reply path, or have the human re-send a valid answer when
   its payload was never retained. Record actual review evidence, not an
   assertion that a zero local gap proves completeness.
3. Drain pending accepted events. Observe the current binding, watermark,
   fence ingress, live session ID and epoch.
4. Invoke existing `qualify` with supported-UI evidence
   (`binding, observed_at, receipt, checks`) and `gap_review`
   (`watermark, binding` digest, `evidence, ingress, session, epoch`).
   Qualification verifies auth/scopes, requires all pending accepted events
   drained, and rechecks unchanged session/epoch/fence/watermark across I/O.
   Any race invalidates that attempt; obtain a fresh real review, not a blind
   counter advance.
5. Confirm the fresh projected state and cleared hold. Retain the incident and
   quarantine records; qualification does not erase them.

Autonomous recovery would require a separately specified and tested completeness
witness; a successful TCP/WebSocket reconnection is insufficient.

## 5. Was any real lost callback Travis's answer?

**Unknown; the supplied evidence cannot establish even that there were three
callbacks.** The proved three-mark reproduction has no human event at all.
The poison reproduction does lose a synthetic following human reply on the base
version, and preserves it on the revised version. Neither observation identifies
the live channel's history or resolves either open decision.

The old fence contains only exclamation marks, not event identity, content or
authorship. Events rejected before the durable ingress write were not retained.
No honest reconstruction of Travis's answer is possible from those bytes. The
manager must inspect the relevant real threads under its existing access and
reconcile with recorded event IDs; this worker did not read the live profile or
attempt credential access.

## Corrections to the brief

- “Three lost callbacks” is not established: disconnect/cleanup appends count
  toward the same fence. A real transport-lock collision reproduces all three.
- The dedicated child's own handler silently swallowed exceptions and called
  os._exit(1). The generic stderr message belongs to the outer CLI handler; it
  was not a usable cause record from the dedicated child.
- A normalization rejection definitely disables intake, but it does not itself
  directly raise out of callback() to terminate the process. Later reconnect,
  renewal or shutdown failure can cause the eventual nonzero exit.
- Covered local ingress and reconnect alone cannot authorize outage clearance.
- Whether the live incident was poison, contention or another failure remains
  unresolved until actual diagnostic or channel-history evidence exists.

Exact test commands, nonzero accounting, failure names and changed lines are in
[the verification record](owner-listener-99-verification.md).
