# RETURN — owner-recover-101

Runtime: gpt-6-astra / high.
Allocation digest: `f483e35a32197d112bd76e2ed6e65e781689b3e04b48b72d54d9844b09d3a6f4`.
Claim: `fee80dbc-73c7-4e22-a73e-33d56916c470`.
Base and starting HEAD: `181b2d72fa02e4208d5472d39f55924e7bc9a34c`.
Brief SHA-256 verified before other reads:
`12f9281c4efdb4df41fc96840369b1b1af553d6e03603971dc9591764835ce31`.

Bounded reliability repair under product-spec heading **Internal delivery
control — TO BUILD**: “durable event dispatch, acknowledgments and watchdog”;
“The matching Slack thread should log his answer and route it to the appropriate
agent through a verified identity/revision-aware manager handoff”; “No arbitrary
Slack reply grants operational authority.” Existing initiative-delivery-control /
PF-80-S01 context; no plan/sprint advancement, deployment, or acceptance claim.

## Held, then bound

Authenticated, validated unbound human messages are durably stored in
`transport.held_human` before ACK. This holds only the normalized reply envelope
needed for eventual delivery, original ingress offset, arrival and expiry.
Retention is **900 seconds / 15 minutes**, with **at most 100 pending envelopes**
and the existing **900,000-byte callback journal admission bound**. A duplicate
event ID with identical content does not extend its deadline; conflicting IDs,
capacity exhaustion and write failures receive no ACK, preserve already-held
replies, and leave the independent ingress fence fail-closed.

The exact alert's `bind_alert` installs its immutable thread route and moves
eligible held envelopes to normal transport events in the same durable write.
It never attributes a follower reply to its parent. Replay increments the
watermark only for a new event; normal drain and reply-ledger deduplication
deliver once. A failed binding/replay write leaves the original retained reply
available for retry. Edits and deletes can resolve their original message from
the held queue, and replay is ordered by ingress.

Expiry wins at the exact deadline even if a route arrives then. Expiry removes
the held content and appends an explicit **unbound-expired** quarantine event
with timestamp, original ingress, pinned-human/shape hints, channel and message
timestamp. A late binding cannot resurrect that retained envelope. Content is
not copied into quarantine. A reply found to be the eventual root/details
message becomes an explicit invalid-message quarantine record instead.

Maintenance runs on normal listener session updates (including idle renewals),
callbacks, binding and drain. It uses the existing transport lock and durable
write; it adds no scheduled qualification or new background store-lock poll.
If no process is running, expiry is enforced and recorded on the next such
operation before replay. There is no claim of wall-clock disk deletion while
the application is stopped. A failed expiry write retains the original reply
and raises; a later successful operation records expiry atomically.

A transport hold is not automatically cleared by binding or expiry. Existing
outage review, qualification, identity/revision checks and explicit manager
interpretation remain required. This is intake recovery, not answer approval.

## Quarantine visibility and minimum locator

Manager status, the saved dashboard Slack projection, and
`decision_feed.slack_health` now expose:

```json
{
  "supervisor_health": {
    "state": "unhealthy",
    "reason": "quarantined-intake",
    "quarantine": {
      "count": 1,
      "held": 0,
      "oldest_at": "2026-09-12T12:00:00Z",
      "age_seconds": 60
    }
  }
}
```

Count includes retained-for-binding replies plus unresolved quarantine total.
The existing bounded audit keeps its latest 128 records; its total and oldest
time survive pruning. Legacy audits without arrival times report null age,
not a fabricated zero. Saved health recomputes age at observation. Nonempty
intake forces overall Slack status held and prevents supervisor health from
being healthy, even after an explicit transport qualification clears hold.
No quarantine-resolution UI or implicit audit-clearing action was introduced.

New content-free quarantine records keep **channel and message_ts** as the
minimum Slack locator, plus the audit timestamp. A malformed/missing message
timestamp becomes null; no identifier is invented. Old records cannot be
retroactively enriched without obtaining their missing information.

**Scope limitation:** the visible dashboard HTML renderer is
`scripts/initiative_control/attention.py`, outside the frozen writable list.
The real projection carries count/age and the existing HTML displays held,
but the HTML does not yet print those two numeric fields. The
[reviewable renderer follow-up](owner-recover-101-renderer-followup.patch) is
provided without applying it. It needs an integrator allocation permitting
that file and renderer regression validation. No misleading HTML-complete
or unqualified handoff claim is made.

## Clean exit

A proved exit 0 with no failure frame returns no failure record. The supervisor
can still record the ordinary child-exit event with returncode 0, but omits
`failure`. Nonzero unframed exits retain child-unreported; signals and reserved
exit codes keep their existing distinctions. A valid explicit failure frame
remains authoritative even for exit 0, and malformed nonempty output is still
unreported. Clean results are cached across pipe closure/reaping.

## What the three live exits would have shown

For **18:07, 18:08 and 18:54**, the answer is conditional, not a retrospective
diagnosis. If each followed the reproduced lock-contention path, all three
would report:

```json
{"reason":"transport-busy","stage":"session-renew","callbacks":0,"disconnect_marks":3,"quarantined":0}
```

The [fresh harness replay](owner-recover-101-reproduction.jsonl) uses actual
transport flock contention in disposable children. Both the historical base
and revised child exit 1 with fence 3 / ingress 0 and zero SDK callbacks.
The revised child emits the frame above. The harness's historical comparison
base is round 99's `55dd447f41fb1c00b73d74d03bcbed8a22dd580c`, not this
allocation's starting HEAD.

This code identifies **transport lock contention during session renewal** for
that path. It does not identify the lock holder or prove that the manager's
qualification was the competing operation. The old live fence/ingress pairs
106/103, 109/106 and 112/109 are consistent with three teardown disconnect marks,
not evidence of three lost Slack messages. The historical children emitted no
cause frame, so no truthful exact record can be assigned retrospectively.

To establish the cause of a future incident, capture the actual bounded child
frame and correlate its stage/time with the manager's operation timeline.
Identifying a competing owner requires operation-scoped lock acquisition/hold
telemetry (fixed operation identifiers and durations, no credentials/content).
SIGKILL or teardown watchdog termination can still leave only child-signalled
or shutdown-timeout fallback evidence. No live listener was restarted to gather
this evidence.

## Validation and readiness

See the [verification record](owner-recover-101-verification.md) for commands,
counts, raw outputs, source hashes and nonzero accounting.

This is an implementation return to the manager. Independent code-blind
functional design, isolated execution, evidence review, applicable live TUI
qualification and named-human acceptance are not claimed. The manager retains
those later handoff gates. No real Slack message, live listener/store/coordinator
operation, native credential prompt, token read, commit or push occurred.

## Brief corrections

The brief correctly identifies the ACK/discard race, invisible quarantine,
missing locator and clean-exit misclassification. The qualification/lock
explanation remains plausible and reproduced, not proved for the three live
events. The visible count/age requirement requires the renderer file omitted
from this allocation; the data projection is fixed and the exact HTML follow-up
is supplied separately. Historical discarded content and missing identifiers
cannot be reconstructed from the prior audit.
