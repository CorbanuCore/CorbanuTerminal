# Group E2 functional execution — in progress

Executor role only. No human-acceptance claim. All times UTC.

## Preflight and identity

- Coordinator preflight files present and read:
  - `evidence/preflight-summary.json`: `"ok": true`, `"failed": []`, `probe_count: 105`.
  - `evidence/preflight-launcher.json`: `"ok": true`.
  - `evidence/preflight-tmux-child.json`: `"ok": true`.
- Executor preflight command: `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=0`; saved JSON has `"ok": true`.
- Candidate: `/opt/agms/pkg/bin/corbanu`; observed SHA-256 `5b805b9c244e55c12c6f61fc3d31bddd760a415554cbabff8cbcb4c3ba0531a1` (matches expected). Program reports Corbanu Terminal 0.1.48 in preflight.
- Packet identities observed: design `f750b006046cb5036f92fd6f4f8d910ccd8aa33f88b33c72ffefd0f6cc01bfd6`, amendments `2a73b6503429dc5a6d989b313e996a9041f5dc60b2c130e329dd39e1d71f105e`, answers `dc4044d36ac3868205ff2dc71cee36805969c683852a6f9d705dc2d7c0c`.
- Fixture started as supplied fresh project: `$AGMS_RUN/fixture/workspace` (ledger-notes), clean at commit `42741cc initial notes`.

Navigation notes: `/cost` selection is bottom-linked and can cycle when Down reaches the end. I preserved each ambiguous navigation in screen labels and reopened pages to reach the named request/technical page.

## ACC-23 — OpenRouter reported charge

### Requested vs actual

- Requested: fresh profile; route openrouter; model `z-ai/glm-5.3-flash`; new conversation; run `agms-tui standin no-charge off` first; send one prompt; `/cost`; open each request and technical page.
- Actual: `agms-tui standin no-charge off` returned 200 and set charging stand-ins to report charges. Started fresh product profile at route/model; waited for composer; screen `ACC23-ready`. Sent literal prompt text then Enter separately (typed screen showed exact prompt before Enter). Received `• OK`. Opened `/cost`, request page, and technical attempt page.
- Turn sent UTC 07:27:01; Enter UTC 07:27:03; `/cost` opened UTC 07:27:08.

### Positive visible checkpoints

- `ACC23-cost-first`: `Cost: $0.000520, charged by OpenRouter.`; 1 request; 2,040 tokens; explanatory line: `Costs marked "charged by OpenRouter" are the charges OpenRouter reported with those responses.`
- `ACC23-request-page-final`: `Cost: $0.000520, charged by OpenRouter`; `Price basis: the charge OpenRouter reported with the response (usage.cost)`; `Tokens: 2,040 tokens`.
- `ACC23-technical-down30`: tokens `Noncached input (derived for inclusive input): 1500 (derived)`, `Cache read: 500`, `Cache write: 0 (reported)`, `Output: 40`, `Reasoning ... 12`, `Total ... 2040`.
- `ACC23-technical-down45`: rates noncached input 0.15, cache read 0.05, output 0.5 USD/M; cache write rate unavailable.
- `ACC23-tech-down30-up15`: `Published-price estimate (not used — OpenRouter reported its charge): $0.000270`; `Known subtotal exact USD: 0.00052`.
- Component arithmetic: `1500×0.15/M + 500×0.05/M + 40×0.5/M = 0.000225 + 0.000025 + 0.000020 = 0.000270` (published estimate); known reported subtotal `0.000520`.

### Expected-result outcomes

- Passed: each observed request cost `$0.000520, charged by OpenRouter`.
- Passed: price basis names `usage.cost` (contains `.cost`).
- Passed: observed token counts as expected.
- Passed: conversation total known subtotal `0.00052 = 1 × 0.00052`.
- Passed: published catalogue figure `$0.000270` is explicitly labelled `(not used — OpenRouter reported its charge)`.

## ACC-24 — OpenRouter reasoning retained as not reported/unknown

### Requested vs actual

- Requested: same profile; new conversation on `deepseek/deepseek-v4.1-flash`; same prompt and `/cost` inspection.
- Actual: stopped prior session; started new conversation on same profile; sent exact prompt text then Enter separately. Received `• OK`; opened `/cost`, request, and technical pages.
- Turn sent UTC 07:33:22; Enter UTC 07:33:23; `/cost` opened UTC 07:33:28.

### Positive visible checkpoints

- `ACC24-cost-first`: `OpenRouter · OpenRouter DeepSeek V4.1 Flash — Pay per use. 1 request, 1,040 tokens. Cost: $0.000310, charged by OpenRouter.`
- `ACC24-cost-first`: prior ACC-23 conversation listed as `1 request in 1 conversation` and `Other conversations' cost: $0.000520, charged by OpenRouter.`
- `ACC24-request-page`: `Cost: $0.000310, charged by OpenRouter`; `Price basis ... (usage.cost)`; `Tokens: 1,040 tokens`.
- `ACC24-tech-input2`: `Noncached input ... 1000`, `Cache read: 0`, `Cache write: 0`, `Output: 40`, `Reasoning ... unknown — no retained numeric evidence`, `Total ... 1040`.
- `ACC24-tech-down30-recheck`: `Known subtotal exact USD: 0.00031`.
- Arithmetic: `0.00031 = 1 × 0.00031`.

### Expected-result outcomes

- Passed: every request observed was present; 1 request, none dropped.
- Passed: reasoning is `unknown — no retained numeric evidence`, not 60 or 0.
- Passed: input 1,000; output 40.
- Passed: cost `$0.000310, charged by OpenRouter`; total `0.00031`.
- Passed: technical page explicitly labels published estimate unused (`Published-price estimate (not used — OpenRouter reported its charge): none — no published price covers every component`).

## ACC-25 — Vercel gatewayCost

### Requested vs actual

- Requested: same profile; route vercel; model `zai/glm-5.3-flash`; new conversation; same prompt and `/cost` inspection.
- Actual: stopped prior session; started new conversation on same profile; sent exact prompt text then Enter separately. Received `• OK`; opened `/cost`, request, and technical pages.
- Turn sent UTC 07:36:14; Enter UTC 07:36:15; `/cost` opened UTC 07:36:24.

### Positive visible checkpoints

- `ACC25-cost-first`: `Vercel · Vercel GLM 5.3 Flash — Pay per use. 1 request, 3,025 tokens. Cost: $0.000712 (rounded), charged by Vercel.`
- `ACC25-request-page`: `Cost: $0.000712 (rounded), charged by Vercel`; `Price basis ... (gatewayCost)`; `Tokens: 3,025 tokens`.
- `ACC25-tech-original-price`: `Input: 3000`; `Cache read: 1000`; `Noncached input ... unknown`; `Cache write ... unknown`; `Output: 25`; `Total ... 3025`.
- `ACC25-tech-components3`: `Output cost: $0.000012 (rounded); exact USD 0.0000125`; `Cache read cost: $0.000030; exact USD 0.00003`; `Cache write cost: unknown`.
- `ACC25-tech-down40`: `Known subtotal exact USD: 0.0007125`.
- `ACC25-tech-down-more`: original usage object ends `..."total":3025,"billed_usd":"0.0007125"`.
- Arithmetic: reported charge exact `0.0007125 = 1 × 0.0007125`. Partial component check with observed rates: `1000×0.03/M + 25×0.5/M = 0.00003 + 0.0000125 = 0.0000425`; input/cache-write components are explicitly unknown, so the product does not present that partial arithmetic as the full charge.

### Expected-result outcomes

- Passed: observed charge is the Vercel reported charge; first/request pages show rounded $0.000712 and the technical page records exact USD 0.0007125, satisfying the exact-amount requirement.
- Passed: price basis names `gatewayCost`.
- Passed: input 3,000, cache read 1,000, output 25, total 3,025; noncached input is explicitly unknown (amendment permits unknown).
- Passed: total known subtotal `0.0007125`.
- Passed: published-price figure is not used; technical page labels the unavailable estimate `not used — Vercel reported its charge`.

## ACC-26 — Corbanu API charge field

### Requested vs actual

- Requested: same profile; route corbanu; model `corbanu/deepseek-v4.1-flash`; new conversation; same prompt and `/cost` inspection.
- Actual: stopped prior session; started new conversation on same profile; sent exact prompt text then Enter separately. Received `• OK`; opened `/cost`, request, and technical pages.
- Turn sent UTC 07:39:13; Enter UTC 07:39:14; `/cost` opened UTC 07:39:20.

### Positive visible checkpoints

- `ACC26-cost-first`: `Corbanu API · DeepSeek V4.1 Flash — Pay per use. 1 request, 1,530 tokens. Cost: $0.001850, charged by Corbanu API.`
- `ACC26-request-page2`: `Cost: $0.001850, charged by Corbanu API`; `Price basis ... (usage.corbanu_charge_usd or usage.cost)`; `Tokens: 1,530 tokens`.
- `ACC26-tech-input3`: `Input: 1500`, `Output: 30`, `Total ... 1530`; cache read/write and reasoning are explicitly unknown.
- `ACC26-tech-up15`: `Known subtotal exact USD: 0.00185`; `Published-price estimate (not used — Corbanu API reported its charge): not available ...`.
- `ACC26-tech-original-json`: original usage object is `{"input":1500,"output":30,"total":1530,"billed_usd":"0.00185"}`.
- Arithmetic: `0.00185 = 1 × 0.00185`.

### Expected-result outcomes

- Passed: cost `$0.001850, charged by Corbanu API`.
- Passed: price basis names `usage.corbanu_charge_usd` (shown as `usage.corbanu_charge_usd or usage.cost`).
- Passed: input 1,500, output 30, total 1,530.
- Passed: total `0.00185`.
- Passed: any published-price figure is not used; the estimate is explicitly unavailable and labelled unused.

## ACC-27 — configured pay-per-use provider with no price

### Requested vs actual

- Requested: same profile; route custom (`mylab`); model `lab-model-1`; new conversation; same prompt and `/cost` inspection.
- Actual: stopped prior session; started new conversation on same profile; sent exact prompt text then Enter separately. Received `• OK`; opened `/cost`, request, and technical pages.
- Turn sent UTC 07:42:16; Enter UTC 07:42:18; `/cost` opened UTC 07:42:24.
- Ambiguity preserved: the session itself showed `⚠ Model metadata for `lab-model-1` not found. Defaulting to fallback metadata; this can degrade performance and cause issues.` This was not one of the stated expected-result items; I did not infer either way from it.

### Positive visible checkpoints

- `ACC27-cost-first`: `mylab · lab-model-1 — Pay per use. 1 request, 820 tokens. Estimated cost: no price available.`
- `ACC27-cost-first`: other conversations cost displayed as `$0.003392 (rounded)`, not treating mylab as $0.
- `ACC27-request-page`: `Estimated cost: no price available`; next step: `check the bill from mylab. No published price covers them, so no cost is shown for them here.`
- `ACC27-tech-down25`: `Billing basis: pay per use (set in your config)`; `Input: 800`.
- `ACC27-tech-price-line5`: `Output: 20`; `Total ... 820`; `Noncached input cost ... unknown`; `Cache read cost ... unknown`; `Cache write cost ... unknown`; `Output cost ... unknown`; `Known subtotal exact USD: none — no price for these attempts`; `Price: none recorded — this request records only its billing basis: pay per use; set in your config`.
- Arithmetic: no dollar component figure; no subtotal.

### Expected-result outcomes

- Passed: first screen has `no price available` and next step to check mylab's bill.
- Passed: technical page reads exactly the required billing-basis wording.
- Passed: technical page reads exactly the required price wording, with no nested parentheses.
- Passed: input 800; output 20.
- Passed: every cost component observed is `unknown — ...`; no `$` component figure appears.
- Passed: `Known subtotal exact USD: none`.

## ACC-28 — several charging providers on first screen

### Requested vs actual

- Requested: from ACC-27 conversation, read `/cost` first screen.
- Actual: returned to composer with Escape, re-opened `/cost`, and captured first screen at UTC 07:44:40.

### Positive visible checkpoints

- `ACC28-cost-first`: `Other conversations on this day, not included above: 4 requests in 4 conversations.`
- Provider/model lines copied exactly:
  - `OpenRouter · OpenRouter DeepSeek V4.1 Flash — Pay per use. 1 request, 1,040 tokens. Cost: $0.000310, charged by OpenRouter.`
  - `OpenRouter · OpenRouter GLM 5.3 Flash — Pay per use. 1 request, 2,040 tokens. Cost: $0.000520, charged by OpenRouter.`
  - `Corbanu API · DeepSeek V4.1 Flash — Pay per use. 1 request, 1,530 tokens. Cost: $0.001850, charged by Corbanu API.`
  - `Vercel · Vercel GLM 5.3 Flash — Pay per use. 1 request, 3,025 tokens. Cost: $0.000712 (rounded), charged by Vercel.`
- `ACC28-cost-first`: `Other conversations' cost: $0.003392 (rounded), charged by OpenRouter, Corbanu API and Vercel.`
- Recomputation from exact figures: `0.00031 + 0.00052 + 0.00185 + 0.0007125 = 0.0033925`, displayed rounded as `$0.003392`.
- mylab appears as `Estimated cost: no price available`; no `$0` appears for it.

### Expected-result outcomes

- Passed: four other conversations with each provider/model and its own charged-by provider.
- Passed: explanatory aggregate names all three charging provider labels.
- Passed: other-conversations cost recomputes to the sum of ACC-23–26 charges (`0.0033925`; display is explicitly rounded to `$0.003392`).
- Passed: mylab is never counted as $0; no displayed total includes it.

## ACC-29 — no warm-up on non-WebSocket reopen

### Requested vs actual

- Requested: stop; start with `resume --last` on custom route; run `/cost` before typing.
- Actual: stopped; started with `--route custom --model lab-model-1 resume --last`; restored transcript showed only `Reply with the single word OK.` and `• OK`; then opened `/cost` before typing at UTC 07:45:21.

### Positive visible checkpoints

- `ACC29-restored`: no warm-up notice or `Reopen warm-up` text.
- `ACC29-cost-first`: exactly same visible current conversation line as ACC-28: `mylab · lab-model-1 — Pay per use. 1 request, 820 tokens. Estimated cost: no price available.`
- Other-conversation aggregate remained `4 requests in 4 conversations` and `$0.003392 (rounded)`.

### Expected-result outcomes

- Passed: no notice and no `Reopen warm-up` line.
- Passed: visible totals identical to before.

## ACC-37 — no-charge fallback across five conversations

### Requested vs actual

- Requested: record ACC-28 other-conversation figures, set `agms-tui standin no-charge on`, then for each listed provider/model stop, start a new conversation, send the exact prompt, `/cost`, open request and technical pages; finally set `no-charge off`.
- Actual: `no-charge on` returned 200 at UTC 07:45:50. Executed all five conversations in the amended order. `no-charge off` returned 200 at UTC 07:58:04. Turn and `/cost` times:
  - MiMo: turn 07:45:50/07:45:51; `/cost` 07:45:53.
  - uncatalogued: turn 07:47:47/07:47:49; `/cost` 07:47:51.
  - GLM: turn 07:51:14/07:51:15; `/cost` 07:51:17.
  - Vercel: turn 07:54:08/07:54:10; `/cost` 07:54:12.
  - Corbanu: turn 07:56:23/07:56:25; `/cost` 07:56:27.
- Each prompt text and Enter were separate actions. Each screen showed the exact prompt before Enter. Each model replied `• OK`.

### Per-conversation first-screen and request-page observations

1. `ACC37-MIMO-cost-first`:
   `OpenRouter · OpenRouter MiMo V2.6 Pro — Pay per use. 1 request, 1,230 tokens. Estimated cost: no price available.`
   `ACC37-MIMO-request`: `Price basis: none — OpenRouter reported no charge, and no published price covers this response`. Technical tokens: input 1,200, cache write 0, output 30, total 1,230. `ACC37-MIMO-tech-down40`: `Known subtotal exact USD: none`; `Price: none recorded — this request records only its billing basis: pay per use`.
2. `ACC37-UNC-cost-first`:
   `OpenRouter · standin/uncatalogued — Pay per use. 1 request, 915 tokens. Estimated cost: no price available.`
   `ACC37-UNC-request-final`: same `Price basis: none ...` form. Technical tokens: input 900, cache write 0, output 15, total 915. `ACC37-UNC-tech-down40`: `Known subtotal exact USD: none`; `Price: none recorded ...`.
3. `ACC37-GLM-cost-first`:
   `OpenRouter · OpenRouter GLM 5.3 Flash — Pay per use. 1 request, 2,040 tokens. Estimated cost: $0.000270.`
   `ACC37-GLM-request4`: `Estimated cost: $0.000270`; `Price basis: estimate from published prices — OpenRouter reported no charge for this response`. Technical: input 2,000, noncached 1,500, cache read 500, cache write 0, output 40, reasoning 12, total 2,040; rates 0.15, 0.05, unavailable-for-cache-write, 0.5 per M. Component sum: `1500×0.15/M + 500×0.05/M + 40×0.5/M = 0.000225 + 0.000025 + 0.000020 = 0.000270`. Known subtotal exact USD `0.00027`.
4. `ACC37-VER-cost-first`:
   `Vercel · Vercel GLM 5.3 Flash — Pay per use. 1 request, 3,025 tokens. Estimated cost: at least $0.000042 (rounded) (1 attempt had incomplete usage).`
   `ACC37-VER-request`: `Price basis: partial estimate from published prices, a lower bound — Vercel reported no charge for this response`; next step says provider did not report every token count. Technical: input 3,000, cache read 1,000, noncached unknown, cache-write unknown, output 25, total 3,025; rates noncached 0.15, cache read 0.03, output 0.5 per M. Partial sum: `1000×0.03/M + 25×0.5/M = 0.00003 + 0.0000125 = 0.0000425`, shown exact as `0.0000425`.
5. `ACC37-COR-cost-first`:
   `Corbanu API · DeepSeek V4.1 Flash — Pay per use. 1 request, 1,530 tokens. Estimated cost: at least $0.000020 (rounded) (1 attempt had incomplete usage).`
   `ACC37-COR-request`: `Price basis: partial estimate from published prices, a lower bound — Corbanu API reported no charge for this response`; next step says provider did not report every token count. Technical: input 1,500, output 30, total 1,530; noncached/cache read/cache write/reasoning unknown; rates noncached 0.22, cache read 0.007, cache-write unavailable, output 0.66 per M. Partial sum: `30×0.66/M = 0.0000198`, shown exact as `0.0000198`.

### Expected-result outcomes

- Passed: none of these five requests reads `charged by` or uses a reported-charge price basis; all use either no-price or published-price estimate wording.
- Passed: exactly one branch per request: GLM is (a); Vercel and Corbanu API are (b); MiMo and `standin/uncatalogued` are (c).
- Passed: `standin/uncatalogued` shows branch (c).
- Passed: tokens match the stand-in reports listed above.
- Passed: no `$0` figure stood in for a missing price; absent component estimates were `unknown`, `none`, or unavailable.
- Passed: incomplete lower-bound conversations state `1 attempt had incomplete usage` / `Full recorded estimate: unavailable (1 of 1 billed attempts incomplete)`; GLM's complete estimate sums exactly.
- Passed: ACC-23–26 charged amounts remained unchanged on ACC-37 first screens: OpenRouter DeepSeek `$0.000310`, OpenRouter GLM `$0.000520`, Corbanu API `$0.001850`, Vercel `$0.000712 (rounded)`/exact technical `0.0007125`.
- Passed: the catalogue-estimate branch was observed, so the permitted `catalogue-estimate branch not observed` limit is not needed.

## ACC-34 — advisory credential pattern scan

### Requested vs actual

- Requested: scan every captured screen, all exec output, and workspace files for `agms-synthetic`, `sk-`, `Bearer `, `Authorization`, `api_key`, and `token=`.
- Actual: no `agms-tui exec` output occurred in this group. I scanned the captured-screen excerpts recorded in this report, plus all non-`.git` workspace files. The scan command was:
  `grep -RInE -I --exclude-dir=.git 'agms-synthetic|sk-|Bearer |Authorization|api_key|token=' $AGMS_RUN/fixture/workspace`.
- Hit observed:
  - Location: `workspace/.env`.
  - Content line: `NOTES_SYNC_TOKEN=agms-canary-7f3a9c-synthetic-token`.
  - Matching pattern: `token=`.
- No hit for `agms-synthetic`, `sk-`, `Bearer `, `Authorization`, or `api_key` in captured screens or workspace files.

### Expected-result outcome

- Failed: the amended expected result says no hit, but one `token=` hit occurred in the supplied workspace `.env`. The hit is reported with location as required; it is visibly labeled a synthetic test value, but I did not alter the expected result or delete it.

## Final case outcomes

- ACC-23: passed.
- ACC-24: passed.
- ACC-25: passed.
- ACC-26: passed.
- ACC-27: passed.
- ACC-28: passed.
- ACC-29: passed.
- ACC-37: passed.
- ACC-34: failed by the literal amended no-hit criterion because of the synthetic `token=` fixture hit.

No case is claimed as human acceptance. No product-model refusal or decline occurred. No credential or system-password prompt, unexpected profile access, missing required control, lost observation, or spend-limit refusal was observed. All assigned turns were stand-in turns; no real-route model turn was sent.

VERDICT ACC-23: passed
VERDICT ACC-24: passed
VERDICT ACC-25: passed
VERDICT ACC-26: passed
VERDICT ACC-27: passed
VERDICT ACC-28: passed
VERDICT ACC-29: passed
VERDICT ACC-37: passed
VERDICT ACC-34: failed
