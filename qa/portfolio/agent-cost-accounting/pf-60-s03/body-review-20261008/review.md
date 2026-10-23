**Verdict: REQUEST CHANGES**

I reviewed origin/main (`63ea3d0cbd`) read-only and did not build or run tests. Line numbers are at origin/main.

### Findings

**1. Blocker: Kimi Code subscription work is recorded as money spent**
- **Where:** `core/src/accounting.rs:424-441` (the API-key branch of `turn_mode`), `core/src/accounting_prices.rs:53-99`. Introduced by `aae4ba660b`.
- **What happens:**
  - Every built-in provider that authenticates with an environment key now gets API-key pricing.
  - `kimi-code` points at `https://api.kimi.com/coding/v1`, authenticates with `KIMI_API_KEY`, and its catalogue row `k3` is listed as metered at $3 / $15 per million tokens.
  - So every Kimi Code turn becomes `Basis::Billed` spend, and `/cost` shows "Pay per use · Estimated cost: $X".
- **Why it's wrong:** Kimi's own documentation describes this base URL as Membership subscription (quota included), separate from the pay-as-you-go platform. It also says A membership does not fund API calls, and an API balance does not add membership quota. This breaks the rule that plan work is never stated as money spent.
- **Fix:**
  - Stop inferring "pay per use" from `AuthMode::ApiKey`. Have each provider declare its billing model (subscription key vs pay-as-you-go), or give `kimi-code` a plan row and route it to the plan basis.
  - Audit the other providers that use environment keys (`ambient`, `meta`) the same way.

**2. Major: plan work with no catalogue price loses its basis, and `/cost` calls it "Pay per use"**
- **Where:** `state/src/runtime/accounting_pricing.rs:283-289` (`is_plan` only reads the price record), `core/src/accounting_prices.rs:121-151` (`plan_original` returns nothing), `tui/src/chatwidget/tokens.rs:1178-1214`, `tokens/scope.rs:76-94`, `core/src/accounting_extensions.rs:155-162`. Commits `5bae03414e`, `8e952e2fe3`, `3b12174b50`, `bc01aef837`, `5c00d993d8`.
- **Root cause:** whether an attempt was plan work is stored only on its price record. When no price record exists, the attempt is treated as pay-per-use.
- **Who is affected:**
  - ChatGPT-plan turns on `gpt-5.5`, `gpt-5.4`, `gpt-6-astra`, `gpt-6-luna`, `gpt-5.3-codex` and `codex-auto-review`, which have no catalogue billing.
  - Plan turns on a non-default service tier, such as priority.
  - Image generation and realtime calls under a ChatGPT login.
  - Claude subscription work reported by the panes bridge, which is recorded with pricing `Unavailable`.
- **What the user sees:** "Billing: Pay per use · Estimated cost: no price available", plus "Next step: check the bill from …". This also inflates the pay-per-use "had no price" count on mixed days.
- **Fix:** whenever the turn's pricing is `Plan`, record a plan-basis price record even if it has no rates (all rates `None`, no burn). The arithmetic already handles this: no figure is invented and the attempt is classified as plan work. For pane and extension paths, carry the basis that was decided from the caller's authentication.

**3. Major: an unversioned change to stored bytes can stop accounting on older ledgers**
- **Where:** `state/src/runtime/accounting_pricing.rs:178-183` and `:262-273`, checked by `accounting_estimates.rs:290-293` ("corrupt estimate payload") and `:322-326` ("noncanonical snapshot"). Commit `5bae03414e`.
- **What happens:**
  - `5bae03414e` added `basis` and `plan_burn_millis` to price records, and `known_equivalent`, `all_buckets_equivalent`, `plan_burn_*` to estimates, all serialized unconditionally.
  - Any price record or estimate written before 2026-09-21 (the store has existed since `6b2dbcab56`, 09-12) no longer re-serializes to the same bytes.
  - The hourly whole-ledger validation and reads therefore fail. That closes accounting and stops every accounted turn on that state DB.
  - The rules versioning in `18eadb5413` only pins bytes from 09-22 onward.
- **The reverse also breaks:** `Patch` uses `deny_unknown_fields` and gained `billed_usd` (`0e0d692166`). An older debug build sharing the same state DB fails to read a newer ledger.
- **Fix:** mark the new fields to skip serialization when at their defaults, or treat payloads from before 5bae as an earlier rules version. Add a ledger-format gate that disables collection, rather than failing turns, when the format is unknown.

**4. Major: compaction and extension tools still fail on accounting errors, despite "best effort"**
- **Where:** `core/src/client.rs:1437-1450` (`resolve_path(...).await?`), `core/src/accounting_transport.rs:308-315` (admission), `:335-344`, `ext/web-search/src/tool.rs:158`. Commits `4e42eed5e7`, `4e1781fec2`, `2a3e9253a8`, `ad219caf84`.
- **What happens:**
  - `4e1781fec2` made attaching accounting best effort, but for Responses and Chat the attach step is deferred and never fails. The database open, admission and observation happen later and fail closed.
  - In the one-shot request path (`execute`), a failed observation after a successful, already-paid response returns `Err(FAILURE)` and throws the response away. Causes include contention past the 120 s budget, or a validation error such as "cache exceeds inclusive input".
  - The result: auto-compaction fails mid-turn, web search returns `FunctionCallError::Fatal`, and a paid image is discarded. This contradicts the documented best-effort behaviour in `accounting_extensions.rs:53-55` and the commit title of `4e1781fec2`.
- **Fix:** on these paths, log the failure and record the attempt as unrecorded, then return the provider response. In `resolve_path`, return `Ok(None)` instead of an error.

**5. Major: ephemeral sessions are silently uncollected, including guardian forked reviews**
- **Where:** `core/src/accounting.rs:256-257`. Commit `1aca553905`.
- **What happens:**
  - Any session with no live thread is disabled with no log line.
  - That covers `exec --ephemeral` and also `guardian/review_session.rs:610` (`fork_config.ephemeral = true`), so every parallel approval review is paid inference that is never recorded.
  - Neither appears in `acct-coverage-audit-20260921.md`.
- **Fix:** attribute guardian forks to the parent session's thread. At minimum, emit a named `accounting.excluded` warning once per session and list it in the coverage audit.

**6. Minor: a missing OpenRouter cache-write count is stored as an observed zero**
- **Where:** `core/src/accounting_chat.rs:296-303`. Commit `94c01d2a9f`.
- **What happens:** when the cache-write field is absent, a `Presence::Number(0)` is written into the stored evidence. That is an inference written as if the provider reported it, and no later rules version can correct it. It goes against the principle stated in `861b933ec1` ("keep the observation as reported"). It can also overwrite an earlier non-zero count during replay.
- **Fix:** store `Missing`, and apply the "absent means zero" rule when quoting, keyed on the price record. Bump the pricing rules version when doing so.

**7. Minor: the Responses path reads cache-write counts on every route**
- **Where:** `core/src/accounting_responses.rs:304`.
- **What happens:** this includes Vercel, which is the replay-failure hazard `7136d59eb0` removed for Chat. If a gateway reports cache writes outside the input count, the observation fails after the request was sent, and the turn ends.
- **Fix:** read the count only on routes where its meaning is established.

**8. Minor: misleading `/cost` wording**
- **Where:** `tui/src/chatwidget/tokens.rs:1180-1192` (`8e952e2fe3`, `bc01aef837`) and `:1502` (`0e0d692166`).
- **What's wrong:**
  - "N attempts had no price" counts any incomplete estimate. On every Chat route except OpenRouter and the free-cache-write providers, the uncached input is unknown because the cache-write count is missing. Failed or 429 attempts with no usage are also counted. The price exists in those cases; a token count is what's missing.
  - "No price available" is shown whenever the known total is exactly 0, even when priced-zero attempts exist alongside unknown ones.
  - "Your provider's bill is the final amount" appears on days that were entirely subscription work.
- **Fix:** separate "missing token counts" from "missing rate", and condition the footer on whether any pay-per-use attempts exist.

**9. Minor (privacy): a caller-supplied base URL is written to logs**
- **Where:** `app-server/src/request_processors/turn_processor.rs:1077-1082`. Commit `8add43ea17`.
- **What happens:** it logs `base_url` and `path` from the pane bridge at warn level. That contradicts the in-code rule at `core/src/accounting.rs:838-839` ("Never log the endpoint itself").
- **Fix:** log only `provider_id`, or just the host.

**10. Minor: any provider with a command-based login is treated as a subscription**
- **Where:** `core/src/accounting.rs:420` (`provider_plan_login = provider.auth.is_some()`). Commits `e474fc2244`, `7be22e72be`.
- **What happens:** the built-in Amazon Bedrock provider lets users set `auth`, so a Bedrock bearer-token command is classified as subscription capacity.
- **Fix:** name `CLAUDE_PLAN_PROVIDER_ID` explicitly, or add a declared provider flag.

**11. Nit: inconsistent auth source**
- **Where:** `core/src/accounting_extensions.rs:100`, `core/src/realtime_conversation.rs:2502`.
- **What happens:** these use the global `auth_manager.auth()` rather than the provider's own `provider.auth()`, unlike `attach_turn`. It's harmless only while these features are OpenAI-only.

**12. Nit: two sites still price by the name sent on the wire**
- **Where:** `core/src/client.rs:1544` (legacy compaction uses the wire model) and the realtime path (`session_config.model`).
- **What happens:** `db1221bce6` missed these. They should use `accounting_model_identity(&model_info.slug)`.

**13. Nit: a failed pane-bridge report is only visible at debug log level**
- **Where:** `tui/src/app/background_requests.rs:1350`.
- **What happens:** at the default log level the record is silently dropped.

**14. Nit: holiday dates and weekdays use different days for overnight windows**
- **Where:** `protocol/src/openai_models.rs:538`.
- **What happens:** the off-peak date check uses the instant's own calendar date, while the weekday check uses the day a window opened. For peak windows that cross midnight, the two can disagree.

### Checked and sound
- **History is not re-priced:** each attempt is bound to an immutable price record at admission. Recorded estimates are verified under their own rules version, and the free-cache-write rule only applies to records that state a zero write rate.
- **DeepSeek:**
  - Peak/off-peak is resolved at the dispatch instant, including the weekday check for overnight windows and holiday dates.
  - The cache-write rule extends the record's identity rather than changing earlier ones.
  - Cache-miss input is derived only on inclusive routes.
- **Anthropic cache writes:** 1.25x (5-minute) on API keys and 2x (1-hour) on the Claude subscription. This matches the client's `anthropic_cache_control(is_claude_plan)`. Writes are left unpriced when the 1.25x multiple isn't exact in the catalogue's unit.
- **Plan basis on the OpenAI and Claude-plan paths:** chosen by authentication. The API-equivalent figure is limited to OpenAI rows, Claude-plan uses its Anthropic API twin, and the plan rate can never sit on a billed record.
- **Stated charges:**
  - Accepted only from OpenRouter, Vercel and the Corbanu API.
  - OpenRouter BYOK and Vercel own-key responses are excluded, and malformed amounts are dropped rather than rounded.
  - They are never added to `known_usd`, and are shown as "stated by the provider".
- **Unknown stays unknown:** parsers keep absent fields as `Missing`, `Null` never overwrites a recorded count, and estimates are shown as "at least" when incomplete.
- **Routing checks:**
  - Compressed and multipart request bodies can be inspected.
  - Unattributable requests are served and logged as excluded.
  - Redirects are refused on recorded requests.
  - Configured query parameters are compared without regard to order.
- **No sensitive data in the ledger:** only numeric usage, provider, model, a random scope UUID and turn labels, with no prompts or credentials. Turn labels are bounded with a digest so they can't collide.
exit=0
