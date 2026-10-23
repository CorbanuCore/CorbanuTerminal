# Intent packet: accounting follow-up live checks (PF-60 follow-ups, October 2026)

Written by the integrator from the accounting sprint records (their mandates, acceptance criteria and the owner's
recorded decisions) and the owner's decisions for this run. It contains no source code, tests, implementation notes or
test results. Screens are in `screens/` (see `screens/README.md`).

## The product and the feature

Corbanu Terminal is a terminal coding agent (a text UI: the person types messages, the model answers and may run
commands). Developer builds have a `/cost` command: a set of pages showing every paid request the product made on the
person's behalf, under the conversation it belongs to, with tokens and a price. Pages:

- **First screen** ("Cost — this conversation"): today's totals per provider/model for this conversation, lines for
  special kinds of request, totals for other conversations, then links (`→ …`) to provider groups and to each request.
  The page is taller than the terminal: ↑/↓ scroll it.
- **Request page** ("Request"): provider, model, billing, cost, tokens, the kind of request, and in its Details the
  request's identifier (`Request: <id>`) and a link to its technical details.
- **Technical page** ("Attempt, components and original price"): the attempt's identifiers (`Request:`, `Attempt:`,
  `Thread:` and a `Turn:` line whose prefix says what kind of work it was), each token component with its rate and
  cost, the billing basis and the price basis.
- Navigation: ↑/↓ to move, Enter to open a link, Esc to go back or close. `/cost YYYY-MM-DD` shows another UTC day.

Billing kinds: **pay per use** (an API key; priced from the provider's published rates), and **no price** (no
published price covers it: no money figure is shown and the page says why).

## Rules that always hold (binding)

1. Unknown stays unknown, never zero: a missing count or price is shown as unknown / "no price", not $0.
2. Nothing is counted twice; every request appears once, under the right conversation, with one Request ID.
3. History is never re-priced: requests already recorded keep their cost and their Request ID when the conversation is
   reopened or more work happens.
4. No credential is ever printed.

## What this run must verify live

This is a short live run of five behaviours of the accounting follow-ups. Design cases only for these (plus the basic
sanity checks they need).

### 1. Code review (`/review`) is attributed to the conversation that ran it

- `/review` starts a code-review task (for example on the workspace's uncommitted changes). Its requests are recorded
  **under the conversation where the person typed `/review`**, labelled as code review (`Turn:` starts with
  `code-review:`), each once, and included in that conversation's total on `/cost`'s first screen.
- Nothing from the review may end up outside the total, e.g. in "Unknown parent population" (that line should keep
  reading 0 attempts), or under another conversation.
- A review made inside `/side` would be labelled `side:code-review:` (if the text UI allows `/review` there at all).

### 2. Side conversations are labelled

- `/side` opens a temporary side conversation on a saved conversation. Every request made in it is recorded under the
  saved (parent) conversation, labelled as side work: its technical page's `Turn:` line starts with `side:`.

### 3. Reopen warm-up and Request-ID stability (OpenAI)

On providers that use a WebSocket connection (the OpenAI route), reopening a conversation — restarting the product on
it (`resume --last` / `resume <id>`), `/resume`, `/side`, or editing an earlier message — sends one warm-up request
(billed like any request; typically 10,000–15,000 input tokens). Owner's choice: show it, and offer a setting to skip
it (off by default). The product must:

- label it a reopen warm-up: `/cost`'s first screen says `Reopen warm-up: N request(s), <cost>`; its request link and
  request page say "Reopen warm-up";
- once the warm-up has actually been sent, show a notice in the session naming the `skip_reopen_warmup` setting;
- with `skip_reopen_warmup = true` (config or `-c skip_reopen_warmup=true` at start), send no warm-up on reopen;
- leave every earlier request unchanged: **same Request ID, same cost, listed once**; nothing hidden or counted twice.
  (Request IDs are shown on request pages and technical pages; this run must actually re-read them after reopens.)

### 4. Moonshot cache read (owner decision D3)

A provider the person configures themselves at `https://api.moonshot.ai/v1` (Kimi Open Platform, Chat Completions) is
priced from Moonshot's published rates, whatever the person named the provider (USD per million tokens):

| Model | Input (cache miss) | Cached input (hit) | Cache write | Output |
| --- | --- | --- | --- | --- |
| `kimi-k3` | 3.00 | 0.30 | 3.00 (5-minute lifetime) | 15.00 |
| `kimi-k2.7-code` | 0.95 | 0.19 | priced as input (0.95) | 4.00 |

Moonshot's rule: cached + written + uncached input = the reported input total; when its report leaves out a cache count
(it omits counts that are zero), the price must still be complete, not a lower bound. A first turn on a model writes
the conversation's prefix to Moonshot's cache; **a second turn on the same conversation within the cache lifetime
should read it**. The owner asked for one real second turn per Kimi model to see a real cache read priced at the
cached rate. Whether this part may run at all, and how many Moonshot turns, depends on the owner's spend answer, which
the tester's run brief will state (see the environment below).

### 5. Realtime voice/text sessions on an API key (owner decision D16)

The product can open a realtime session with OpenAI (an app-server feature; WebSocket connection, the default). A
realtime session paid on an API key must be recorded as **one request** in `/cost`, under the conversation it belongs
to, labelled a realtime session, with its start and end times; its token counts and cost are **unknown, not zero**
(the protocol doesn't report its usage to this client): no `$` figure anywhere for it, and the first screen says
`Realtime: N session(s); token counts and cost unknown, not zero`. See `screens/realtime-01-...` (a test render).

## Owner decisions that bind the tests

- When a model is offered over several connections, the OpenAI-style connection is the default.
- Reopen warm-up: shown in `/cost` and on reopen, with a skip setting off by default.
- D3: a real Moonshot cache read, small spend (see above). D16: one real realtime session, under $0.05.

## The test environment (what the tester will have)

- A macOS test machine (a fresh virtual machine). The tester drives the product only through a terminal driver
  (`agms-tui`: start the product, type text, press keys, read the screen, wait for text, stop). Each case group starts
  with a fresh private product profile and a small git project as the workspace (it has tracked files the tester may
  edit); profile and project persist within the group (stop/start = restart on the same profile; product arguments
  such as `resume --last`, `resume <id>` or `-c key=value` can be given at start).
- The tester can also run `corbanu exec` (one-shot) on the same profile and project.
- Connections ("routes"):

  | Route | What it is | Models |
  | --- | --- | --- |
  | `zai` | Z.AI pay-per-use API, **real** (the default; cheap) | `glm-5.3-flash` |
  | `openai` | OpenAI API key, **real** (small spend; WebSocket connection, so reopen warm-ups happen here) | `gpt-6-luna` (cheapest) |
  | `moonshot` | Moonshot Kimi Open Platform, **real**, configured as a person would (a provider named `kimi` at `https://api.moonshot.ai/v1`, Chat Completions) | `kimi-k3`, `kimi-k2.7-code`; turn limits per the owner's spend answer |

- **Realtime:** a driver command runs one realtime session on the OpenAI route's API key through the product's app
  server: it starts the session (text output), sends one short line as speech input (e.g. "Reply with the single word
  OK."), waits for the reply, stops the session, and prints the reply and the conversation (thread) ID it used. At most
  one session is the plan; the harness refuses a third connection. Whether that conversation can then be opened in the
  text UI (`resume <id>`) is not known in advance.
- Real routes report real usage; the tester recomputes from the tokens on the technical page and the rates above.
- The product's model for ordinary work is GLM 5.3 Flash; the same model family also drives the tester.
- The tester cannot read the product's internal files or databases, only screens, the workspace files and the
  product's own output.
- Spend: keep real turns short ("Reply with the single word OK." style); total OpenAI spend well under $1; Moonshot and
  realtime per the owner limits above.

## Out of scope / named limits

Everything else in the accounting feature (other providers, prices, stand-ins), TensorCash, WebRTC realtime, Windows
and Linux, re-pricing history.
