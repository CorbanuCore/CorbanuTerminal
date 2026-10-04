# Smallest next step: collect startup WebSocket prewarm explicitly

**Recommendation: startup prewarm on the already admitted direct OpenAI
Responses WebSocket path.** This is a proposed product-initiative slice for
Fable to allocate, not an implementation mandate or a qualified inherited item.
It addresses one known loss point and reuses the working native WS fixture and
usage observer. Gateway accounting also needs a gateway economic authority;
legacy acquisition needs an original corpus and provenance adapter; auxiliary
collection needs a new endpoint/operation population and response adapter.
Prewarm already sends a distinguishable `generate=false` request and receives
a completion in the existing transport. That makes it the cheapest evidenced
engineering path here, not a measured time/line estimate.

The current blocker is specific: `client.rs:3447` forces `deferred=None`
for warmup, and `prewarm_websocket:3727` drains completion without collecting.
`session_startup_prewarm.rs:305–324` already has the owning session/startup
context and marks transport metadata `CodexResponsesRequestKind::Prewarm`.
The existing fixture sends input/output/total 999/999/1998, whereas
`accounting_responses_ws_native_prewarm_preconnect_and_cached_reuse` requires
zero warmup observations and then one sampling observation with input 100.
These are source facts, not a claim that a real provider bills warmup.
[Exact source excerpts and hashes](next-step-source.json) bind this proposal
to base 270a6644e01d780ef93f8715dd0051481643dc7d.

## Product work required before a positive collection run

1. **Define startup ownership and operation identity.** Extend the typed
   accounting attempt contract with a distinct startup-prewarm operation;
   retain the existing thread/root, scope, request UUID, physical attempt UUID,
   retry link, admitted provider/model and dispatch UTC timestamp. Give startup
   a stable lifecycle identity (for example `startup:<UUID>`) distinct from
   a prompted user turn. Generate a new lifecycle for an actual fresh startup,
   not on reader refresh. An unsent setup/skipped prewarm must not create an
   attempt; each actual retransmission must have a separate linked attempt.
   The current `Attempt` has a required nonempty `turn` and no operation kind.
   Do not silently pretend startup is its first prompted turn.
2. **Connect the startup caller to the admitted WS observer.** In
   `session_startup_prewarm.rs`, `client.rs`, `accounting_responses.rs` and
   `accounting_websocket.rs`, attach an independent startup collection scope,
   validate the same approved endpoint and credential provenance, and admit
   immediately before the physical `generate=false` send. Route its raw usage
   observations through the native WS accounting observer before the completion
   drain. Preserve missing/null/number/zero independently, event source/sequence,
   completion/failure and replay identity. Clear the scope on success, failure
   and cancellation so cached-connection sampling cannot inherit warmup usage
   or identity. Do not merely remove the `warmup` exclusion: it currently has
   no correctly bound startup collector. Preserve existing no-install/default-OFF
   behavior and provider eligibility; an enabled recording failure must not
   silently produce a clean zero.
3. **Persist and inspect the distinction.** Adapt the state attempt
   serialization/reader projection and, if required by the storage encoding,
   version or migrate the persisted representation. Existing payloads use
   `deny_unknown_fields`; compatibility must be handled explicitly. Preserve
   old evidence unchanged and use an explicit unknown operation for historical
   records lacking provenance, rather than backfilling a guessed operation.
   Extend the existing root/day request and attempt UI with “Startup prewarm”
   and its lifecycle/attempt identity, reported metrics, status and economic
   unknowns. Account for this contribution exactly once in root/day totals
   and expose it separately from prompted sampling. Any unpriced prewarm makes
   the full estimate incomplete even when sampling has a known subtotal.
   Retention/reopen must not erase or misclassify the operation.

**Economic contract for the first slice:** warmup is connection setup with
`generate=false`, so a direct sampling tariff is not automatically applicable.
Store reported tokens and original field presence, but record price applicability
as unknown unless an authoritative source explicitly covers this operation.
Proposed visible text: “Startup prewarm”; “Reported input: 999”;
“Reported output: 999”; “Cost unknown — no applicable prewarm price evidence”;
“Billed cost: unavailable — no settlement evidence”.
999/999/1998 are fixture expectations only. They prove neither 1998 billed
tokens nor zero cost. No allowance, settlement or sampling-price substitution.
A later priced slice requires an original prewarm-specific economic source bound
to provider/account scope, model, dispatch time, currency, units and attempt;
that missing source is a real remaining prerequisite.

## QA work and first qualifiable slice

The first qualifiable claim would be: **one startup prewarm population is
collected, attributed, retained and inspectable, with reported token presence
and explicitly unknown cost, on direct OpenAI Responses WS.** It would remove
the deliberate-noncollection blocker for this path. It would not close the
whole inherited startup item, its real economic qualification, or complete
application coverage. Fable must record/accept that bounded scope in the active
PF-60 plan and an executable single-feature sprint before product edits.

QA can reuse `accounting_responses_ws_support.rs`'s warmup branch and cached
reuse test, but must adapt their assertions and add fixtures for:

- Fresh startup: exactly one sent prewarm, input/output/total 999/999/1998,
  distinct request/attempt/lifecycle; independently reconcile captured raw
  completion, persisted presence and the actual inspector. Missing cache fields
  remain missing; do not derive a fully known noncached amount.
- Then one sampling request on the reused connection: a separate attempt with
  its own 100-input fixture, no 999-token leakage or price inheritance.
  Warmup and sampling partitions each reconcile; root/day includes each once.
- Absent/null/zero usage, failure before versus after send, cancellation after
  admission, duplicate completion, retry with a new physical attempt,
  and disabled/skipped warmup. Unknown is not a measured zero; no-send has no
  attempt. Repeated events, refresh and reopen cannot double-count.
- Resume the same root, start a different root and cross a UTC admission-day
  boundary; verify ownership, startup lifecycle, time bucket, operation kind
  and unknown-cost warning persist through reread and retention. Check old
  stored payload compatibility and default-OFF/no-store parity.

Product work is the three units above (identity/persistence contract, native
collection plumbing, reader projection). QA work is fixture adaptation,
independent arithmetic/presence checks, compatibility/exclusion regressions and
packaged real-key evidence. This is not achievable by adding QA emissions or
receipts alone; changing Rust test expectations before the collector exists
would only hide the blocker. No product or Rust test change occurs in round 66.

Before a human-test handoff, freeze a fresh-context independent functional
design and use a separate code-blind executor with enforced package/filesystem,
process/IPC, network and credential boundaries, raw keys and positive/negative
access controls, then independent evidence review. Run guarded affected state,
API, Core default/feature and TUI tests after formatting only changed files.
The first packaged workflow must cover fresh startup, failure/recovery and
resume. The implementer's integration fixtures are supporting evidence only.
Fable retains platform/live-repository applicability, scope acceptance and
remaining PF-60-S03 reconciliation. No new human acceptance, waiver or release
authorization is supplied by this proposal.
