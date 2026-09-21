# The last two excluded classes: agent identity and startup prewarm

*PF-60 S03, 21 September 2026. Author: Fable. Branch
`bootstrap/acct-activation-20260916`.*

## What was still excluded

Every earlier increment in this workstream ended with the same two sentences in
its "what does not collect" section:

- **Agent-identity telemetry sessions.** When the client resolved agent-identity
  telemetry, Chat turns and WebSocket-capable Responses turns called
  `deferred.exclude()` before a collector existed, and
  `Provenance::capture` marked the route ineligible outright.
- **Startup prewarm and auxiliary inference.** `stream_responses_websocket`
  skipped reading the deferred sampling whenever `warmup` was set, so the
  prompt this client sends to prime the model was never recorded.

Both are now collected. Neither was a provider or model class, but both meant
that on **every** provider a whole class of paid inference recorded nothing.

## Agent identity is a credential, not a reason to stop counting

The exclusion is from the original API-key-only scope, where "custom command
auth, experimental bearer override, header auth, agent identity and subscription
backend are outside the new qualification" (`responses-dispatch-allocation.md`).
That scope has since been widened everywhere else: subscription turns collect,
header auth collects, command auth takes the plan side. Agent identity was the
last one refused, and refusing it at collection made the policy that already
names `AuthMode::AgentIdentity` as subscription capacity dead code - there was
nothing collected for those economics to apply to.

`Provenance::capture` no longer takes an agent-identity flag, and neither
streaming path excludes on it. Which economics such a turn admits is decided
once, in `turn_mode`, like every other credential.

`accounting_agent_identity_session_collects` builds a real
`CodexAuth::AgentIdentity` from a generated key with a pre-registered task id -
so construction never reaches the network - drives a real turn, and asserts the
attempt and its usage are recorded. Restoring the exclusion makes that test fail
with no accounting tables at all, which is what the defect looked like.

## Prewarm is inference the operator paid for

Startup prewarm sends the session's whole prompt with `generate: false` to prime
the model. The provider charges for that prompt. It now attaches collection
through the same `accounting::attach_turn` compaction uses, under its own
`prewarm:` turn identity, and is best effort in exactly the same way: a prewarm
that cannot be recorded still runs, because accounting is an observer here and
not a gate on starting a session.

Two consequences, both deliberate:

- A prewarm is a **separate turn** in the ledger, not part of the first real
  turn. An operator can see what priming cost.
- Its frame reports no cached-token or reasoning detail, so its uncached bucket
  is unknown and only its output side is priced. The day's totals therefore
  carry one more incomplete estimate. That is the truth about the evidence the
  provider sent, not a gap in this work.

The WebSocket accounting suites are updated rather than adjusted: they now read
the turn's own rows through `turn_attempts` and `turn_observations`, and where a
day's totals are the subject they state the prewarm's contribution explicitly
through `with_prewarm`, including its $0.02997 at the fixture's own rates. The
resume test stops asserting a day total it was never about, because a reopened
session starts its own prewarm and whether that has landed is a race; what it
still asserts is that the turn's own evidence and its bound prices survive two
reopens unchanged.

## What this does not claim

- Auxiliary inference other than startup prewarm: there is none. The only three
  call sites that open their own client session are local compaction, remote
  compaction and startup prewarm, and all three now attach.
- The legacy `/responses/compact` endpoint is still uninstrumented; it is
  reachable only by disabling `remote_compaction_v2`, which is Stable and
  default-on, and it posts through `ApiCompactClient`, which has no collector
  seam.
- Off a provider's own route, economics are still not claimed. Collection is
  unaffected: those turns record tokens.

With this increment the "what does not collect" list holds no session class
that ordinary use reaches.
