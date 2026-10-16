# #368: price catalogue refresh (read 2026-10-14)

Data upkeep after #361 (PF-60-S07 items 2–5). No accounting behaviour changes. Readings are in [data/](data/).

| Item | Source (read 2026-10-14 UTC) | Result |
| --- | --- | --- |
| 3. `gpt-6.1-sol` | developers.openai.com/api/docs/pricing; developers.openai.com/api/docs/models/gpt-6.1-sol ([data](data/openai-pricing-20261014.txt)) | **Added.** $2.00 input / $0.10 cached (5%) / $2.50 cache writes (1.25x) / $10.00 output; above 272K input 2x input and cache rates, 1.5x output. 1,050,000 context, 128,000 output. Manual selection only (like GPT-6 Astra and Luna): listed in `/model`, priced for accounting, never auto-allocated. |
| 4. GPT-5.6 Sol promotion | same pricing page and developers.openai.com/api/docs/models/gpt-5.6-sol | **Unchanged.** $4 / $0.40 / $5 / $20, "available at least through November 21, 2026". OpenAI publishes no post-promotion price, so the row keeps "no API price after 2026-11-21T23:59:59.999Z". A test pins the boundary: the last millisecond is priced, the next has no API-key price, and a ChatGPT-login turn keeps its plan burn with no API equivalent. |
| 5. DeepSeek holidays | api-docs.deepseek.com/quick_start/pricing ([data](data/deepseek-pricing-20261014.txt)) | **Not published yet.** Prices unchanged. DeepSeek bills "Chinese public holidays" off-peak but lists no dates; the dates come from the State Council's yearly notice (2026: 国办发明电〔2025〕7号, www.gov.cn/zhengce/content/202511/content_7047090.htm). No 2027 notice exists yet (usually November or December), so no dates were added. `scripts/check_price_calendars.py` reads each list as covering through 31 December of its last year: PR CI warns, and nightly-ci fails from 30 days before (2026-12-01 for the current list). |
| 2. Moonshot / BigModel | platform.kimi.ai/docs/pricing/chat ([data](data/moonshot-pricing-20261014.txt)); open.bigmodel.cn/pricing ([data](data/bigmodel-pricing-20261014.txt)) | **Read, not added** (see below). |

## Why item 2 has no rows

Both pages were read, but a catalogue row cannot price either route without new accounting behaviour:

- **Custom routes are never priced per token.** These routes are reached through a user-defined provider
  (`model_providers.<any-id>` with `base_url = https://api.moonshot.ai/v1` or `https://open.bigmodel.cn/api/paas/v4`).
  Accounting applies catalogue rates only to a built-in provider at its own built-in route (`own_route` in
  `core/src/accounting.rs`), and matches a row by provider id, which the user names freely. A row would never be used.
- **BigModel bills in CNY.** The ledger has one currency (USD); a converted rate would be invented.
- **BigModel's slugs (`glm-5.3`, `glm-5.2`, `glm-5.3-flash`) are already Z.AI's rows.** A price lookup requires a unique
  slug, so a second row would remove Z.AI's prices.
- **K3 cache writes depend on the TTL** ($3.00 for 5 min, the default; $6.00 for 1 h). A row states one cache-write rate.
- BigModel's GLM-5.3-Flash also shows a half-price "limited time" offer with no end date.

Pricing these routes needs a route-keyed price lookup (and CNY for BigModel). That is new accounting behaviour, so it is left
for the PF-60 follow-up sprints with OpenRouter's per-endpoint pricing (item 1).

**Update, later on 2026-10-14 (PF-60-S07, Travis's decision):** Moonshot is now priced by route, with K3 writes at
the five-minute rate this client's requests get; BigModel stays deferred. See [the S07 gate](../pf-60-s07/gate.md).

**Update, 2026-10-15 (real key):** the first real K3 and K2.7 Code requests showed Moonshot leaves out zero cache
counts, so `/cost` showed only a lower bound; fixed in #859. See [moonshot-real-20261015](moonshot-real-20261015/README.md).

## Checks

- `python3 scripts/check_openai_api_prices.py` against the 2026-10-14 page: every catalogue row matches, `gpt-6.1-sol` included.
- `python3 scripts/check_price_calendars.py`: DeepSeek lists cover through 2026-12-31; Sol's rates hold through
  2026-11-21T23:59:59.999Z (warns from 2026-10-22).
