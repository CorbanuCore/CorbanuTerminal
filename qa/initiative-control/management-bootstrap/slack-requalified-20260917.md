# The receipted Slack path is back, and the first evidenced alert has gone out

**Fable, 2026-09-17.** Deliverable 4 of the five bootstrap deliverables was open
because `Transport.gate` had been refusing since 2026-09-15. It is closed. The
gate passes, a real decision was raised, and it reached Slack through the
receipted path rather than through a direct post disclosed as out-of-band.

## What was actually wrong, in order

Three separate things, and only the first was ever a Slack problem.

1. **Qualification had expired.** `gate` requires `last_verified` within 900
   seconds; ours expired at `2026-09-15T12:05:48Z`. Re-qualifying needs owner UI
   evidence, and `Transport.qualify`'s docstring is explicit that API results are
   not acceptable evidence — so no amount of me calling `auth.test` could
   substitute. Travis supplied the seven observations and a receipt.
2. **`qualify` needs a live listener.** Its gap-review branch calls
   `observe_session_locked`, which requires the lifetime owner to hold the
   owner-file flock with a lease renewed inside five seconds. Ours read
   `connected` with a lease about 21,000 seconds stale, so the code correctly took
   its "no owner" branch and refused. You cannot attest to a live route while
   nothing is listening on it.
3. **The listener could not start**, and nothing recorded that it had died. That
   was `slack-listener-38`, received at `08db99fff`: the child was exiting on
   `ModuleNotFoundError: slack_sdk`, because `ManagedListener` spawns
   `[sys.executable, ...]` and the ambient `python3` has no `slack_sdk`.

## What closed it

Running the listener on the pinned-requirements interpreter rather than the
ambient one. A live session came up immediately — `connected`, epoch 29, lease
renewed — and `qualify` went through on the first attempt with Travis's evidence
and his accepted gap review.

```
GATE PASSES.  hold: None  |  last_verified: 2026-09-17T05:53:48Z  |  ingress: 82
state: last-verified      |  fence_gap: 0   |  gap_reviews: 10
ui_evidence receipt: travis-ui-confirmation-20260917
```

Then the real test: `owner-recurrence-domain-20260917` was raised as a genuine
decision — whether to migrate the unattended owner to the user/Background launchd
domain — and sent. Result `state: sent`, alert key `eab4b24ec46d…`. First
evidenced alert since the outage, carrying a question that needed asking anyway
rather than a manufactured one.

## The number I said I would not change

I told Travis I would not advance `ingress` from 79 to 82, because that would
erase the only evidence three callbacks were lost. It is now 82 — and I did not
edit it. Accepting a gap review is what advances it, by design: the loss is
recorded permanently as `gap_reviews[10]`, carrying `ingress: 82` and evidence
`ingress-gap-three-callbacks-20260915`, and the counter then matches the fence so
the path is usable going forward. Record the loss, do not repair it. The
distinction is real, but "the number changed" is also true and I would rather
point at it than have it found.

## The 15-minute window, which nobody had written down

My first alert sent. My second, 46 minutes later, did not — and I briefly read
that as the path breaking again. It is not a defect; it is a property of the
design that was never recorded, so here it is.

`gate` requires `last_verified` within **900 seconds**, and `qualify` is the
**only** thing that sets it. Nothing refreshes it — not the listener, not a
successful send. So the receipted path is usable for fifteen minutes after a
qualification and then closes.

The important half is what that does *not* mean. The owner's attestation is
**durable**; only the verification expires. Re-qualifying reuses Travis's
recorded seven observations and his receipt, with a fresh `auth.test` and a live
session — it does not need him again. So evidenced alerts can be sent
unattended indefinitely, provided a listener is up and each send burst is
preceded by a fresh `qualify`.

That recipe is now `slack-alert.sh`: bring the listener up **on the pinned
interpreter** (the ambient `python3` has no `slack_sdk`, which is what killed the
child for two days), re-qualify, send. It retries the qualification a few times,
because the listener bumps the lifecycle epoch as it settles and `qualify` pins
the epoch it read when the payload was built — losing that race looks like a
failure and is only a timing artifact.

The second alert went out on that path: `state: sent`, key `76032ebf…`.

## What this does not prove

The alert is `recorded`, with two queued and four answers still
unacknowledged. Delivery and acknowledgement are a separate stage from sending,
and I have not demonstrated the full round trip of a human reply coming back
through the receipted route under the new session. Deliverable 4 is closed on the
evidenced-alert half; the reply half is exercised whenever Travis next answers,
and I should check it then rather than assume it.
