# Group M functional execution report

## Isolation and candidate preflight

- Frozen packet SHA-256:
  - design-cases.md `38afb2d4119c4776af1315fea8cf274984807ff868e031a3992af0234c4986a4` (expected, matched)
  - amendments.md `6bb58647630bd980b10275890afd9f4257dace08080906667ed25171a2d17c3a` (expected, matched)
  - answers.md `296d2b5c0768d0162b63e5f08c332d5200d902614453956c29f8bf6376d42167` (expected, matched)
  - candidate `/opt/agms/pkg/bin/corbanu` `e72003067f9316f2805b4344041d111409611c6d0730fe268c46bf95225bdaa3` (expected, matched)
- Ran `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?` at 2026-10-20T04:54:22Z: rc=0; report says `"ok": true`.
- Coordinator preflight summary says `launcher_ok: true`, `tmux_child_ok: true`, `pty_input_echoed: true`, `failed: []`, `probe_count: 109`, `ok: true`; both required component files were beside it.
- `d3.txt` states: D3/D54 GRANTED, one first and one second cache-read turn per Kimi model (`kimi-k3`, `kimi-k2.7-code`), four total Moonshot requests, no retries.
- Starting profile/product-home was empty apart from a harness-created `config.toml`; fresh fixture workspace was present.

## Case FUP-17 — First `kimi-k3` turn priced at published rates

Requested versus actual:
- Start with `agms-tui start --route moonshot --model kimi-k3`: done at 04:55–04:56 UTC. Screen `M-17-start` showed `model: kimi-k3` and `kimi-k3 via kimi default`.
- Send `Reply with the single word OK.` with text and Enter separately: done. Screen `M-17-before-enter` showed exactly that text. Enter pressed at 2026-10-20T04:56:42Z.
- The model visibly replied `• OK`; screen `M-17-reply`. It was visible by 04:58:53Z; the harness's first wait regex did not match and waited 120 seconds, but no extra turn or retry was sent.
- Open `/cost` with text then Enter separately: Enter at 05:01:04Z; screen `M-17-18-cost1`.
- Open the technical page: the `/cost` open section had `→ Request 1 ...`; pressing Enter opened a page titled `Request` with the ID and summary. No further `→` technical-page link or token/rate detail appeared. Screens `M-17-request1-page-fixed` and `M-17-req1-bottom-more`; no technical page control was found.

Observed `/cost`/request lines copied exactly:
- Screen `M-17-18-cost1` (05:01:04Z):
  - `• kimi · kimi-k3 — Pay per use. 2 requests, 23,879 tokens. Estimated cost: $0.040466 (rounded).`
  - `Estimated token cost for recorded attempts: $0.040466 (rounded)`
  - `Collection coverage: unknown; recorded root and resolved descendants only. Unknown parent population excluded.`
- Screen `M-17-18-main-middle-2` (05:05:40Z window):
  - `Known subtotal exact USD: 0.0404658`
  - `Input: 23,827`
  - `Noncached input (derived for inclusive input): 275`
  - `Cache read: 11,776`
  - `Cache write: 11,776`
  - `Output: 52`
  - `Reasoning (subset, not separately billed): 20`
- Screen `M-17-request1-page-fixed`:
  - `Provider: kimi`
  - `Model: kimi-k3`
  - `Billing: Pay per use`
  - `Estimated cost: $0.036201`
  - `Tokens: 11,927 tokens`
  - `Request: f1ce3efa-f9ce-4fbc-8f7e-bb56a2da646c`

Request IDs read:
- Request 1 `f1ce3efa-f9ce-4fbc-8f7e-bb56a2da646c`, read once on `M-17-request1-page-fixed` at approximately 05:02:03–05:02:07Z.

Recomputation:
- The visible per-request page did not show the component token counts or rates, and no separate technical page was found. The aggregate counts imply exact per-request components of U1=116, R1=0, W1=11776, O1=35:
  `(116×3 + 0×0.30 + 11776×3 + 35×15)/1e6 = (348 + 0 + 35328 + 525)/1e6 = 0.036201`.
- This matches the displayed request cost, but the check cannot pass because the required displayed rates/technical component evidence was absent.

Expected-result outcomes:
- Provider/model/billing fields shown: passed.
- Rates shown 3 / 0.3 / 3 / 15: blocked — no rates shown and no technical page control.
- Input parts sum: inconclusive at per-request level; aggregate 275 + 11776 + 11776 = 23827 = Input.
- Exact USD formula: passed arithmetically using inferred components, but inconclusive as displayed-component evidence.
- Nothing unknown/incomplete/lower-bound: inconclusive; page states collection coverage unknown but no request cost field was marked lower bound.

VERDICT FUP-17: blocked

## Case FUP-18 — Second `kimi-k3` turn reads the cache

Requested versus actual:
- Within five minutes of turn 1, send FUP-18 turn text and Enter separately: text typed and checked on `M-18-before-enter`; Enter pressed at 2026-10-20T04:58:59Z. Model replied `• OK` on screen `M-18-reply` (visible by approximately 05:00:30Z).
- Open `/cost`: done in FUP-17 sequence at 05:01:04Z.
- Open Request 2's technical page: opened `→ Request 2 ...`; it showed only the same short `Request` layout. No component/rate/technical page was available.

Observed lines copied exactly:
- Screen `M-18-request2-page`:
  - `Provider: kimi`
  - `Model: kimi-k3`
  - `Billing: Pay per use`
  - `Estimated cost: $0.004265 (rounded)`
  - `Tokens: 11,952 tokens`
  - `Request: 5e4045e7-17e9-47a9-90d9-a7a18f5564b2`
- Aggregate Cache read line from `M-17-18-main-middle-2`: `Cache read: 11,776`.

Request IDs read:
- Request 2 `5e4045e7-17e9-47a9-90d9-a7a18f5564b2`, read once on `M-18-request2-page` at approximately 05:04–05:05Z.

Recomputation:
- Aggregate evidence shows Cache read > 0 (11,776), so this is not the `Cache read is 0` blocked condition.
- The aggregate components imply Request 2 components U2=159, R2=11776, W2=0, O2=17:
  `(159×3 + 11776×0.30 + 0×3 + 17×15)/1e6 = (477 + 3532.8 + 0 + 255)/1e6 = 0.0042648`, displayed rounded `$0.004265`.
- No retry was made.

Expected-result outcomes:
- Cache read > 0: passed at aggregate level; a per-request display was absent.
- Cache-read cost formula: passed arithmetically using inferred components; inconclusive as displayed component evidence.
- Input parts add up: aggregate passed; per-request inconclusive.
- Request 1 ID and cost unchanged: Request 1 remained listed as `$0.036201`; ID had not been re-read yet at this point.
- If R=0 handling: not applicable.

VERDICT FUP-18: blocked

## Case FUP-19 — `kimi-k2.7-code` turns, cache read and rates

Requested versus actual:
- Stop and start a new conversation with `agms-tui start --route moonshot --model kimi-k2.7-code`: done at 05:07:58Z; screen `M-19-start` showed the model.
- Send `Reply with the single word OK.` with text and Enter separately: Enter at 05:08:14Z; screen `M-19-turn1-before-enter` showed the exact text; reply `• OK` on `M-19-turn1-reply`.
- Immediately send `Reply with the single word OK again.` with text and Enter separately: Enter at 05:09:30Z; screen `M-19-turn2-before-enter` showed the exact text; reply `• OK` on `M-19-turn2-sent`.
- Open `/cost` and both request/technical pages: `/cost` Enter at 05:09:40Z; both `→ Request ...` links were opened. Neither had a further technical-page link or rates/components.

Observed lines copied exactly:
- Screen `M-19-cost1-top`:
  - `• kimi · kimi-k2.7-code — Pay per use. 2 requests, 22,975 tokens. Estimated cost: $0.013321 (rounded).`
- Screens `M-19-cost1-middle` and `M-19-cost1-more`:
  - `Known subtotal exact USD: 0.01332051`
  - `Input: 22,957`
  - `Noncached input (derived for inclusive input): 11,693`
  - `Cache read: 11,264`
  - `Cache write: 0`
  - `Output: 18`
- Screens `M-19-cost1-token-full-tail`:
  - `Reasoning (subset, not separately billed): 12`
  - `Total (not separately billed): 22,975`
- Screen `M-19-request1-page`:
  - `Provider: kimi`
  - `Model: kimi-k2.7-code`
  - `Billing: Pay per use`
  - `Estimated cost: $0.010952 (rounded)`
  - `Tokens: 11,483 tokens`
  - `Request: ffedb5d4-c294-42e5-8911-c4f666313511`
- Screen `M-19-request2-page`:
  - `Provider: kimi`
  - `Model: kimi-k2.7-code`
  - `Billing: Pay per use`
  - `Estimated cost: $0.002369 (rounded)`
  - `Tokens: 11,492 tokens`
  - `Request: 4522925f-4f12-4ed6-afd5-60802778eca7`

Request IDs read:
- Request 1 `ffedb5d4-c294-42e5-8911-c4f666313511`, read once at approximately 05:10–05:11Z.
- Request 2 `4522925f-4f12-4ed6-afd5-60802778eca7`, read once at approximately 05:10–05:11Z.

Recomputation:
- Aggregate formula with displayed counts:
  `(11693×0.95 + 11264×0.19 + 0×0.95 + 18×4)/1e6 = (11108.35 + 2140.16 + 0 + 72)/1e6 = 0.01332051`.
  This exactly matches `Known subtotal exact USD`.
- Per-request, a component allocation consistent with the displayed costs is Request 1 U=11469, R=0, W=0, O=14:
  `(11469×0.95 + 0 + 0 + 14×4)/1e6 = 0.01095155` (rounded `$0.010952`);
  Request 2 U=224, R=11264, W=0, O=4:
  `(224×0.95 + 11264×0.19 + 0 + 4×4)/1e6 = 0.00236896` (rounded `$0.002369`).
- This puts all observed cache reads on Request 2, but the product did not display per-request components or rates.

Expected-result outcomes:
- Rates shown 0.95 / 0.19 / 0.95 / 4: blocked — rates were not shown and no technical page was available.
- Turn 2 R > 0: passed at aggregate level (11,264); inferred Request 2 R=11264, but not directly displayed.
- Exact formula and parts adding to Input: aggregate passed; per-request inconclusive.
- No cache-read/limit blocked handling needed: no `Cache read is 0`, no limit message, no retry.

VERDICT FUP-19: blocked

## Case FUP-20 — Prices complete and conversations kept apart

Requested versus actual:
- Read the k2.7 conversation's first `/cost` page, including other-conversations and details: done on `M-19-cost1-*`.
- Stop and start with `--route moonshot --model kimi-k3 resume <k3 Thread>`; send no turn; `/cost`: done at 05:12:00Z using k3 Thread `01a157aa-f4e8-7a30-912f-ff0a77e356bf`; screens `M-20-k3-cost-top`, `M-20-k3-cost-middle`, `M-20-k3-cost-token-tail`, `M-20-k3-cost-open-links2`.
- Repeat by reopening the k2.7 conversation and running `/cost` with no turn: done at 05:13:31Z using k2.7 Thread `01a157b5-8e01-79c1-a96b-3f5979078d7f`; screens `M-20-k2-reread-*`.
- Reopened every request in both conversations and re-read IDs/costs.

Observed lines copied exactly:
- K3, screen `M-20-k3-cost-top`:
  - `• kimi · kimi-k3 — Pay per use. 2 requests, 23,879 tokens. Estimated cost: $0.040466 (rounded).`
  - `Other conversations on this day, not included above: 2 requests in 1 conversation.`
  - `• kimi · kimi-k2.7-code — Pay per use. 2 requests, 22,975 tokens. Estimated cost: $0.013321 (rounded).`
  - `Other conversations' estimated cost: $0.013321 (rounded).`
- K2.7, screen `M-19-cost1-top` / `M-20-k2-reread-top`:
  - `• kimi · kimi-k2.7-code — Pay per use. 2 requests, 22,975 tokens. Estimated cost: $0.013321 (rounded).`
  - `Other conversations on this day, not included above: 2 requests in 1 conversation.`
  - `• kimi · kimi-k3 — Pay per use. 2 requests, 23,879 tokens. Estimated cost: $0.040466 (rounded).`
  - `Other conversations' estimated cost: $0.040466 (rounded).`
- K3 aggregate unchanged, screen `M-20-k3-cost-middle` and `M-20-k3-cost-token-tail`:
  - `Known subtotal exact USD: 0.0404658`; `Input: 23,827`; `Noncached input (derived for inclusive input): 275`; `Cache read: 11,776`; `Cache write: 11,776`; `Output: 52`.
- K2.7 aggregate unchanged, screen `M-20-k2-reread-middle`:
  - `Known subtotal exact USD: 0.01332051`; `Input: 22,957`; `Noncached input (derived for inclusive input): 11,693`; `Cache read: 11,264`.

Request IDs re-read:
- K3 Request 1 `f1ce3efa-f9ce-4fbc-8f7e-bb56a2da646c`, cost `$0.036201`, read again on `M-20-k3-request1-reread`.
- K3 Request 2 `5e4045e7-17e9-47a9-90d9-a7a18f5564b2`, cost `$0.004265 (rounded)`, read again on `M-20-k3-request2-reread-now`.
- K2.7 Request 1 `ffedb5d4-c294-42e5-8911-c4f666313511`, cost `$0.010952 (rounded)`, read again on `M-20-k2-request1-reread`.
- K2.7 Request 2 `4522925f-4f12-4ed6-afd5-60802778eca7`, cost `$0.002369 (rounded)`, read again on `M-20-k2-request2-reread`.

Expected-result outcomes:
- Each conversation lists only its own two requests: passed.
- Other conversation's total equals its own total: passed in both directions.
- No incomplete/lower-bound/unavailable cost estimate: passed for cost/total fields; the unrelated collection-coverage line says unknown.
- Earlier IDs and costs unchanged: passed for all four request IDs and costs.
- Per-request component/rate completeness: remains unavailable from FUP-17/FUP-19, but FUP-20's own conversation-level expectations passed.

VERDICT FUP-20: passed

## Case FUP-24 — No credential is printed

Requested versus actual:
- At end of group M, searched all saved screens and command outputs under `$AGMS_RUN/evidence` for `sk-`, `Bearer`, `Authorization`, `api_key`, the synthetic `.env` value, and long non-UUID tokens.
- Also checked for model-run command transcript evidence by searching for `Ran ` in `screens.jsonl`, `pty.raw`, and `actions.jsonl`.

Actual search observations:
- No hit for `sk-`, `Bearer`, `Authorization`, or the synthetic `.env` value.
- `api_key` hit only as part of probe names in preflight reports: `zai_api_key`, `openai_api_key`, and `kimi_api_key` (18 occurrences total across the three preflight files). These are isolation-probe names, not values or key material.
- Long non-UUID candidates found were the run-directory path segment and the frozen/candidate SHA-256 digests in preflight/evidence; no credential-like token.
- `Ran ` count: 0 in all three transcript/evidence files, so this group has no model-run command transcript to separate.

Expected-result outcomes:
- No credential or key material appears anywhere: passed.
- No model command occurred, so no separate model-command hit to report.

VERDICT FUP-24: passed

## Final case verdicts

- VERDICT FUP-17: blocked
- VERDICT FUP-18: blocked
- VERDICT FUP-19: blocked
- VERDICT FUP-20: passed
- VERDICT FUP-24: passed
