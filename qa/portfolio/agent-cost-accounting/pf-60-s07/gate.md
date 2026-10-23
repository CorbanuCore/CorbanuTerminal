# PF-60-S07 gate — Moonshot route pricing, charge pricing for Vercel and the Corbanu API, #786

Travis's 2026-10-14 decisions (comments on #368 and #786): price Moonshot's Kimi Open Platform by route first and defer
BigModel; price Vercel and Corbanu API requests from the charge they report, by OpenRouter's rule (#784); for #786,
record a reasoning count above the output count as not reported and keep the rest of the record. Code-blind execution
is PF-60-S10's job, under the enforced isolation gate.

## What changed

- **Moonshot by route (AC2).** A provider the user configures at `https://api.moonshot.ai/v1` (Chat Completions, API
  key it can attribute) gets `PriceAuthority::RouteRates`. Admission binds Moonshot's published rates for `kimi-k3`,
  `kimi-k2.7-code`, `kimi-k2.7-code-highspeed` and `kimi-k2.6` (read 2026-10-14,
  [issue-368 data](../issue-368/data/moonshot-pricing-20261014.txt)). The provider id is not part of the price
  identity. A model the table doesn't list has no price.
- **K3 cache writes by TTL.** Moonshot bills writes $3.00/M for five minutes and $6.00/M for one hour, and five minutes
  when a request names no lifetime. This client's Chat Completions body has no `prompt_cache_options`
  ([usage reading](../issue-368/data/moonshot-chat-usage-20261014.txt)), so every request is priced at the five-minute
  rate. A test checks the request body names no lifetime. K2 rows state only hit and miss prices, so a written token is
  priced as a miss.
- **Moonshot usage.** On that route `prompt_tokens_details.cache_write_tokens` is recorded. Moonshot documents it as the
  written part of `prompt_tokens`, separate from the cached part.
- **Vercel and the Corbanu API priced from their charge.** `/cost` treats the Vercel AI Gateway (`gatewayCost`) and the
  Corbanu API (`usage.corbanu_charge_usd`, both dialects) like OpenRouter. The charge is the cost, labelled
  `charged by <provider>`. Each page's `Price basis` names the field, and the catalogue figure stays on the attempt
  page as an unused estimate. Several charging providers on one screen are each named. The ledger is unchanged.
- **#786.** A Chat usage report whose reasoning count is above its output count records reasoning as not reported. The
  rest of the report is kept, the charge included. Before, replay refused the whole record. The recovery test that
  listed that report as invalid evidence was updated. Responses routes and split or cumulative reports are #814.

## Evidence

| Check | Result |
| --- | --- |
| Fail-before | Sources reverted to `origin/main` with the new tests kept: `accounting_chat_reasoning_above_output_keeps_the_record` fails (0 observations recorded); the 2 new TUI charge tests and the updated `accounting_inspect_reads_plainly_for_every_provider` fail. The Moonshot tests use the new route type, so they can't build on the old tree; there the route was `Unavailable` (no rate). |
| Pass-after, new tests | core: `moonshot_route_is_priced_by_the_route`, `moonshot_route_admission_binds_published_rates` (K3 $0.0321 and K2.6 $0.00992 exactly from a Moonshot-shaped report), `accounting_chat_cache_writes_are_read_on_the_moonshot_route`, `accounting_chat_reasoning_above_output_is_not_reported`, suite `accounting_chat_reasoning_above_output_keeps_the_record`, `accounting_chat_requests_name_no_cache_lifetime`; tui: `vercel_and_corbanu_api_requests_are_priced_from_their_charge`, `charges_from_several_routes_name_each_provider`. |
| `just test -p codex-core -E 'test(accounting) \| test(spawn)'` | 357/357 with `developer-accounting`; 352/352 without (macOS, head `b568924af3`) |
| `just test -p codex-tui --features developer-accounting -E 'test(charge) \| test(tokens) \| test(cost)'` | 131/132. The one failure is the `cost_scope` tmux test, which times out waiting for a turn on this Mac; earlier workers report it failing locally with their builds too (acct-or result), and it isn't in PR CI. |
| Clippy `-D warnings --tests` | Linux (glitch): `codex-core` and `codex-tui`, with and without `developer-accounting`, clean on `d840550b1f`; re-run on the final head in the PR. macOS: in the PR. |
| tmux, Moonshot route | Real candidate binary, disposable home, `CORBANU_TEST_NO_NATIVE_KEYRING=1`. No Moonshot key exists in the vault, so `https://api.moonshot.ai/v1` was reached through a local stand-in: a CONNECT proxy (`HTTPS_PROXY`) presenting a throwaway-CA certificate for `api.moonshot.ai` (`CODEX_CA_CERTIFICATE`) and answering in Moonshot's documented usage shape. `/cost`: `Estimated cost: $0.032100`; components 832 × $3 + 2,000 × $0.30 + 7,168 × $3 + 500 × $15 per million ([first screen](tmux-20261014/moonshot-first-screen.txt), [request](tmux-20261014/moonshot-request.txt), [technical](tmux-20261014/moonshot-technical.txt)). The stand-in logged no `prompt_cache_options` in the request. |
| tmux, Corbanu API (real key `provider/pfterminal_plan_api_key`) | `corbanu/deepseek-v4.1-flash`: `Cost: $0.002145, charged by Corbanu API`; `Price basis: the charge Corbanu API reported with the response (usage.corbanu_charge_usd)`; the other-conversations view sums it the same way ([first screen](tmux-20261014/corbanu-api-first-screen.txt), [request](tmux-20261014/corbanu-api-request.txt), [technical](tmux-20261014/corbanu-api-technical.txt), [other conversations](tmux-20261014/corbanu-api-other-conversations.txt)). |
| tmux, Vercel (real key `provider/ai_gateway_api_key`) | `zai/glm-5.3-flash`: the priced attempt reads `Cost: $0.001397 (rounded), charged by Vercel`, `Price basis: … (gatewayCost)`, exact 0.0013967 ([technical](tmux-20261014/vercel-technical-attempt2.txt)). The request also recorded a first attempt with no usage at all, so the request reads `at least $0.001397` ([first screen](tmux-20261014/vercel-first-screen.txt), [request](tmux-20261014/vercel-request.txt)); see observations. |
| Key scan | Both vault keys: 0 hits in 22,459 files (run directories and this folder). |
| Videos | [Moonshot route](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s07-pf60s07-moonshot-route-cost-b568924af3b8-2026-10-14.mp4) (33 s, stand-in), [Corbanu API charge](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s07-pf60s07-corbanu-api-charge-cost-b568924af3b8-2026-10-14.mp4) (40 s, real key) |

Spend: two real one-word turns, about $0.0035 together.

## Not verified live, and why

- **A real Moonshot request.** The vault has no Kimi Open Platform key (`provider/kimi_api_key` is Kimi Code's and
  returns 401 on `api.moonshot.ai`). The route, the rates and the usage reading are shown against the stand-in and in
  tests; a real request still needs a key.

## Observations

- **Vercel: an extra attempt with no usage.** The Vercel turn recorded two attempts for one request: the first has no
  usage, the second carries the charge. `/cost` therefore reads `at least` for the request. This is how the ledger
  records that request today, not something this change introduced.
- **The provider's name on a user-configured route** is the user's id (`kimi`), as for any custom provider.
