# Image generation: the default-on client that recorded nothing

*PF-60 S03, 21 September 2026. Author: Fable. Branch
`bootstrap/acct-activation-20260916`.*

## What was wrong

Independent review, checking a completeness claim I had made, found that the
conversational model path was not the whole product. `ext/image-generation`
builds its own `ImagesClient` on a bare transport with the session's provider
and auth and posts `/v1/images/generations` and `/v1/images/edits`.
`Feature::ImageGeneration` is Stable and **default-on**. So in a default
configuration, an operator generating images was paying for model inference that
reached no ledger at all - no attempt, no tokens, no money.

It had no collector for a structural reason, not an oversight: an extension owns
its client. There is no `ModelClientSession` to attach to and no `TurnContext`
to read, so every seam this workstream had built was unreachable from there.

## The seam

`codex_core::accounting_extensions::ExtensionAccounting` is a host handle,
inserted into the thread's extension store once the session exists and holding
it **weakly**. An extension wraps its own transport with it:

```rust
let transport = accounting
    .transport(transport, provider, endpoint, model, "images/generations", "image")
    .await;
```

- The recorded turn is `image:<uuid>`. An extension's requests are their own
  units of work, not part of whatever turn happened to trigger them.
- The request is pinned to that path under the provider's approved route, using
  the same path-override the legacy compaction endpoint needed. A request to
  anywhere else is refused at admission rather than attributed here.
- Collection is best effort in the now-usual sense: accounting off, owner gone,
  or a route that collects nothing all return the transport unwrapped and the
  request proceeds unrecorded. An older host that supplies no handle gets
  `ExtensionAccounting::unrecorded`, which is the same passthrough.
- The numbers come from the response the provider already sends: the images API
  returns `usage` in the same shape the Responses body parser reads, so tokens
  are recorded rather than left unknown.
- **Nothing is snapshotted.** The handle holds only the session, weakly. The
  accounting mode and provider identity are read from the owner at call time,
  and the provider and endpoint are the caller's own - the very ones the request
  will use. Review caught two versions of this: first a provider snapshot, then
  a provider identity still frozen at session start beside a live provider, and
  an endpoint resolved here through a different auth source than the caller's,
  which could pin a route the request never takes. All three are gone.
- One thing does change about the request, and it is the same limit as
  everywhere else: once evidence exists, an accounting fault - a failed
  admission, a route the transport refuses, a failed observation write - fails
  the image request, which before could only fail for its own reasons. Best
  effort covers getting attached, not staying attached.

This works because `AccountingTransport::execute` learned to collect in the
previous increment, for the legacy compaction endpoint. Image generation is the
second caller of that path and the first outside `codex-core`.

## Verification

`accounting_extension_client_records_its_own_request` builds a real session with
a real state runtime, wraps a real `ImagesClient` through the handle, drives a
generation against a mock images endpoint, and asserts one `image:` attempt for
model `gpt-image-1` with its usage observed. Returning no evidence from the
handle - which is what the defect looked like - fails it.
`accounting_extension_client_without_its_session_records_nothing` drops the
session first and asserts the request still succeeds with no accounting
installed at all.

`accounting_extension_client_binds_the_route_it_actually_sends_to` pins the
property review caught twice and no test covered: the session is configured for
one route while the extension's client sends to another, and the request must
record against the route it actually takes. Re-resolving the endpoint from the
session instead of taking the caller's fails it with an admission error - which
is what the snapshot defect would have done to a real image request.

All three tests exercise the seam directly rather than through `backend.rs`, so
the edits path's pin and the no-handle branch are covered by the type system and
by reading, not by a test. That is stated rather than implied.

Clean-host lanes, RTX workstation, fmt-clean: `codex-core` accounting with
`developer-accounting` **151/151**, the image-generation extension 10/10.

## What is still uncollected

Two of the four clients review named remain, and both are narrower than image
generation was:

- **Realtime calls.** `ModelClient::create_realtime_call_with_headers` posts
  through the plain transport. `realtime_conversation` is under development.
- **The Claude panes bridge.** `tui/src/claude_panes` posts chat completions to
  Ambient on a bare client with an operator credential from the vault, and
  selects an Anthropic OAuth passthrough for the same surface.

Both are named here rather than left to be discovered, and both are reachable:
the panes bridge is a shipped surface. They are the next increments.
