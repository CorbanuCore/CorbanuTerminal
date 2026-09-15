# Slack transport re-qualification — what the owner has to supply

**Status:** open, blocking the evidenced alert path only.
**Raised by:** Fable, 2026-09-15.

## What is broken

`Transport.gate` refuses to send unless `last_verified` is within 900 seconds.
Ours expired at `2026-09-15T12:05:48Z`, so no new decision alert can be sent
through the receipted path. The dashboard shows this honestly as
`decision_feed.slack.state = stale`.

Two separate things are affected, and only one of them is broken:

- **Evidenced alerts and threaded answers: blocked.** Anything that goes
  through `decision_manager send`, and therefore through `gate()`, refuses.
- **Reaching Travis at all: not blocked.** A direct post still works and was
  used once today, disclosed at the time as out-of-band precisely because it
  bypasses the receipted path.

## Why I cannot fix it myself

`Transport.qualify` takes `ui_evidence` whose docstring is explicit: *"Owner's
supported-UI evidence, not extra OAuth lookups or Slack assertions."* The seven
checks in `UI_CHECKS` are observations of the Slack workspace UI, not API
results. I can call the API; that is exactly what the contract says is not
acceptable evidence. Manufacturing it would defeat the control.

There is a second condition. `qualify` computes
`uncovered = ingress != value["ingress"]`. Ours is uncovered: the fence records
**45** marks against a journal `ingress` of **42**, from the listener crash at
12:22 UTC. When `uncovered` is true, `qualify` additionally requires a
`gap_review`. That is the designed mechanism for exactly our situation, and it
records the loss rather than repairing it. Three inbound callbacks were fenced
and never recorded; they are not recoverable and must not be invented.

## What I need from Travis

1. Confirm the seven UI checks in the Slack client, in this order:
   `private-channel`, `human-member`, `bot-member`, `app-identity`,
   `socket-mode`, `message.groups`, `connections:write`.
2. A receipt token for that observation.
3. Acceptance of a gap review stating that three inbound callbacks between
   roughly 12:05 and 12:22 UTC were fenced and never recorded, that their
   content is unknown and will stay unknown, and that at least one human message
   arrived in that window and was read out of band rather than ingested.

With those three, re-qualification is a single operation and the alert path
comes back. Item 3 is disclosure, not repair: the gap is recorded permanently.

## What must not happen instead

- Do not advance `ingress` to 45 to make the numbers agree. That would erase the
  only evidence that anything was lost.
- Do not re-qualify from API assertions.
- Do not treat the out-of-band post as equivalent to an evidenced alert. It has
  no receipt and no route binding, and it was disclosed as such.
