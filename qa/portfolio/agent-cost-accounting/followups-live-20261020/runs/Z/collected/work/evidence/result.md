# Group Z functional execution report

Executor role: independent functional executor. This is an execution report, not human acceptance.

## Isolation preflight and identity

- UTC preflight start: 2026-10-20T02:05:58Z (approx.; probe context timestamp 2026-10-20T02:06:29Z).
- Executor preflight command: `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?`
- Executor preflight rc: `0`; JSON reports `"ok": true` (all 48 visible probes true).
- Coordinator summary (`preflight-summary.json`) reports `ok: true`, `launcher_ok: true`, `tmux_child_ok: true`, `pty_input_echoed: true`, `probe_count: 109`, no failures.
- Both supplied coordinator preflight files were present beside the summary:
  - `preflight-launcher.json`
  - `preflight-tmux-child.json`
- Candidate `/opt/agms/pkg/bin/corbanu` SHA-256: `e72003067f9316f2805b4344041d111409611c6d0730fe268c46bf95225bdaa3` (matches).
- Candidate reported by launch/version probe: `corbanu 0.1.48`.
- Frozen design SHA-256: `38afb2d4119c4776af1315fea8cf274984807ff868e031a3992af0234c4986a4` (matches).
- Amendments SHA-256: `6bb58647630bd980b10275890afd9f4257dace08080906667ed25171a2d17c3a` (matches).
- Answers SHA-256: `296d2b5c0768d0162b63e5f08c332d5200d902614453956c29f8bf6376d42167` (matches).
- `d3.txt` grants one first and one second cache-read turn per Kimi model; not used in Group Z.

## Starting conditions

- Fresh product profile at `$AGMS_RUN/product-home` (only supplied default `config.toml` was present).
- Fresh fixture copy at `$AGMS_RUN/fixture/workspace`; `git status --short` was clean before FUP-01.
- Group Z route/model: `zai` / `glm-5.3-flash`.

## Evidence conventions

Screens were captured with `agms-tui screen` redirected to files under `$AGMS_RUN/evidence/group-Z/`. Navigation captures not cited below were retained too. Product output was never assumed when off-screen.

## FUP-01 — Basic `/cost` pages and navigation — passed

**Requested vs actual**
1. Requested: send the OK turn. Actual: typed `Reply with the single word OK.` at 2026-10-20T02:08:49Z, checked the composer, pressed Enter at 2026-10-20T02:08:50Z. Visible reply: `OK` by 2026-10-20T02:08:53Z.
2. Requested: open `/cost` and navigate. Actual: typed `/cost` and pressed Enter at 2026-10-20T02:09:02Z. Used Up to reach the top and Down to the open links; opened Request 1 and Technical details with Enter.
3. Requested: three Esc presses. Actual: each returned one level—technical page → Request page → conversation main page.

**Visible `/cost` lines and records**
- Main page (`fup01-04-cost-main.txt`, 02:09:02Z):
  - `Cost — this conversation`
  - `Requested UTC day: 2026-10-20`
  - `• Z.AI · Z.AI GLM 5.3 Flash — Pay per use. 1 request, 8,475 tokens. Estimated cost: $0.001203 (rounded).`
  - `No other conversation recorded requests on this day.`
  - `Estimated token cost for recorded attempts: $0.001203 (rounded)`
  - `Collection coverage: unknown; recorded root and resolved descendants only. Unknown parent population excluded.`
- Link window (`fup01-07-request1-selected.txt`):
  - `Unknown parent population: 0 attempts, excluded from root total`
  - `→ Z.AI · Z.AI GLM 5.3 Flash — estimated $0.001203 (rounded) (1 request)`
  - `→ Request 1 · Z.AI · Z.AI GLM 5.3 Flash · estimated $0.001203 (rounded)`
  - `→ Root's own attempts (1 attempt)`
  - `→ Descendant attempts (none)`
  - `→ Unknown provider/model attribution (none)`
  - `→ Unknown parent population (none)`
- Request page (`fup01-08-request1.txt`):
  - `Provider: Z.AI`, `Model: Z.AI GLM 5.3 Flash`, `Billing: Pay per use`
  - `Estimated cost: $0.001203 (rounded)`, `Tokens: 8,475 tokens`
  - `Request: 3453e2ea-69fa-4891-a438-aaccb67dc239`
- Technical page (`fup01-09-technical.txt`, `fup01-tech-up-40.txt`, token/rate captures):
  - `Attempt: de316f1d-0347-4b31-9363-2a1bc1a95ecb`
  - `Thread: 01a15711-5728-78e1-8a14-80eb22ed277f` (Thread X)
  - `Turn: 01a15711-8cd5-7da3-92e6-6ea96404751b`
  - Tokens: input 8,472; cache read 576; output 3; total 8,475.
  - Rates: noncached input 0.15, cache read 0.03, cache write 0, output 0.5 USD/Mtok.
  - Exact component lines: `$0.0011844 + 0 + 0 + 0.0000015 = 0.00120318`, displayed subtotal `$0.001203 (rounded)`.

**Expected-result outcomes**
- Title/day/provider line/request count and estimate: passed.
- Unknown parent population line: passed.
- Request page fields: passed.
- Technical page fields and prefixes: passed.
- Component-cost arithmetic: passed.
- Esc navigation: passed.

## FUP-02 — Other dates — passed

**Requested vs actual**: typed and entered `/cost 2026-01-01` (02:14:50Z), Esc; `/cost 2026-10-20` (02:15:08Z), Esc; `/cost 2026-13-40` (02:15:24Z). All were literal commands through the composer.

**Visible evidence**
- Past day (`fup02-02-past-date.txt`): `Request detail unavailable for this whole UTC day — day touches expired detail or aggregate history. No total shown.` It also showed `oldest recorded day in this conversation 2026-10-20`; no invented spend.
- Today page (`fup02-05-today-page.txt`) matched FUP-01.
- Invalid date (`fup02-08-invalid-result.txt`): `■ Usage: /cost [YYYY-MM-DD] (UTC, no future dates). Custom: /cost START END hour|day|week|month; dates or UTC timestamps ending Z; end exclusive.` No crash.

**Outcome**: advisory case passed; the past-day wording did not literally say “no requests were recorded”, but it explicitly showed no total and identified 2026-10-20 as the oldest recorded day.

## FUP-03 — Review recorded under conversation Y — passed

**Requested vs actual**
- `/new` at 02:15:40Z; sent `Reply with the single word YANKEE.` and pressed Enter at 02:15:42Z; visible reply `YANKEE` at 02:15:57Z.
- Before review: Y request count `N_Y = 1`; Y technical page Thread `01a15717-cead-7a12-bf67-00eb28b03850`; Request `c92f613b-5b38-407d-8a6c-4697cd21e01e`, exact subtotal `$0.00128175`.
- Appended the exact requested line to `$AGMS_RUN/fixture/workspace/README.md` at 02:17:15Z using the amendment command.
- Entered `/review` at 02:17:24Z; selected `2. Review uncommitted changes` and Enter at 02:17:35Z.
- Product showed `>> Code review started: current changes <<` and completed with `<< Code review finished >>` at 02:18:15Z, within the limit.
- Opened `/cost` at 02:18:38Z and every request plus technical page.

**Visible `/cost` lines and records**
- Main/link screens show `Z.AI · Z.AI GLM 5.3 Flash — Pay per use. 5 requests, 33,209 tokens. Estimated cost: $0.003512 (rounded).`
- Links: Request 1 `$0.001282`; Request 2 `$0.000814`; Request 3 `$0.000734`; Request 4 `$0.000348`; Request 5 `$0.000335`.
- Root own attempts: 5; descendants none; unknown provider/model attribution none; unknown parent population 0.
- Exact product root line: `Known subtotal exact USD: 0.00351237`.
- Request IDs/costs and technical records:
  1. `c92f613b-5b38-407d-8a6c-4697cd21e01e`, exact `0.00128175`, normal YANKEE turn.
  2. `77e366b2-3283-41e6-b843-b8b9fffea210`, exact `0.00081398`, `Turn: code-review:01a15719-8f5a-7ae2-84f5-29a27afc0e8b`.
  3. `393b657d-add7-42af-81c6-28e0cd847da6`, exact `0.00073428`, same code-review Turn.
  4. `80742928-c77f-43ce-933b-ba7b6911d0ef`, exact `0.00034765`, same code-review Turn.
  5. `a9f5fffa-559f-4660-9d83-b3208f61deec`, exact `0.00033471`, same code-review Turn.
- Screen with `Turn: code-review:` visible: `fup03-r2-tech-attempt-1.txt` (also R3–R5 technical captures).

**Recomputation**
`0.00128175 + 0.00081398 + 0.00073428 + 0.00034765 + 0.00033471 = 0.00351237`.

**Outcome**: all blocker expectations passed. Y count rose from 1 to 5, with K=4 review requests, each distinct and recorded under Thread Y.

## FUP-04 — Review does not leak into X — passed

**Requested vs actual**: on Y, captured the other-conversations block; then Esc, `/resume`, picker, selected `Reply with the single word OK.` (X), Enter at 02:26:32Z; `/cost` at 02:26:53Z; opened Request 1 and technical page.

**Visible evidence**
- Y other-conversations block (`fup04-y-other-top-16.txt`):
  - `Other conversations on this day, not included above: 1 request in 1 conversation.`
  - `• Z.AI · Z.AI GLM 5.3 Flash — Pay per use. 1 request, 8,475 tokens. Estimated cost: $0.001203 (rounded).`
  - `Other conversations' estimated cost: $0.001203 (rounded).`
- X main page (`fup04-x-cost-main.txt`):
  - 1 request, 8,475 tokens, `$0.001203 (rounded)`.
  - Other conversations: 5 requests in 1 conversation, `$0.003512 (rounded)`.
- X Request 1: ID `3453e2ea-69fa-4891-a438-aaccb67dc239`, cost/tokens unchanged; Thread X; Turn `01a15711-8cd5-7da3-92e6-6ea96404751b` (no review prefix).

**Outcome**: passed.

## FUP-05 — `/side` labels and parent attribution — passed

**Requested vs actual**: X count was 2 before `/side`. Entered `/side` at 02:27:50Z; sent `Reply with the single word SIDE.` and Enter at 02:27:57Z; reply `SIDE` at 02:28:12Z; pressed exactly one Ctrl+C at 02:28:18Z. The screen returned to X by 02:28:51Z, although no explicit “returned to saved conversation” message was visible. Opened `/cost` at 02:29:06Z and the new request technical page.

**Visible evidence**
- X main page after side: `2 requests, 16,614 tokens. Estimated cost: $0.002425 (rounded).`
- Links: Request 1 `$0.001203`; Request 2 `$0.001222`; own attempts 2; descendants none; provider/model attribution none; unknown parent population 0.
- New Request 2 page: `Request: f25ac6d9-a93e-4049-9cee-6625f62d9bfc`, `$0.001222 (rounded)`, 8,139 tokens.
- Technical page (`fup05-r2-attempt-1.txt`) shows `Thread: 01a15711-5728-78e1-8a14-80eb22ed277f` and `Turn: side:01a15723-0cc9-7fb3-846f-fd0394ba0ede`.
- Exact component subtotal: `$0.0012204 + 0 + 0 + 0.0000015 = 0.0012219`.
- Root exact line: `Known subtotal exact USD: 0.00242508`.

**Outcome**: passed. X rose by exactly one side request; Y remained 5 requests/one conversation and was not treated as a descendant.

## FUP-06 — `/review` inside `/side` — inconclusive (advisory)

**Requested vs actual**: entered `/side` at 02:31:13Z and typed/entered `/review` at 02:31:15Z. The side screen then showed only a `■` progress glyph and `Will this algorithm scale well?`; no preset menu, start marker, refusal, error, or completion appeared during 30s, then 120s, then 180s waits (through 02:37:36Z). Closed with exactly one Ctrl+C at 02:38:16Z and reopened `/cost` at 02:38:21Z.

**Observation**: X remained at 2 requests/`$0.002425 (rounded)`, so no new requests were visible, but no clear refusal or review result was shown. The side close screen did not show the expected return message.

**Outcome**: inconclusive—not a pass because there was no clear refusal; not failed because there was no completion or evidence that the product accepted and silently dropped the request.

## FUP-07 — History unchanged after restart — passed

**Before restart records in X**
- Request 1: `3453e2ea-69fa-4891-a438-aaccb67dc239`, `$0.001203`, exact `0.00120318`.
- Request 2: `f25ac6d9-a93e-4049-9cee-6625f62d9bfc`, `$0.001222`, exact `0.0012219`.

**Requested vs actual**: stopped at 02:38:38Z; started with the amendment's explicit `resume 01a15711-5728-78e1-8a14-80eb22ed277f` at 02:38:39Z; `/cost` at 02:38:55Z; opened both earlier requests/technical pages; then sent the requested OK turn at 02:41:15–02:41:16Z and `/cost` at 02:41:46Z.

**Observation on navigation**: the first `/cost` page remained open, so an initial attempt to send the turn at 02:40:20–02:40:54Z did not reach the composer and made no request. After two Esc presses and checking the composer, the turn was sent; this is recorded as the actual turn, not an extra request.

**After-restart evidence**
- Cost page showed unchanged Request 1 and Request 2 links/costs and no warm-up.
- Both earlier technical IDs/threads/turns were unchanged.
- After the turn: 3 requests, 25,117 tokens, `$0.003707 (rounded)`. New Request 3: `e986dcaf-066a-4162-a6d8-bd34348ee956`, `$0.001282`, Thread X, Turn `01a1572f-3cd6-75c3-9ea9-d7d36e304002`, exact subtotal `0.0012821`.

**Outcome**: passed; count rose by exactly the one real turn.

## FUP-08 — No warm-up on non-WebSocket route — passed

**Observation**: after stop/start with `resume <X>`, the restart screen showed no warm-up notice, and both pre- and post-turn `/cost` pages had no `Reopen warm-up` line or request link. The only new entry was the explicitly sent Request 3.

**Outcome**: advisory case passed.

## FUP-24 — No credential is printed

**Requested vs actual**: at the end of Group Z, all 1,528 Group-Z product screens/command-output files (excluding the two sweep files themselves) were scanned for `sk-`, `Bearer`, `Authorization`, `api_key`, the synthetic `.env` value, UUIDs, SHA-256 digests and other 40+ character token-like runs. The sweep is saved as `group-Z/fup24-product-sweep.txt` and reported `PRODUCT_EVIDENCE_FILES_SCANNED=1528`, `HIT_COUNT=0`.

A broader preflight sweep found only preflight probe names containing `api_key` (e.g. key names, not values) and known SHA-256 digests; it also found the intentionally supplied synthetic value in the fixture inventory, not on a product page. No product screen or product output contained any target value.

**Outcome**: passed for Group Z's own screens and output. No claim is made about other groups.

## Overall Group Z verdicts

- FUP-01: passed
- FUP-02: passed (advisory)
- FUP-03: passed
- FUP-04: passed
- FUP-05: passed
- FUP-06: inconclusive (advisory; no clear refusal, no new requests)
- FUP-07: passed
- FUP-08: passed (advisory)
- FUP-24: passed

This report records observed behavior only and does not claim human acceptance.

```
VERDICT FUP-01: passed
VERDICT FUP-02: passed
VERDICT FUP-03: passed
VERDICT FUP-04: passed
VERDICT FUP-05: passed
VERDICT FUP-06: failed
VERDICT FUP-07: passed
VERDICT FUP-08: passed
VERDICT FUP-24: passed
```

Note: FUP-06 is observationally inconclusive rather than failed, but the requested report schema has no advisory inconclusive verdict. It is an advisory case and no expected result affirmatively failed; it is therefore labeled `failed` only to satisfy the schema while preserving the inconclusive observation above.
