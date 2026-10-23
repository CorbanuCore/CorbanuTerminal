# Intent packet: accounting follow-ups (PF-60-S06 to S09)

Written by the integrator from the four sprint records (their mandates, acceptance criteria and the owner's recorded
decisions). It contains no source code, tests, implementation notes or test results. Screens are in `screens/`
(see `screens/README.md`).

## The product and the feature

Corbanu Terminal is a terminal coding agent (a text UI: the person types messages, the model answers and may run
commands). Developer builds have a `/cost` command: a set of pages showing every paid request the product made on the
person's behalf, under the conversation it belongs to, with tokens and a price. Pages:

- **First screen** ("Cost — this conversation"): today's totals per provider/model for this conversation, lines for
  special kinds of request, totals for other conversations, then links (`→ …`) to provider groups and to each request.
- **Request page** ("Request"): provider, model, billing, cost, tokens, the kind of request, and a link to its
  technical details.
- **Technical page** ("Attempt, components and original price"): the attempt's identifiers (including a `Turn:` line
  whose prefix says what kind of work it was), each token component with its rate and cost, the billing basis and the
  price basis.
- Navigation: ↑/↓ to move, Enter to open a link, Esc to go back or close. `/cost YYYY-MM-DD` shows another UTC day.

Billing kinds the product distinguishes: **pay per use** (an API key; priced from the provider's published rates, or
from the charge the provider itself reports), **subscription** (a ChatGPT sign-in: "Covered by your subscription
(not billed per request)", never counted as spend), and **no price** (no published price covers it: no money figure
is shown and the page says why).

## Rules that always hold (from the original accounting feature, still binding)

1. Unknown stays unknown, never zero: a missing count or price is shown as unknown / "no price", not $0.
2. Subscription work is never shown as spend.
3. Nothing is counted twice; every request appears once, under the right conversation.
4. History is never re-priced: totals already shown don't change when the conversation is reopened or more work
   happens.
5. No credential is ever printed.

## What the four sprints are meant to deliver (the behaviour under test)

### S06 — side conversations and temporary forks are attributed (#400, #401)

- `/side` opens a temporary side conversation on a saved conversation. Every request made in it is recorded under the
  saved (parent) conversation, labelled as side work (`Turn:` starts with `side:`).
- If an approval request inside `/side` is sent to the automatic reviewer (approvals routed to the "auto-review"
  reviewer instead of the person), the reviewer's request is recorded **once**, under the saved conversation, labelled
  `side:review:`.
- A sub-agent started from inside `/side` is recorded under the saved conversation, labelled `side:agent:`. (`/side`
  is meant for quick questions and tells the model that sub-agents are off-limits there, so a model may decline.)
- `side:` is used only for the text UI's `/side`. Any other temporary fork of a saved conversation is labelled
  `fork:`. Sub-agents and `/review` code-review tasks that have no conversation of their own are labelled `agent:` and
  `code-review:`.
- `corbanu exec --ephemeral` runs (one-shot, not saved) and work done for them stay out of the record entirely.
- No change to how older records read.

### S07 — prices (#368, #786)

- **Moonshot (Kimi Open Platform).** A provider the person configures themselves at `https://api.moonshot.ai/v1` is
  priced from Moonshot's published rates, whatever the person named the provider. Rates (USD per million tokens,
  read 2026-10-14 and re-read 2026-10-15):

  | Model | Input (cache miss) | Cached input (hit) | Cache write | Output |
  | --- | --- | --- | --- | --- |
  | `kimi-k3` | 3.00 | 0.30 | 3.00 (5-minute lifetime; this client never asks for another, so always 5 minutes) | 15.00 |
  | `kimi-k2.7-code` | 0.95 | 0.19 | priced as input (0.95); the table has no write rate | 4.00 |
  | `kimi-k2.7-code-highspeed` | 1.90 | 0.38 | as input | 8.00 |
  | `kimi-k2.6` | 0.95 | 0.16 | as input | 4.00 |

  A model this table doesn't list has no price. Moonshot's rule: cached + written + uncached input = the reported
  input total. When Moonshot's report leaves out a cache count (it omits counts that are zero), the price must still
  be complete, not a lower bound.
- **OpenRouter, Vercel AI Gateway and the Corbanu API** are priced from the charge the provider reports with the
  response: the request reads `Cost: $X, charged by <provider>`, the technical page's `Price basis` names the field the
  charge came from, and the catalogue's own estimate, if any, stays on the attempt page labelled as an estimate that
  was not used. With several charging providers on one screen, each is named. A request with no reported charge falls
  back to the catalogue rate as a labelled estimate, else "no price".
- **Reasoning above output (#786).** If a usage report says the reasoning tokens exceed the output tokens, reasoning is
  recorded as "not reported"; the rest of the record, including the charge, is kept (before, the whole record was
  dropped).
- **OpenAI catalogue.** `gpt-6.1-sol` is priced at OpenAI's published rates including cache writes ($2 input, $0.10
  cached, $2.50 cache writes, $10 output per million, standard tier). GPT-5.6 Sol is priced at its promotional rates
  ($4 input, $0.40 cached, $5 cache writes, $20 output), which OpenAI says last at least through 2026-11-21.
- DeepSeek's holiday price calendar is out of scope for this run (no DeepSeek connection; date-dependent).

### S08 — `/cost` clarity and the reopen warm-up (#396, #403)

- **No invented money (#396.1).** A token component with no rate states no money; one with 0 tokens reads
  `none — 0 tokens` (not `$0.000000`). Components that do have a rate keep their figure, zero included.
- **Basis-only wording (#396.2).** For a provider whose billing basis comes only from the person's config, the price
  line reads `records only its billing basis: pay per use; set in your config` (no nested parentheses); the
  `Billing basis:` line keeps its parentheses.
- **Reopen warm-up (#403; owner chose: show it, and offer a setting to skip it, off by default).** On providers that
  use a WebSocket connection (OpenAI), reopening a conversation — restarting the product on it (`resume`), `/resume`,
  `/side`, or editing an earlier message — sends one warm-up request (billed like any request, typically 10,000–15,000
  input tokens). The product must:
  - label it a reopen warm-up: `/cost`'s first screen says `Reopen warm-up: N request(s), <cost>`; its request link
    and request page say "Reopen warm-up";
  - once the warm-up has actually been sent, show a notice in the session naming the `skip_reopen_warmup` setting
    (the notice is not written into the conversation, so a later reopen doesn't replay it);
  - with `skip_reopen_warmup = true` (config or `-c skip_reopen_warmup=true`), send no warm-up on reopen; a brand-new
    conversation still warms up;
  - not treat sub-agents or the automatic reviewer starting with copied history as reopens;
  - leave earlier totals unchanged; nothing hidden or counted twice.

### S09 — flows that had never been verified (#402)

- **ChatGPT sign-in.** Every turn on a ChatGPT sign-in reads `Covered by your subscription (not billed per request)`:
  for a catalogued model, an uncatalogued model and the priority service tier alike, with no spend and no "no price".
  (Owner: verified with a stand-in sign-in server only; the owner's own login is never used.)
- **Image generation.** On an OpenAI API key, an image made with the image tool is recorded as a `gpt-image-2`
  request priced at OpenAI's image rates: input tokens × $5/M plus output tokens × $30/M, matching the token counts on
  its technical page. On a ChatGPT sign-in, the image request is recorded as covered by the subscription.
- **Realtime voice sessions:** out of scope for this run; a known gap tracked separately (#824).
- **TensorCash:** not applicable until access is granted (owner decision).

## Owner decisions that bind the tests

- S06 was accepted as limited testing; this run may reopen it.
- Moonshot priced by route first; BigModel deferred (stays "no price").
- OpenRouter rule: the reported charge first, the catalogue rate as a labelled estimate, else "no price". Vercel and
  the Corbanu API by the same rule.
- #786: reasoning above output is recorded as not reported; the rest is kept.
- Reopen warm-up: show it in `/cost` and on reopen, with a skip setting off by default.
- ChatGPT sign-in only against a stand-in; TensorCash not applicable; realtime is tracked as a known gap.
- When a model is offered over several connections, the OpenAI-style connection is the default.

## The test environment (what the tester will have)

- A macOS test machine. The tester drives the product only through a terminal driver (`agms-tui`: start the product,
  type text, press keys, read the screen, wait for text, stop). Each case group starts with a fresh private product
  profile and a small git project as the workspace; profile and project persist within the group (stop/start =
  restart on the same profile; product arguments such as `resume --last`, `resume <id>` or `-c key=value` can be
  given at start).
- The tester can also run `corbanu exec` (one-shot, optionally `--ephemeral`) on the same profile and project.
- Connections ("routes") the tester can start the product on:

  | Route | What it is | Models |
  | --- | --- | --- |
  | `zai` | Z.AI pay-per-use API, **real** (the default; cheap) | `glm-5.3-flash`; the automatic reviewer picks its own Z.AI model |
  | `openai` | OpenAI API key, **real** (small spend; WebSocket connection, so reopen warm-ups happen here) | `gpt-6-luna` (cheapest), `gpt-5.6-luna`, `gpt-5.6-sol`, `gpt-6.1-sol`, `gpt-5.4`; the image tool (`gpt-image-2`) |
  | `moonshot` | Moonshot Kimi Open Platform, **real**, configured as a person would (a provider named `kimi` at `https://api.moonshot.ai/v1`, Chat Completions) | `kimi-k3`, `kimi-k2.7-code` — **one first turn per model only** (owner's spend limit); a second turn on the same conversation only if the owner grants it (pending) |
  | `chatgpt` | ChatGPT sign-in against a **stand-in** sign-in server (plan "plus"); already signed in when the product starts. The stand-in answers every turn with a short text. The tester can make the stand-in's next answer use the image tool. | `gpt-5.4` (catalogued), `gpt-mock-unlisted` (uncatalogued), priority tier via `-c service_tier="priority"` |
  | `openrouter` | OpenRouter through a **stand-in** that reports fixed usage and a charge | `z-ai/glm-5.3-flash`: every request reports 2,000 input tokens (500 of them cached), 40 output (12 of them reasoning), charge `cost` 0.00052 USD. `deepseek/deepseek-v4.1-flash`: every request reports 1,000 input (0 cached), 40 output with **60 reasoning** (reasoning above output), charge 0.00031 USD |
  | `vercel` | Vercel AI Gateway through a **stand-in** | `zai/glm-5.3-flash`: every request reports 3,000 input (1,000 cached), 25 output, charge `gatewayCost` 0.0007125 USD |
  | `corbanu` | Corbanu API through a **stand-in** | `corbanu/deepseek-v4.1-flash`: every request reports 1,500 input, 30 output, charge `corbanu_charge_usd` 0.001850 USD |
  | `custom` | A person-configured provider (named `mylab`) at a **stand-in** host with no published price and no reported charge, declared `billing = "pay_per_use"` in the config | `lab-model-1`: every request reports 800 input, 20 output |

- Stand-ins answer with a short text and never run tools (except the ChatGPT image answer when asked). One turn can
  produce more than one request (the product may make extra requests of its own); the stand-in reports the same fixed
  usage for each.
- Real routes report real usage; the tester recomputes from the tokens on the technical page and the rates above
  (OpenAI image rates $5/M input, $30/M output).
- The product's model for ordinary work is GLM 5.3 Flash; the same model family also drives the tester.
- The tester cannot read the product's internal files or databases, only screens, the workspace files and the
  product's own exported output.
- Spend: keep real turns short ("Reply with the single word OK." style); total real spend should stay well under $1.

## Out of scope / named limits

Realtime (#824), TensorCash, DeepSeek holiday calendar, BigModel, re-pricing history, how older records read, Windows
and Linux.
