# The panes bridge, the realtime call, and the bodies that were refused

*PF-60 S03, 21 September 2026. Author: Fable. Branch
`bootstrap/acct-activation-20260916`, commits `ad219caf8..8d7a66321`.*

## What this closes

Three clients that spend the operator's money and reached no ledger:

- **Realtime call creation.** A model request like any other.
- **Web search.** Wired through `ExtensionAccounting` exactly as image
  generation is.
- **The Claude panes bridge.** The one I had stopped at, and the reason for
  most of this record.

## Two defects the realtime work found

Recording the realtime call did not work the first three times, and each
failure was a defect in the shared machinery rather than in the new caller.

**Multipart bodies were served unrecorded.** The attributability check read a
request body as a single JSON document and refused anything it could not read
as "uninspectable". A realtime call body is a multipart envelope - an SDP part
beside a session JSON part - so every call would have been excluded. Exempting
the route would have stopped inspecting routing keys on it entirely. Instead
the JSON parts are read and checked like any other body, and a part that
claims to be JSON and is not still refuses. The compressed-JSON path that an
earlier round had to fix is untouched: it is still tried first.

**The route pin was missing half the route.** Call creation appends the query
pairs that select its architecture, and the pin was the path alone, so
admission compared a pin against a URL that did not match and failed the call
outright - not silently, but fatally. The route now comes from the API client
that builds it rather than being restated in `codex-core`, and a pinned route
can carry query of its own, merged with the provider's configured query and
compared canonically.

Realtime token usage never reaches this client - the protocol carries none -
so the call records with its tokens unknown. That is what the provider said.

## The panes bridge: recorded through the app server

The bridge is a loopback HTTP server **inside the TUI** that posts a pane's
turns to Ambient, Anthropic, the Vercel gateway or z.ai on the operator's own
credential. `codex-tui` does not depend on `codex-core`, so the transport seam
the extensions use is unreachable from it. The two options inside the TUI were
to write the collection policy a second time or to write to `codex-state`
behind core's back. This is the third: **the bridge reports each send to the
app server**, over a new request `thread/recordSentModelRequest`, and the
handler holds the core session and uses the one implementation of the policy.

Two properties differ from the transport path, and both are stated in the code
rather than implied:

- The attempt is admitted **when the send is reported**, which is after it
  happened. A pane turn whose reporter dies mid-flight is not recorded at all,
  rather than recorded with its outcome unknown.
- The route is the one the caller says it used. There is no request here to
  check it against.

**No money is claimed.** The credential is the pane's, not one this session
can attribute to a catalogue account, so these turns record tokens and state
no economics.

## The boundary, after three rounds of review took it apart

My first version had a hardcoded table of two routes in the app server, and
independent review found three attribution defects in it:

1. The Anthropic passthrough handler serves **every** Anthropic-shaped
   upstream the pane profiles name, not only Anthropic. Reporting a hardcoded
   Anthropic route would have entered Vercel gateway spend in the ledger as
   Anthropic spend, on a route the request never took.
2. The OAuth lane also proxies **token counting**, which is free and is not
   inference. Those were reported as `messages` model requests, so a pane that
   counts tokens before each prompt would have filled the ledger with phantom
   attempts carrying no numbers.
3. The server recorded the **caller's own strings**, so one real route could
   land under several endpoint spellings and break grouping by route.

What replaced the table is better than the table. The client names the account
it believes it billed and reports the upstream it actually posted to; the
server agrees only when that provider is one this build ships, the reported
base URL is that provider's own API base URL, and the path is that provider's
dialect's default path. It then records **the catalogue's** strings. A
provider the build does not know, or a known provider on a route it does not
serve, is not recorded at all.

Round two then found that my repair had broken the thing it repaired, and that
I had described it dishonestly:

4. The inference gate compared the request target for **exact equality**, and
   the client this bridge serves sends `/v1/messages?beta=true`. Every real
   turn on both passthrough lanes would have been dropped - silently, because
   nothing logs a send that is never reported. The target is compared without
   its query now, which still excludes token counting, since
   `/v1/messages/count_tokens` differs before the query.
5. **Nothing drove the reporting path at all.** Both existing bridge tests
   pass no sender, so none of the new logic ran in any test. That is why I did
   not catch the defect above myself.
6. My comments claimed a **boundary the code does not provide**. It is a
   well-formedness check: the caller names the account, the accepted base URLs
   are public constants, and two providers sharing a route cannot be told
   apart. What it guarantees is that nothing lands under a route its named
   provider does not serve, and that the endpoint recorded is the catalogue's.
   Nothing monetary rides on the name. The comments say that now.

Round three found one thing, and it was the silent-drop failure mode again:
the agreement between a pane profile and the catalogue was pinned nowhere, so
an edit to either side would stop recording for that lane without failing
anything. It is a table test now.

## Verification

Clean-host lanes on the RTX workstation, fmt-clean, `--offline`:

- `codex-core` accounting **148/148**; with `developer-accounting` **153/153**
- `codex-state` **168/168**; `codex-tui` usage **92/92**
- `codex-tui` tokens 66 of 67, the one failure
  (`accounting_inspect_maintenance_with_healthy_raw_renders_lag`) pre-existing
  and verified at base
- `codex-http-client` **76/76**; `codex-app-server-protocol` **287/287**
  including the regenerated schema fixtures; `codex-tui` pane tests **99/99**

`pf_60_s03_realtime_call_is_recorded` drives a real call through the real
client against a mock endpoint and asserts one `realtime:` attempt. It fails
when the scopes are dropped, when the multipart inspection is reverted, and
when the pin loses the appended query - the last two are how the defects above
were found rather than argued.
`accounting_records_a_request_another_client_already_sent` records a reported
send on a different provider and route from the session's own, asserts the
provider, model, turn label and one observation, and asserts that a session
which is not collecting records nothing and says so.
`only_a_known_provider_s_own_route_is_recorded` pins the server's check,
including the two cases review named: a provider this build does not ship, and
a known provider on a route it does not serve.
`passthrough_bridge_reports_inference_and_not_token_counting` drives the real
bridge handler over a real socket against a real upstream and asserts the
report - provider, route, model, and the tokens the upstream stated - and that
token counting on the same lane reports nothing. Restoring the exact-equality
comparison fails it, which is how it was proved rather than assumed.
`every_bridge_profile_reports_its_provider_s_own_route` pins the agreement
across the crate boundary; pointing a profile at the wrong provider fails it.

The integration test I had written for the realtime suite was deleted rather
than kept: that whole suite cannot run on the verification host, so it proved
nothing.

## What is still uncollected, stated plainly

- **Realtime session tokens.** The protocol this client parses carries none
  anywhere. Only the call is recordable.
- **Streamed Anthropic pane turns.** Anthropic reports usage inside stream
  events; the bridge proxies bytes and this path never sees them, so those
  calls record with tokens unknown rather than as zero. A non-streamed
  response is parsed and recorded.
- **Pane spend economics.** Tokens only, by design, for the reason above.
- **Collection in any build a user could receive.** Still impossible, still
  gated on the security workstream.
