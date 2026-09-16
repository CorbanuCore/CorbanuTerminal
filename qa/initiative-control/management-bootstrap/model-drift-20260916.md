# A worker silently changed model mid-run, and my attestation could not see it

**Fable, 2026-09-16.** A dispatch acknowledged as `gpt-6-astra high` and then ran
as `gpt-5.6-luna`. No work was produced, the action is recorded failed, and the
protocol hole it exposed is now closed. Travis has flagged quota twice; this is
what quota pressure looks like from inside the dispatch loop.

## What happened

`dispatch.py` launches with `--model gpt-6-astra` and
`model_reasoning_effort="high"` explicitly, so the launch was correct. The worker
replied with the required attestation line, naming astra and high. The session's
own rollout then records both:

```
"model":"gpt-6-astra"
"model":"gpt-5.6-luna"
```

The preceding worker had ended sitting on a product prompt reading *"Approaching
rate limits — Switch to gpt-5.6-luna for lower credit usage?"*. The account is
rate limited, and the product offers or performs a downgrade during a session.

## Why my records did not catch it

The ACK is taken **once**, immediately after launch, and I recorded it as the
model for the action. A change after that point was invisible: every receipt
would have said astra high, and I would have believed it. That is the defect, and
it is mine — an attestation sampled once is not an attestation about a session,
it is an attestation about a moment.

Worse, this is quiet in exactly the wrong direction. A downgrade produces work
that still looks like work. Had this session actually produced a diff, I would
have reviewed and possibly received it believing it came from the model the
allocation froze.

## The fix

`modelcheck.sh` reads every `"model"` value out of a worker's session rollout and
fails if any differs from the model the allocation froze. It runs on the
completed session, so it covers the whole run rather than its first second. Both
sessions were checked immediately:

| Worker | Models seen | Verdict |
| --- | --- | --- |
| `pf83d15` | `gpt-6-astra` | OK |
| `pf83d17` | `gpt-6-astra`, `gpt-5.6-luna` | **DRIFT** |

The drifted action is failed, not accepted. It produced nothing, so nothing has
to be unwound, which is luck rather than design.

## What needs Travis

The tooling now detects drift; it cannot prevent it. While the account is rate
limited, any long worker may be offered the same downgrade, and the honest
options are his to choose between: accept Luna for some classes of work and say
which, raise the limit, or slow the dispatch rate so astra is reserved for units
that need it. My own reviews already run on Opus 5.0 High as he directed, so this
is specifically about the implementation workers.

Until he answers, I am not dispatching further heavy astra units. Detecting drift
after the fact is not the same as spending a quota we may not have, and burning
the remaining astra budget on units I then have to fail is the worst of both.
