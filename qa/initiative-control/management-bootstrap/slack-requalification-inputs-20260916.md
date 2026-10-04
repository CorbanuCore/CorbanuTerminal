# Travis's re-qualification inputs, held ready — and the defect that stops us using them

**Fable, 2026-09-16.** Both owner inputs the Slack re-qualification needed are now
in hand. I could not complete the operation, for a reason that is a genuine
defect rather than a missing input, so I am recording the inputs precisely
enough that nobody has to ask him again.

## What the owner supplied

**The seven UI observations.** Confirmed by Travis in-thread: `private-channel`,
`human-member`, `bot-member`, `app-identity`, `socket-mode`, `message.groups`,
`connections:write` — all pass. Receipt token `travis-ui-confirmation-20260917`,
which identifies his confirmation rather than pretending he supplied a secret.

**The gap review, accepted verbatim:**

> Between 2026-09-15T12:05:48Z and roughly 12:22 UTC, three inbound Slack
> callbacks were fenced and never recorded. Their content is unknown and will
> remain unknown. At least one human message arrived in that window and was read
> out of band rather than ingested. This is recorded permanently as loss, not
> repaired.

The "three" is not from the incident write-up. It is the live store: recorded
`ingress` 79 against an `.ingress.fence` of 82 bytes, and `project-status`
independently reports `fence_gap: 3`.

## Why it still did not go through

`qualify` refuses, and correctly. Its gap-review branch calls
`observe_session_locked`, which requires the **lifetime owner of the listener to
be holding the owner-file flock with a session lease renewed inside five
seconds**. Ours is not: the recorded session still reads `phase: connected` at
epoch 28, but its lease is about **21,000 seconds** stale. The flock is free, so
the code takes its "no owner, including SIGKILL before any replacement starts"
branch and refuses.

That is the control behaving exactly as designed. You cannot attest to a live
route while nothing is listening on it.

## The actual defect, and a second one behind it

Starting the listener fails. `decision_manager.py listen --live` exits through
`d.require(manager.process.returncode == 0)`: the child process starts and dies
immediately. `supervise-listener` runs, updates its observation timestamp, and
leaves `supervisor_health.state` at `unknown` with reason `observation-stale`.

The second one is worse than the first. **`listener_exits` is still `0` and
`last_listener_exit` is still `null`.** A child died twice while I watched, and
nothing recorded it. Increment D exists precisely to record listener exits
durably — that was its motivating incident, a listener that died at 12:22 and
stayed dead about two hours with nothing surfacing the gap. So either this path
is outside what increment D supervises, or increment D's recording does not cover
a child that dies during startup rather than after it. Both are worth knowing and
I do not yet know which.

Diagnosis is also deliberately hard here, and that is not an accident to fix
casually: the child's stderr goes to `DEVNULL` by construction, and every CLI
failure is redacted to "Slack operation held; inspect redacted status and
retained evidence." Those choices are right for a credential-bearing path. They
do mean the durable exit record is the *only* intended diagnostic, which makes
its absence the thing to fix first.

## What must not happen

Do not advance `ingress` from 79 to 82 to make the numbers agree. The gap review
records the loss; erasing the evidence is not the same as accepting it. Do not
re-qualify from API assertions — I already know `auth.test` succeeds, and that is
exactly the evidence `qualify` refuses. Do not treat a direct post as equivalent
to an evidenced alert; every one I have sent tonight was disclosed as out-of-band.

## State when the fix lands

Nothing above expires. The binding, watermark 5, epoch 28, the receipt token and
the accepted gap review are all recorded here. When a listener holds a live
session, re-qualification is one operation and the alert path returns.
