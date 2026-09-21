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
failing on each run. The stage-one suite passes 24/24, logged as
`rtx-20260921/head-memory-stage-one.txt`. Raw logs under `rtx-20260921/`.

One pre-existing flake worth naming because I chased it: `session::tests::
non_steerable_turn_defers_user_input_until_completion` fails most runs on this
host at this tree, at the integration tip, and at the older integration base
alike, in isolation as well as under load. It is not caused by this work; I
attributed it by running the same test at all three trees.

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
variable. Eight consecutive full-lane runs were clean; the repeat summaries are retained
in `rtx-20260921/classifier-repeats.txt`, alongside the failing run the fix
addresses.

## The fifth call site: stage-one memory extraction

`memory_stage_one::StageOneMemoryClient` builds its own `ModelClient` and
streams a real billable request. It is reachable in production - `app-server`'s
turn processor starts the memories task, which reaches it through
`memories/write`'s phase-one runtime - and it is gated: `Feature::MemoryTool` is
default-off, and the pipeline is skipped for ephemeral and non-root sessions.
The feature is Stable and the TUI offers to turn it on, so an operator who
enables memories was running extraction that reached no ledger.

It now collects, under a `memory:` turn of its own, with two properties this
path needs and the others did not:

- **Collection inputs are read once, at admission.** The accounting mode and
  provider id are captured in `StageOneMemoryClient::new`, from the same policy
  read that admits the client, rather than re-read per request. This is one
  read instead of one per extraction; it is not protection from lock
  contention, since `check_completion` already takes the session lock on entry
  and on every stream event. It cannot go stale unnoticed: the client is built
  per pipeline run, `config.model_provider` is forced to the admitted provider,
  and a provider or policy change denies the binding before anything records.
- **It collects only on the route the admitted configuration approved.** If the
  client's provider is not the one the binding validated, the extraction runs
  and records nothing, rather than failing the route check at admission.
  Best effort, and the same honest limit as everywhere else in this workstream:
  it covers **attach** time. Once scopes are held, an accounting fault inside
  the request - a poisoned slot, a route or mode disagreement, a failed
  admission - fails the extraction, exactly as it fails an ordinary turn, and
  the request runs through the accounting transport rather than the plain one.
  So an extraction can now fail for accounting reasons where before it had no
  collector and could not. The denial contract itself is untouched: the
  guarded transport still carries the binding, `check_completion` still runs on
  entry and on every event, and only bounded metadata - turn label, provider,
  model, token counts - reaches the ledger.

`pf_60_s03_stage_one_extraction_records_its_own_turn` drives a real extraction
against a mock at the session's own configured route and asserts the `memory:`
row; dropping the scopes instead of holding them fails it. The existing
stage-one security fixtures, which substitute a socket endpoint through a
private hook, keep passing precisely because of the route rule above.

## What this does not claim

- Auxiliary inference that opens its own client session: local compaction,
  remote compaction, startup prewarm, the completion classifier and stage-one
  memory extraction. All five attach, the fifth as described above.
- The legacy `/responses/compact` endpoint is still uninstrumented; it is
  reachable only by disabling `remote_compaction_v2`, which is Stable and
  default-on, and it posts through `ApiCompactClient`, which has no collector
  seam.
- Off a provider's own route, economics are still not claimed. Collection is
  unaffected: those turns record tokens.

With this increment the "what does not collect" list holds no session class
that ordinary use reaches, and none that a non-default but Stable configuration
reaches either.
