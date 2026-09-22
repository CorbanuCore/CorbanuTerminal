# Every client that can spend the operator's money, named

*PF-60 S03, 21 September 2026. Author: Fable. Branch
`bootstrap/acct-activation-20260916`, commits `8d7a66321..d1950590d`.*

The goal of this workstream is that accounting is available on all models and
providers. Up to here that claim rested on the clients I happened to have
found. This is the audit that tests it, and the two gaps it turned up.

## What the audit did

Every client in the workspace that can send a model request, enumerated three
ways: the API client types in `codex-api` and their construction sites; every
raw HTTP post to an inference-shaped path; and every crate that holds a
provider credential. Each one is named below as collected, not collected, or
not inference. An independent reviewer ran the same audit separately.

## Collected

| Client | Where |
|---|---|
| Responses, streaming and WebSocket | `core/src/client.rs` |
| Chat Completions | `core/src/client.rs` |
| Anthropic Messages | `core/src/client.rs` |
| Compaction: local, remote v2, legacy `/responses/compact` | `core/src/client.rs` |
| Realtime call creation | `core/src/client.rs` |
| Memory stage one | `core/src/memory_stage_one.rs` |
| **`memories/trace_summarize`** | `core/src/client.rs` — **this range** |
| Image generation and edits | `ext/image-generation` |
| Web search | `ext/web-search` |
| Startup prewarm, turn classifier, agent-identity sessions | `core` |
| Pane bridge: Ambient chat | `tui/src/claude_panes/bridge.rs` |
| Pane bridge: Anthropic passthrough, including the Vercel gateway | `tui/src/claude_panes/bridge.rs` |
| **Direct pane profiles: z.ai, Baseten, OpenRouter** | `tui/src/app/background_requests.rs` — **this range** |
| Rented GPU endpoints (`gpu-*` providers) | the ordinary turn path |

Rented GPUs deserve a note: a rental becomes a per-turn `model_provider_id`,
so its turns collect like any other provider's. They are billed by the hour
rather than by the token and are not in the catalogue, so tokens are recorded
and **no money is claimed** - which is the honest answer for an hourly rental.

## The two gaps, and what closed them

**`memories/trace_summarize`** names a model and a reasoning effort, so it is
inference the operator pays for, and it answers with one JSON body rather than
a stream - exactly how the legacy compaction endpoint escaped collection.
Nothing had reached it. It has no production caller yet, which is the argument
for doing it now: whoever wires stage two would otherwise ship an invisible
cost. The accounting slot is a parameter, so the call cannot be made without
deciding what records it.

**The three direct pane profiles** - z.ai, Baseten and OpenRouter on their
Anthropic-compatible routes - use no bridge. They hand Claude Code the
provider's base URL and a vault credential and let it talk to the provider
itself, so this process never sees the requests: there is nothing to wrap and
nothing to report send by send. What does come back is the pane's own report
of what the turn cost, which is the provider's numbers relayed through Claude
Code. A direct turn is now recorded once from that, against the provider's own
route, through the same server-side check a bridged send passes.

Three things about that are stated rather than implied:

- **One record per turn, not per request.** A pane turn can be many model
  requests and this client saw none of them individually, so the attempt count
  for these profiles counts turns. The numbers are the turn's.
- **The numbers are the turn's total, not its first request's.** Review caught
  me recording the wrong field: the pane's display summary is the first usage
  a transcript carries, so a turn that ran a tool loop would have been recorded
  with one request's tokens in a turn-shaped row, undercounting every aggregate
  a reader computes. The total the pane states when the turn ends is a separate
  field now, and that is what the ledger takes. The per-turn audit file states
  both, each named.
- **A turn that states no total is not recorded.** Unlike a send, there is no
  request here whose existence is itself the fact worth recording - only the
  numbers - and summing per-request usage into a total would be this client
  inventing one. An interrupted turn that did state a total is recorded, and
  that total is salvaged even from a transcript that no longer parses.

A bridged profile must never take this path, or the same spend is counted
twice. A test pins both halves.

## Also closed in this range

**Streamed pane turns now record real numbers.** Claude Code streams, so
"tokens unknown for streamed Anthropic responses" meant the ledger had no
numbers for the lane that is actually used. A streamed Messages response
states its input and cache counts once with `message_start` and restates a
running output total with each `message_delta`; the bridge already buffers the
whole response, so all of it was in hand. The reading lives beside the event
shapes in `codex-api`, and the last statement of a field wins because the
provider is restating a total.

## Not inference, and why

- **Model listing** (`model-provider`, `app-server` refresh worker) - a
  catalogue read, no tokens.
- **The Responses WebSocket doctor probe** (`cli/src/doctor.rs`) - a handshake
  that is closed immediately, with no request.
- **LM Studio and Ollama model loading** - local model management on the
  operator's own machine; no account is billed.
- **GPU rental management** (Vast.ai, RunPod) - provisioning, billed by time.
- **Cloud task creation** (`backend-client`) - spend on the service side, not
  a model request from this client.

The one judgement call here is the **GPU readiness probe**
(`gpu-market/src/readiness.rs`), which does post a real `chat/completions` to a
rented endpoint. It is provisioning validation on an endpoint billed by the
hour, before that endpoint is a provider at all, so it costs no per-token
money. It is named rather than hidden.

## Still not collected, by construction

Two paths spend the operator's credential from outside this process entirely,
and neither can be reached by wrapping a transport:

- **`responses-api-proxy`** - a standalone binary that forwards `/v1/responses`
  with the operator's key for other tools.
- **Credential brokering to child processes** (`network-proxy`) - a child given
  a credential can spend it.

Both would need their own reporting boundary, the way the panes bridge now
has one. They are named here so they are a decision rather than an oversight.

And the standing one: **collection is impossible in any build a user could
receive**. Non-default feature, compile error in optimised builds, byte marker,
package builder refusal. That gate belongs to the security workstream.

## One more thing review found

Reporting ran from the turn's `Result`, and every outcome branch ends in a
fallible audit write, so a full disk would have turned a real charge into an
error the dispatcher skipped. The report is now sent where the output is
built, before anything that can fail.

## Verification

Clean-host lanes on the RTX workstation, fmt-clean, `--offline`. The lanes now
include a `pf_60_s03` filter: the previous filter was the word `accounting`,
which never matched those tests, so they only ran when I ran them by hand -
and a mutation I had reverted locally came back when I pulled formatted
sources off the verifier and landed in a commit. Independent review caught it
in the same round the new lane did.
