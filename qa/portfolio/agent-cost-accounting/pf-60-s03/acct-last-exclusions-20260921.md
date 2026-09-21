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

## Verification

Clean-host lanes at this tree, RTX workstation, formatted and fmt-clean:
`codex-core` accounting **141/141**, with `developer-accounting` **146/146**,
`codex-state` accounting **168/168**, `codex-tui` usage **92/92**, `codex-tui`
tokens 66 of 67 - the one failure is the stale snapshot that fails identically
at the integration tip. The `websocket` suite passes 69/69 and the `compact`
suite passes with retries; with retries disabled the compaction suite is
load-flaky at this tree and at the integration tip alike, with different tests
failing on each run. Raw logs under `rtx-20260921/`.

## A third class review found: the turn-completion classifier

Review read the claim that startup prewarm was the last auxiliary inference and
found it false. `session/turn.rs` builds a **fresh** client session for
`assess_turn_completion` - deliberately, so the classifier does not queue behind
the turn's transport teardown - and that session had no collector. The
classifier is a real billable request, and it is reached in ordinary use: the
built-in Kimi Code provider's stop is `AmbiguousForActionTurns`, so every
text-stopped action turn on that provider ran a classifier call that was paid
for and recorded nowhere.

It now attaches through `accounting::attach_turn` under an `assess:<turn id>`
identity - the classifier belongs to the turn it assesses - and is best effort
in the same way.
`accounting_chat_completion_assessment_collects` drives a real Kimi-shaped turn
and asserts the `assess:` row; dropping the scopes instead of holding them fails
it.

**The flake that test had, and what it actually was.** As first written the test
failed in 2 of roughly 8 full-lane runs and never in isolation, and I recorded a
guess - that the assessment's scopes closed before its stream admitted. Review
refuted it from the code: `AccountingTransport::stream` commits the attempt row
*before* handing the request to the inner transport, and a scope drop clears the
slot without rejecting an already-resolved sampling, so there is no window of
that kind to lose. The guess would have sent the next increment after the wrong
thing.

The actual cause was the fixture, as review also suggested: the shared Chat
support sets a two-second stream idle timeout with no retries, so on a loaded
host the turn itself could end before the ambiguous-stop branch was reached. The
test now waits for the classifier identified by **its own instructions** rather
than by counting requests - a second POST from some other path is not the thing
under test - and raises the idle timeout for this case so the turn is not the
variable. Eight consecutive full-lane runs are clean.

## What this does not claim

- Auxiliary inference that opens its own client session: local compaction,
  remote compaction, startup prewarm and the completion classifier all attach.
  There is a fifth, **stage-one memory extraction**, which builds its own
  `ModelClient` and streams a real request with no collector. It is not
  attached here and it is not reachable in ordinary use: `Feature::MemoryTool`
  is default-off and `StageOneMemorySession` has no production caller in this
  tree - only its own tests. Naming it is the point; the earlier version of this
  sentence claimed four call sites and was wrong.
- The legacy `/responses/compact` endpoint is still uninstrumented; it is
  reachable only by disabling `remote_compaction_v2`, which is Stable and
  default-on, and it posts through `ApiCompactClient`, which has no collector
  seam.
- Off a provider's own route, economics are still not claimed. Collection is
  unaffected: those turns record tokens.

With this increment the "what does not collect" list holds no session class
that ordinary use reaches, with the classifier flake above as the one open
question about how reliably the newest of them is recorded.
