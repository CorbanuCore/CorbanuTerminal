# #361: OpenAI API-key prices, cache writes and long-context tier (2026-10-09)

**Result.** A real `gpt-5.6-luna` request on the OpenAI API key (vault `openai-api-key`, developer-accounting build of
this branch, disposable home) shows `/cost` "Known subtotal exact USD: 0.0062586". The independent recompute from the
provider-reported usage at OpenAI's published Standard rates is **$0.0062586**: equal to the micro-dollar, with no
"had no price" attempt.

## Prices

- **Source.** https://developers.openai.com/api/docs/pricing (Standard table) and
  https://developers.openai.com/api/docs/models/<slug>, read 2026-10-09 ~19:35Z with curl
  ([data/openai-pricing-20261009.txt](data/openai-pricing-20261009.txt)).
- **Catalogue.** `codex-rs/models-manager/models.json` is hand-maintained, not generated: the upstream
  `rust-release-prepare` workflow that refreshes it is gated to `openai/codex` and its source (the ChatGPT models
  endpoint) carries no Corbanu billing. `scripts/check_openai_api_prices.py` now compares the catalogue with the page.
- **Rows.** Standard rates, cache writes where the sheet lists them, and the long-context tier where the model page
  states one; see `MODEL-ECONOMICS-DATA.md` for the table.
- **History.** Rates are bound to each attempt when it is admitted, so earlier records keep the rates they were
  priced at. A row that states a cache-write rate or a long-context tier gets a new price identity (`stated-v1`);
  other rows keep theirs.

## Real check (macOS)

| Step | Evidence |
| --- | --- |
| `corbanu exec -m gpt-5.6-luna "Reply with exactly the word: ok"` with `CODEX_API_KEY` from the vault on the command only | [data/exec-luna-t1.jsonl](data/exec-luna-t1.jsonl) |
| Provider usage, from the WebSocket frames (`tungstenite::protocol=trace`) | [data/provider-usage-luna-t1.jsonl](data/provider-usage-luna-t1.jsonl): prewarm 9,600 input; turn 17,331 input = 3 uncached + 17,328 cache writes, 5 output |
| Recompute at $0.20 / $0.02 / $0.25 cache writes / $1.20 | [data/recompute-luna-t1.txt](data/recompute-luna-t1.txt), [tools/recompute.py](tools/recompute.py): **$0.0062586** |
| `/cost` in the TUI (placeholder key, same home) | [captures/luna-t1-conv.txt](captures/luna-t1-conv.txt): exact USD 0.0062586; request 1 $0.001920, request 2 $0.004339 |
| Request 2 technical page | [captures/luna-t1-request2-technical.txt](captures/luna-t1-request2-technical.txt): rates 0.2 / 0.02 / 0.25 / 1.2, long-context tier 0.4 / 0.04 / 0.5 / 1.8 above 272,000 input tokens, "not applied: input 17331 ≤ 272000" |

**Safety.** `CORBANU_TEST_NO_NATIVE_KEYRING=1` and one disposable `CODEX_HOME`/`CORBANU_HOME`/`PFTERMINAL_HOME`.
The key was resolved by the installed wrapper only inside the consuming command; the TUI ran with a placeholder.
Logging was limited to `codex_api=trace,tungstenite::protocol=trace`. Key scan of the scratch directory and this
directory: 0 files with the key, 0 key-shaped strings.

## Other pay-per-use routes (same day)

| Route | Source | Result |
| --- | --- | --- |
| `zai` (pay-as-you-go API) | docs.z.ai/guides/overview/pricing | GLM-5.3-Flash 0.15 / 0.03 / 0.50, GLM-5.3 and GLM-5.2 1.40 / 0.26 / 4.40; cache storage "limited-time free": **match** |
| `deepseek` | api-docs.deepseek.com/quick_start/pricing | Flash 0.15 / 0.003 / 0.60 off-peak, 2x peak; V4-Pro 0.66 / 0.022 / 1.98 off-peak, 2x peak; windows 01-04 and 06-10 UTC Mon-Fri: **match**. Holiday dates are listed through 2026-10-07 only (follow-up). |
| `anthropic` (API key) | platform.claude.com/docs/en/about-claude/pricing | Opus 5.5 4 / 20 / 0.20, 5m writes 5; Opus 5 5 / 25 / 0.50, 6.25; Fable 5.1 10 / 50 / 0.25, 12.50; Fable 5 10 / 50 / 1, 12.50; no long-context surcharge for these: **match** |
| `openrouter` | openrouter.ai/api/v1/models and `/endpoints` | **Out of scope, follow-up.** The price depends on the upstream endpoint that served the request (e.g. `moonshotai/kimi-k3` input $0.65-$4.50 across endpoints), so one catalogue rate can't be right for every request. Listing mismatches: `x-ai/grok-4.7` 1.6 / 4.8 / 0.4 vs 2.0 / 6.0 / 0.5; `z-ai/glm-5.3-flash` cached 0.05 vs 0.03; `minimax/minimax-m3` 0.6 / 2.4 vs 0.3 / 1.2 / 0.06; `moonshotai/kimi-k3` 3 / 15 / 0.3 vs 0.9 / 14 / 0.6 (cheapest); `tencent/hy3:free` no longer listed. |
| Moonshot / Kimi Open Platform (`api.moonshot.ai/v1`), BigModel (`open.bigmodel.cn/api/paas/v4`) | platform.moonshot.ai/docs/pricing/chat; open.bigmodel.cn/pricing | No catalogue row prices these routes, so every attempt shows "no price" (never a wrong one). Adding them is a follow-up. |
