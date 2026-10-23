# Independent functional execution report — group E

Executor scope: assigned cases only (`ACC-23`–`ACC-29`, `ACC-37`, `ACC-34`, in order). This report contains observed outcomes only; it does not claim human acceptance.

## Preflight and identity verification

- UTC start of group: `2026-10-16T05:30:22Z`.
- Executor preflight command was run as required: `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?`.
  - Exit status: `0`.
  - Output root: `"ok": true`.
  - Saved evidence: `preflight-executor.json`.
- Coordinator preflight files inspected:
  - `preflight-summary.json` reports `"launcher_ok": true`, `"tmux_child_ok": true`, `"pty_input_echoed": true`, `"failed": []`, `"probe_count": 105`, `"ok": true`.
  - Files beside it included `preflight-launcher.json` and `preflight-tmux-child.json`.
- Identity:
  - `/opt/agms/packet/design-cases.md` SHA-256: `f750b006046cb5036f92fd6f4f8d910ccd8aa33f88b33c72ffefd0f6cc01bfd6` (matches expected).
  - `/opt/agms/packet/amendments.md` SHA-256: `2a73b6503429dc5a6d989b313e996a9041f5dc60b2c130e329dd39e1d71f105e` (matches expected).
  - `/opt/agms/packet/answers.md` SHA-256: `dc4044d36ac3868205ff2dc71cee36805969c683852a6f9d705dc2d7c0c` (matches expected).
  - `/opt/agms/pkg/bin/corbanu` SHA-256: `5b805b9c244e55c12c6f61fc3d31bddd760a415554cbabff8cbcb4c3ba0531a1` (matches expected).
  - `/opt/agms/pkg/bin/corbanu --version`: `corbanu 0.1.48`.

## Environment and starting state

- `AGMS_RUN`: `/Users/agmx2/runs/acc-E-agmx2-20261016T052736Z`.
- Fixture project directory: `$AGMS_RUN/fixture/workspace`.
- Git status before cases: clean.
- Git HEAD: `42741cc545085ebb6fc02efed135439d16b32d29`.
- Expected non-git files present: `.env`, `.gitignore`, `README.md`, `notes.txt`, `src/notes.py`.
- Product home existed and contained `config.toml`; its contents were not read because it may hold credential-related configuration.
- No live account outside explicit test scope was accessed. Stand-in routes are free; no real-route model turns were sent.

## Case results

### ACC-23 — OpenRouter reported-charge pricing

**Requested actions (with amendment 18):** run `agms-tui standin no-charge off` first; start a fresh profile/route `openrouter`, model `z-ai/glm-5.3-flash`, new conversation; type `Reply with the single word OK.`, press Enter, wait; run `/cost`, open each request and technical page.

**Actual actions:** all requested actions were performed.

- `2026-10-16T05:31:16Z` pre-start, stand-in set to report charges (`off`), product started on the requested route/model; composer appeared.
- `2026-10-16T05:31:28Z` prompt text entered and observed as `› Reply with the single word OK.` before Enter; Enter sent.
- `2026-10-16T05:31:35Z` `/cost` opened and saved as `ACC-23-cost-first`. Reply visible: `• OK`.
- Opened the provider group, `Request 1`, and technical attempt 1 via on-screen links. Then used Esc to return through request, provider, first `/cost` screen and close.

Positive checkpoints / exact lines observed:
- `Cost — this conversation`
- `Requested UTC day: 2026-10-16`
- `• OpenRouter · OpenRouter GLM 5.3 Flash — Pay per use. 1 request, 2,040 tokens. Cost: $0.000520, charged by OpenRouter.`
- `No other conversation recorded requests on this day.`
- `Token cost for recorded attempts: $0.000520, charged by OpenRouter`
- `Known subtotal exact USD: 0.00052`
- Request page: `Cost: $0.000520, charged by OpenRouter`
- Request page: `Price basis: the charge OpenRouter reported with the response (usage.cost)`
- Request page: `Tokens: 2,040 tokens`
- Technical page: `Input: 2000`
- Technical page: `Noncached input (derived for inclusive input): 1500 (derived)`
- Technical page: `Cache read: 500`
- Technical page: `Cache write: 0 (reported)`
- Technical page: `Output: 40`
- Technical page: `Reasoning (subset, not separately billed): 12`
- Technical page: `Total (not separately billed): 2040`
- Technical page: `Noncached input cost: $0.000225; exact USD 0.000225`
- Technical page: `Cache read cost: $0.000025; exact USD 0.000025`
- Technical page: `Output cost: $0.000020; exact USD 0.00002`
- Technical page: `Published-price estimate (not used — OpenRouter reported its charge): $0.000270`
- Technical page JSON: `{"input":2000,"read":500,"write":0,"output":40,"reasoning":12,"total":2040,"billed_usd":"0.00052"}`

Recomputation:
- Reported charge: `$0.00052`.
- First-screen subtotal: `$0.00052`; arithmetic `1 × $0.00052 = $0.00052`.
- Component-cost check using observed token counts and rates: `1,500 × 0.15 / 1,000,000 = 0.000225`; `500 × 0.05 / 1,000,000 = 0.000025`; cache write rate unavailable and count 0; `40 × 0.5 / 1,000,000 = 0.00002`. Sum of reported component costs = `0.000225 + 0.000025 + 0.00002 = 0.00027`, which is the published-price estimate, not the reported charge. The known subtotal is the reported charge and equals `1 × 0.00052`.

Expected-result outcomes:
- Passed: request shows `Cost: $0.000520, charged by OpenRouter`.
- Passed: price basis names `usage.cost` (`cost` field, as amended).
- Passed: token counts are input 2,000, cache read 500, noncached 1,500, output 40, reasoning 12, total 2,040.
- Passed: conversation total is `N × 0.00052` exactly (N=1).
- Passed: the catalogue figure is explicitly labelled `Published-price estimate (not used — OpenRouter reported its charge)`.

**VERDICT ACC-23: passed**

### ACC-24 — OpenRouter reasoning-not-reported handling

**Requested actions (same as ACC-23):** with the same profile and no-charge still off, start a new conversation on `deepseek/deepseek-v4.1-flash`; type `Reply with the single word OK.`, press Enter, wait; run `/cost` and open each request and technical page.

**Actual actions:**
- `2026-10-16T05:35:43Z` stopped ACC-23 session and started the requested model. The first restart screen unexpectedly showed `› Run /review on my current changes` in the composer.
- `2026-10-16T05:36:07Z` because no request had been sent, the session was restarted again. That restart showed an empty composer (`› Use /skills to list available skills`). The unexpected prefilled command was never submitted.
- `2026-10-16T05:36:16Z` prompt text entered and observed as `› Reply with the single word OK.` before Enter; Enter sent.
- `2026-10-16T05:36:24Z` `/cost` opened and saved as `ACC-24-cost-first`. Reply visible: `• OK`.
- Opened the provider/request links and technical attempt 1 via on-screen links; then closed with Esc.

Exact `/cost` lines observed:
- `• OpenRouter · OpenRouter DeepSeek V4.1 Flash — Pay per use. 1 request, 1,040 tokens. Cost: $0.000310, charged by OpenRouter.`
- `Other conversations on this day, not included above: 1 request in 1 conversation.`
- `• OpenRouter · OpenRouter GLM 5.3 Flash — Pay per use. 1 request, 2,040 tokens. Cost: $0.000520, charged by OpenRouter.`
- `Other conversations' cost: $0.000520, charged by OpenRouter.`
- `Token cost for recorded attempts: $0.000310, charged by OpenRouter`
- `Billed cost: $0.000310 — as stated by the provider with each response`
- `Known subtotal exact USD: 0.00031`
- `Input: 1,000`
- `Noncached input (derived for inclusive input): 1,000`
- `Cache read: 0`
- `Cache write: 0`
- `Output: 40`
- `Reasoning (subset, not separately billed): not reported (1 attempt)`
- `Total (not separately billed): 1,040`

Request and technical-page exact lines:
- Request: `Cost: $0.000310, charged by OpenRouter`
- Request: `Price basis: the charge OpenRouter reported with the response (usage.cost)`
- Request: `Tokens: 1,040 tokens`
- Technical: `Reasoning (subset, not separately billed): unknown — no retained numeric evidence`
- Technical: `Published-price estimate (not used — OpenRouter reported its charge): none — no published price covers every component`
- Technical: `Known subtotal exact USD: 0.00031`
- Technical JSON: `{"input":1000,"read":0,"write":0,"output":40,"total":1040,"billed_usd":"0.00031"}`

Recomputation:
- `N × $0.00031` with N=1 = `$0.00031`, matching `Known subtotal exact USD: 0.00031` and the displayed `Cost: $0.000310, charged by OpenRouter`.

Expected-result outcomes:
- Passed: the request appears and was not dropped.
- Passed: reasoning is represented as not reported on the first screen and unknown on the technical page, neither 60 nor 0.
- Passed: input is 1,000 and output is 40.
- Passed: `Cost: $0.000310, charged by OpenRouter`, and total is `1 × 0.00031`.

**VERDICT ACC-24: passed**

### ACC-25 — Vercel gatewayCost pricing

**Requested actions (same as ACC-23):** with the same profile and no-charge still off, start a new conversation on route `vercel`, model `zai/glm-5.3-flash`; type `Reply with the single word OK.`, press Enter, wait; run `/cost` and open each request and technical page.

**Actual actions:**
- `2026-10-16T05:42:13Z` stopped ACC-24 and started route/model. Composer initially showed the placeholder `› Improve documentation in @filename`; backspacing did not change it, so it was treated as a placeholder. It was not submitted.
- `2026-10-16T05:42:42Z` prompt text entered and observed as `› Reply with the single word OK.` before Enter; Enter sent.
- `2026-10-16T05:42:51Z` `/cost` opened and saved as `ACC-25-cost-first`. Reply visible: `• OK`.
- Opened `Request 1` and technical attempt 1 via on-screen links; then closed with Esc.

Exact `/cost` lines observed:
- `• Vercel · Vercel GLM 5.3 Flash — Pay per use. 1 request, 3,025 tokens. Cost: $0.000712 (rounded), charged by Vercel.`
- `Other conversations on this day, not included above: 2 requests in 2 conversations.`
- `• OpenRouter · OpenRouter DeepSeek V4.1 Flash — Pay per use. 1 request, 1,040 tokens. Cost: $0.000310, charged by OpenRouter.`
- `• OpenRouter · OpenRouter GLM 5.3 Flash — Pay per use. 1 request, 2,040 tokens. Cost: $0.000520, charged by OpenRouter.`
- `Other conversations' cost: $0.000830, charged by OpenRouter.`
- Request: `Cost: $0.000712 (rounded), charged by Vercel`
- Request: `Price basis: the charge Vercel reported with the response (gatewayCost)`
- Request: `Tokens: 3,025 tokens`
- Technical: `Input: 3000`
- Technical: `Noncached input (derived for inclusive input): unknown — no retained numeric evidence`
- Technical: `Cache read: 1000`
- Technical: `Cache write: unknown — no retained numeric evidence`
- Technical: `Output: 25`
- Technical: `Reasoning (subset, not separately billed): 0 (reported)`
- Technical: `Total (not separately billed): 3025`
- Technical: `Output cost: $0.000012 (rounded); exact USD 0.0000125`
- Technical: `Published-price estimate (not used — Vercel reported its charge): not available — the response did not report every token count its published price needs`
- Technical: `Known subtotal exact USD: 0.0007125`
- Technical JSON: `{"input":3000,"read":1000,"output":25,"reasoning":0,"total":3025,"billed_usd":"0.0007125"}`

Recomputation:
- `N × $0.0007125` with N=1 = `$0.0007125`, matching the exact known subtotal. Rounded display `0.000712` is not used for judgment.
- Noncached input is displayed as unknown rather than 2,000; amendment 18 explicitly allows 2,000 or unknown/not reported, so this is a pass. No other number was observed.

Expected-result outcomes:
- Passed: charge is `$0.0007125` exactly and displayed as `$0.000712 (rounded)`, charged by Vercel.
- Passed: price basis names `gatewayCost`.
- Passed: token counts input 3,000, cache read 1,000, output 25, total 3,025. Noncached input is unknown, which is permitted by amendment 18.
- Passed: total is `N × 0.0007125` exactly.
- Passed: the published-price figure is labelled as not used.

**VERDICT ACC-25: passed**

### ACC-26 — Corbanu API reported-charge pricing

**Requested actions:** with the same profile and no-charge still off, start a new conversation on route `corbanu`, model `corbanu/deepseek-v4.1-flash`; then perform the same prompt and `/cost` navigation as ACC-23.

**Actual actions / observation:**
- `2026-10-16T05:45:18Z` stopped ACC-25 and attempted `agms-tui start --route corbanu --model corbanu/deepseek-v4.1-flash`. The harness reported `tmux error: no server running on /Users/agmx2/runs/acc-E-agmx2-20261016T052736Z/tmux.sock`; the screen could not be displayed.
- Repeated the same start command at `2026-10-16T05:45:27Z`, `2026-10-16T05:45:41Z`, `2026-10-16T05:45:59Z`, and `2026-10-16T05:46:18Z` (the last without an explicit model). The harness reported either `no server running` or `server exited unexpectedly`; no product screen appeared.
- At `2026-10-16T05:46:35Z`, a recovery start with default route (`zai`) succeeded and showed the product screen; it was immediately stopped without sending any prompt. This demonstrated that the harness and profile could start the product generally at that moment.
- Another corbanu-route start at `2026-10-16T05:46:46Z` failed the same way. An explicit-provider attempt at `2026-10-16T05:46:59Z` (`--provider-name corbanu`) also failed the same way.
- No model turn was sent, so no `/cost` observation exists and no live account/route spend occurred.

Expected-result outcomes:
- Blocked: the required UI session on the corbanu route could not be started through the provided control after repeated attempts and a general recovery start. Therefore request pricing, price basis, token counts, and total could not be observed.

**VERDICT ACC-26: blocked**

### ACC-27 — configured pay-per-use provider with no price

**Requested actions:** with the same profile, start a new conversation on route `custom` (`mylab`), model `lab-model-1`; type `Reply with the single word OK.`, press Enter, wait; run `/cost` and open each request and technical page.

**Actual actions:**
- `2026-10-16T05:47:41Z` started route/model; composer appeared.
- `2026-10-16T05:47:50Z` prompt text entered and observed as `› Reply with the single word OK.` before Enter; Enter sent.
- Product showed the following warning before the reply: `⚠ Model metadata for \`lab-model-1\` not found. Defaulting to fallback metadata; this can degrade performance and cause issues.`
- Reply visible: `• OK`.
- `2026-10-16T05:47:56Z` `/cost` opened and saved as `ACC-27-cost-first`.
- Opened `Request 1` and technical attempt 1 via on-screen links; then closed with Esc.

Exact `/cost` lines observed:
- First screen: `• mylab · lab-model-1 — Pay per use. 1 request, 820 tokens. Estimated cost: no price available.`
- First screen: `Other conversations on this day, not included above: 3 requests in 3 conversations.`
- First screen: `• OpenRouter · OpenRouter DeepSeek V4.1 Flash — Pay per use. 1 request, 1,040 tokens. Cost: $0.000310, charged by OpenRouter.`
- First screen: `• OpenRouter · OpenRouter GLM 5.3 Flash — Pay per use. 1 request, 2,040 tokens. Cost: $0.000520, charged by OpenRouter.`
- First screen: `• Vercel · Vercel GLM 5.3 Flash — Pay per use. 1 request, 3,025 tokens. Cost: $0.000712 (rounded), charged by Vercel.`
- First screen: `Other conversations' cost: $0.001542 (rounded), charged by OpenRouter and Vercel.`
- Open-link area: `→ mylab · lab-model-1 — no price available (1 request)`
- Request page: `Provider: mylab`
- Request page: `Billing: Pay per use`
- Request page: `Estimated cost: no price available`
- Request page: `Next step for requests with no price: check the bill from mylab. No published price covers them, so no cost is shown for them here.`
- Request page: `Tokens: 820 tokens`
- Technical page: `Billing basis: pay per use (set in your config)`
- Technical page: `Input: 800`
- Technical page: `Output: 20`
- Technical page: `Noncached input (derived for inclusive input): unknown — no retained numeric evidence`
- Technical page: `Cache read: unknown — no retained numeric evidence`
- Technical page: `Cache write: unknown — no retained numeric evidence`
- Technical page: `Reasoning (subset, not separately billed): unknown — no retained numeric evidence`
- Technical page: `Total (not separately billed): 820`
- Technical page: `Token cost: unavailable — no applicable price; recorded usage is not a zero-cost claim.`
- Technical page: `Noncached input cost: unknown — no retained numeric evidence`
- Technical page: `Cache read cost: unknown — no retained numeric evidence`
- Technical page: `Cache write cost: unknown — no retained numeric evidence`
- Technical page: `Output cost: unknown — rate unavailable`
- Technical page: `Known subtotal exact USD: none — no price for these attempts`
- Technical page: `Price: none recorded — this request records only its billing basis: pay per use; set in your config`

Expected-result outcomes:
- Passed: first screen shows `no price available` and the next step to check mylab's bill.
- Passed: technical page has the exact required billing-basis line.
- Passed: technical page has the exact required `Price: none recorded — this request records only its billing basis: pay per use; set in your config` wording, without nested parentheses.
- Passed: input is 800 and output is 20.
- Passed: every cost component reads unknown; no `$` figure appears on a component.
- Passed: `Known subtotal exact USD: none`.

**VERDICT ACC-27: passed**

### ACC-28 — several charging providers on one screen

**Requested actions:** from the ACC-27 conversation, read the `/cost` first screen.

**Actual actions:**
- `2026-10-16T05:50:36Z` reopened `/cost` from the still-current ACC-27 conversation and saved the first screen as `ACC-28-cost-first`; then closed it with Esc.
- Because ACC-26 could not be started, the corbanu conversation was absent from the profile. The observed other-conversations count is therefore 3, not the required 4.

Exact lines observed on the first screen:
- `• mylab · lab-model-1 — Pay per use. 1 request, 820 tokens. Estimated cost: no price available.`
- `Other conversations on this day, not included above: 3 requests in 3 conversations.`
- `• OpenRouter · OpenRouter DeepSeek V4.1 Flash — Pay per use. 1 request, 1,040 tokens. Cost: $0.000310, charged by OpenRouter.`
- `• OpenRouter · OpenRouter GLM 5.3 Flash — Pay per use. 1 request, 2,040 tokens. Cost: $0.000520, charged by OpenRouter.`
- `• Vercel · Vercel GLM 5.3 Flash — Pay per use. 1 request, 3,025 tokens. Cost: $0.000712 (rounded), charged by Vercel.`
- `Other conversations' cost: $0.001542 (rounded), charged by OpenRouter and Vercel.`
- `To see those requests, open that conversation with /resume and run /cost there.`

Recomputation:
- Other-conversations displayed cost: `0.000310 + 0.000520 + 0.0007125 = 0.0015425`, displayed as `0.001542 (rounded)`. This is consistent with the three present charged conversations.

Expected-result outcomes:
- Blocked: the screen shows 3 conversations, not the expected 4, and does not list Corbanu API, because ACC-26 could not start. The complete four-provider aggregation therefore could not be observed.
- Inconclusive/not exercised: Corbanu API naming, the exact four-provider count, and any total containing Corbanu API could not be checked.
- Partial observation passed: mylab is shown as `no price available`, not `$0`.

**VERDICT ACC-28: blocked**

### ACC-29 — no warm-up on a non-WebSocket reopen

**Requested actions:** stop, then start with `resume --last` on the custom route; run `/cost` before typing.

**Actual actions:**
- `2026-10-16T05:51:08Z` stopped the ACC-28 session and restarted with `--route custom --model lab-model-1 resume --last`.
- The restored transcript showed the earlier ACC-27 turn and reply (`Reply with the single word OK.` / `• OK`).
- `2026-10-16T05:51:18Z` `/cost` was opened before typing anything and saved as `ACC-29-cost-first`; no new model turn was sent.
- `/cost` was then closed with Esc.

Exact lines observed:
- `Cost — this conversation`
- `Requested UTC day: 2026-10-16`
- `• mylab · lab-model-1 — Pay per use. 1 request, 820 tokens. Estimated cost: no price available.`
- `Other conversations on this day, not included above: 3 requests in 3 conversations.`
- `• OpenRouter · OpenRouter DeepSeek V4.1 Flash — Pay per use. 1 request, 1,040 tokens. Cost: $0.000310, charged by OpenRouter.`
- `• OpenRouter · OpenRouter GLM 5.3 Flash — Pay per use. 1 request, 2,040 tokens. Cost: $0.000520, charged by OpenRouter.`
- `• Vercel · Vercel GLM 5.3 Flash — Pay per use. 1 request, 3,025 tokens. Cost: $0.000712 (rounded), charged by Vercel.`
- `Other conversations' cost: $0.001542 (rounded), charged by OpenRouter and Vercel.`

Expected-result outcomes:
- Passed: no notice appeared and no `Reopen warm-up` line was observed.
- Passed: the ACC-27 conversation total remains 1 request / 820 tokens / no price, identical to the ACC-28 observation. Other conversations remain the same three observed in ACC-28.

**VERDICT ACC-29: passed**

### ACC-37 — no-charge fallback (part 1 of 5: OpenRouter MiMo)

**Starting-state note:** ACC-28 figures were recorded above before no-charge mode. `agms-tui standin no-charge on` was run at `2026-10-16T05:51:46Z`; the harness confirmed that OpenRouter, Vercel, and Corbanu API would leave out their charge.

**Requested actions:** stop; start a new conversation on `openrouter xiaomi/mimo-v2.6-pro`; type `Reply with the single word OK.`, press Enter, wait; run `/cost`; open each request and technical page.

**Actual actions:**
- `2026-10-16T05:51:55Z` prompt entered and observed before Enter; Enter sent.
- `2026-10-16T05:52:02Z` `/cost` opened; reply visible `• OK`.
- Opened Request 1 and technical attempt 1 via on-screen links; closed with Esc.

Exact lines observed:
- `• OpenRouter · OpenRouter MiMo V2.6 Pro — Pay per use. 1 request, 1,230 tokens. Estimated cost: no price available.`
- Request: `Estimated cost: no price available`
- Request: `Price basis: none — OpenRouter reported no charge, and no published price covers this response`
- Request: `Next step for requests with no price: check the bill from OpenRouter. No published price covers them, so no cost is shown for them here.`
- Request: `Tokens: 1,230 tokens`
- Technical: `Input: 1200`
- Technical: `Output: 30`
- Technical: `Total (not separately billed): 1230`
- Technical: `Output cost: unknown — rate unavailable`
- Technical: `Known subtotal exact USD: none — no price for these attempts`
- Technical: `Price: none recorded — this request records only its billing basis: pay per use`

Partial outcome:
- Passed: no `charged by` wording or reported-charge price basis was observed for this request.
- Passed: branch (c) applies: no money figure, with a stated reason.
- Passed: tokens are 1,200 input and 30 output.
- Passed: no `$0` stands in for a missing price; the subtotal is `none`.

### ACC-37 part 2 of 5 — OpenRouter `standin/uncatalogued`

**Actual actions:** `2026-10-16T05:53:07Z` prompt entered and observed before Enter; Enter sent. `2026-10-16T05:53:14Z` `/cost` opened. Request 1 and technical attempt 1 were opened via on-screen links, then closed with Esc. The product warned `⚠ Model metadata for \`standin/uncatalogued\` not found. Defaulting to fallback metadata; this can degrade performance and cause issues.`

Exact lines observed:
- `• OpenRouter · standin/uncatalogued — Pay per use. 1 request, 915 tokens. Estimated cost: no price available.`
- Request: `Estimated cost: no price available`
- Request: `Price basis: none — OpenRouter reported no charge, and no published price covers this response`
- Request: `Next step for requests with no price: check the bill from OpenRouter. No published price covers them, so no cost is shown for them here.`
- Request: `Tokens: 915 tokens`
- Technical: `Input: 900`
- Technical: `Output: 15`
- Technical: `Total (not separately billed): 915`
- Technical: `Output cost: unknown — rate unavailable`
- Technical: `Known subtotal exact USD: none — no price for these attempts`
- Technical: `Price: none recorded — this request records only its billing basis: pay per use`

Partial outcome:
- Passed: `standin/uncatalogued` shows branch (c), no money and a stated reason.
- Passed: no `charged by` wording or reported-charge basis was observed.
- Passed: tokens are 900 input and 15 output.
- Passed: no `$0` stands in for a missing price.

### ACC-37 part 3 of 5 — OpenRouter `z-ai/glm-5.3-flash` with no charge

**Actual actions:** `2026-10-16T05:54:11Z` prompt entered and observed before Enter; Enter sent. `2026-10-16T05:54:20Z` `/cost` opened. Request 1 and technical attempt 1 were opened via on-screen links, then closed with Esc.

Exact lines observed:
- `• OpenRouter · OpenRouter GLM 5.3 Flash — Pay per use. 1 request, 2,040 tokens. Estimated cost: $0.000270.`
- Open-link area: `→ OpenRouter · OpenRouter GLM 5.3 Flash — estimated $0.000270 (1 request)`
- Request: `Estimated cost: $0.000270`
- Request: `Price basis: estimate from published prices — OpenRouter reported no charge for this response`
- Request: `Tokens: 2,040 tokens`
- Technical: `Estimated token cost for recorded attempts: $0.000270`
- Technical: `Noncached input (derived for inclusive input): 1500 (derived)`
- Technical: `Cache read: 500`
- Technical: `Cache write: 0 (reported)`
- Technical: `Output: 40`
- Technical: `Reasoning (subset, not separately billed): 12`
- Technical: `Total (not separately billed): 2040`
- Technical: `Noncached input cost: $0.000225; exact USD 0.000225`
- Technical: `Cache read cost: $0.000025; exact USD 0.000025`
- Technical: `Cache write cost: none — 0 tokens`
- Technical: `Output cost: $0.000020; exact USD 0.00002`
- Technical: `Known subtotal exact USD: 0.00027`
- Technical: `Noncached input rate: 0.15 USD per million tokens`
- Technical: `Cache read rate: 0.05 USD per million tokens`
- Technical: `Rate unavailable for Cache write`
- Technical: `Output rate: 0.5 USD per million tokens`
- Technical JSON: `{"input":2000,"read":500,"write":0,"output":40,"reasoning":12,"total":2040}`

Recomputation:
- `1,500 × 0.15 / 1,000,000 = 0.000225`
- `500 × 0.05 / 1,000,000 = 0.000025`
- cache write 0 with no rate = no money
- `40 × 0.5 / 1,000,000 = 0.00002`
- Sum `0.000225 + 0.000025 + 0.00002 = 0.00027`

Partial outcome:
- Passed: branch (a) applies: a cost is labelled an estimate from published prices, and the technical-page rates × tokens equal the exact USD subtotal.
- Passed: no `charged by` wording or reported-charge price basis was observed.
- Passed: tokens match ACC-23 (`input` 2000, cache read 500, noncached 1500, output 40, reasoning 12, total 2040).
- Passed: no `$0` stands in for a missing price.

### ACC-37 part 4 of 5 — Vercel `zai/glm-5.3-flash` with no charge

**Actual actions:** `2026-10-16T06:05:33Z` prompt entered and observed before Enter; Enter sent. `2026-10-16T06:05:39Z` `/cost` opened. Request 1 and technical attempt 1 were opened via on-screen links, then closed with Esc.

Exact lines observed:
- `• Vercel · Vercel GLM 5.3 Flash — Pay per use. 1 request, 3,025 tokens. Estimated cost: at least $0.000042 (rounded) (1 attempt had incomplete usage).`
- Open-link area: `→ Vercel · Vercel GLM 5.3 Flash — estimated at least $0.000042 (rounded) (1 attempt had incomplete usage) (1 request)`
- Request: `Estimated cost: at least $0.000042 (rounded) (1 attempt had incomplete usage)`
- Request: `Price basis: partial estimate from published prices, a lower bound — Vercel reported no charge for this response`
- Request: `Next step for requests with incomplete usage: check the bill from Vercel. Their provider did not report every token count, so the estimate is missing or only a lower bound.`
- Request: `Tokens: 3,025 tokens`
- Technical: `Full recorded estimate: unavailable (1 of 1 billed attempts incomplete)`
- Technical: `Input: 3000`
- Technical: `Noncached input (derived for inclusive input): unknown — no retained numeric evidence`
- Technical: `Cache read: 1000`
- Technical: `Cache write: unknown — no retained numeric evidence`
- Technical: `Output: 25`
- Technical: `Reasoning (subset, not separately billed): 0 (reported)`
- Technical: `Total (not separately billed): 3025`
- Technical: `Cache read cost: $0.000030; exact USD 0.00003`
- Technical: `Output cost: $0.000012 (rounded); exact USD 0.0000125`
- Technical: `Known subtotal exact USD: 0.0000425`
- Technical: `Noncached input rate: 0.15 USD per million tokens`
- Technical: `Cache read rate: 0.03 USD per million tokens`
- Technical: `Rate unavailable for Cache write`
- Technical: `Output rate: 0.5 USD per million tokens`

Recomputation:
- `1,000 × 0.03 / 1,000,000 = 0.00003`
- `25 × 0.5 / 1,000,000 = 0.0000125`
- Noncached input rate is available but the count is not retained, so its cost is unknown.
- Known subtotal `0.00003 + 0.0000125 = 0.0000425`, displayed on the first screen as `at least $0.000042 (rounded)`.

Partial outcome:
- Passed: branch (b) applies: the estimate is marked as a lower bound (`at least …`) and the missing token count is identified as incomplete usage / noncached input / cache write.
- Passed: no `charged by` wording or reported-charge price basis was observed.
- Passed: tokens match ACC-25 for the reported counts (`input` 3000, cache read 1000, output 25, total 3025).
- Passed: no `$0` stands in for a missing price; the total is marked incomplete.

### ACC-37 part 5 of 5 — Corbanu API with no charge

**Requested actions:** stop; start a new conversation on `corbanu corbanu/deepseek-v4.1-flash`; perform the same prompt and `/cost` navigation; then run `agms-tui standin no-charge off`.

**Actual actions / observation:**
- `2026-10-16T06:06:42Z` and `2026-10-16T06:06:50Z`, `agms-tui start --route corbanu --model corbanu/deepseek-v4.1-flash` failed with `tmux error: no server running on /Users/agmx2/runs/acc-E-agmx2-20261016T052736Z/tmux.sock`. No product screen appeared and no turn was sent.
- `2026-10-16T06:06:??Z` (immediately after those attempts) `agms-tui standin no-charge off` was run as required; the harness confirmed charging stand-ins report their charge again.

Partial outcome:
- Not exercised: the Corbanu no-charge conversation could not be observed because the required route session could not be started.
- Because one of the five requested conversations was not exercised, the complete ACC-37 comparison cannot be recorded as passed.

**Overall ACC-37 outcome: blocked** because one requested conversation could not start; the four observable conversations passed the applicable no-charge checks.

**VERDICT ACC-37: blocked**

### ACC-34 — no credential is printed (group E scope only)

**Requested actions (with amendment 21):** scan every captured screen, all exec output, and the workspace files for `agms-synthetic`, `sk-`, `Bearer `, `Authorization`, `api_key`, and `token=`.

**Actual actions:**
- At `2026-10-16T06:07:33Z`, stopped the product and scanned the group-E screen/action log files and the five workspace files.
- Files scanned:
  - `evidence/screens.jsonl` (captured screens)
  - `evidence/actions.jsonl` (captured actions/output)
  - `evidence/pty.raw` (captured terminal output)
  - `fixture/workspace/.env`
  - `fixture/workspace/.gitignore`
  - `fixture/workspace/README.md`
  - `fixture/workspace/notes.txt`
  - `fixture/workspace/src/notes.py`
- There were no `agms-tui exec` commands in this group's assigned cases, so no additional exec output existed.

Scan result:
- `evidence/screens.jsonl`: no pattern hits.
- `evidence/actions.jsonl`: no pattern hits.
- `evidence/pty.raw`: no pattern hits.
- All five workspace files: no pattern hits.
- No match contents were printed or copied into this report.

Expected-result outcomes:
- Passed: none of the requested patterns was observed in the group-E captured screens/output or workspace files.
- Not exercised: real-route exec output, because no exec action was assigned in this group.

**VERDICT ACC-34: passed**

## Final case verdicts

- `VERDICT ACC-23: passed`
- `VERDICT ACC-24: passed`
- `VERDICT ACC-25: passed`
- `VERDICT ACC-26: blocked`
- `VERDICT ACC-27: passed`
- `VERDICT ACC-28: blocked`
- `VERDICT ACC-29: passed`
- `VERDICT ACC-37: blocked`
- `VERDICT ACC-34: passed`

This report records functional observations only and does not claim human acceptance.
