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
let transport = accounting.transport(transport, model, "images/generations", "image").await;
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
- The provider passed in is the caller's **live** one, not a snapshot taken when
  the handle was made. `turn_mode` then binds the route the request actually
  takes, exactly as it does for a turn, so a session whose provider changed
  records against the new route rather than a stale one. Review caught the
  snapshot; it would have bound collection to a provider the request no longer
  used.
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

Both tests exercise the seam directly rather than through `backend.rs`, so the
edits path's pin and the no-handle branch are covered by the type system and by
reading, not by a test. That is stated rather than implied.

Clean-host lanes, RTX workstation, fmt-clean: `codex-core` accounting with
`developer-accounting` **150/150**, the image-generation extension 10/10.

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
