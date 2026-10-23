# Group D functional execution report

Scope: assigned cases ACC-19, ACC-20, ACC-21, ACC-22, ACC-34 only. This is an independent executor report, not human acceptance.

## Environment / preflight

- Packet design SHA-256: `f750b006046cb5036f92fd6f4f8d910ccd8aa33f88b33c72ffefd0f6cc01bfd6` (matches).
- Amendments SHA-256: `2a73b6503429dc5a6d989b313e996a9041f5dc60b2c130e329dd39e1d71f105e` (matches).
- Environment answers SHA-256: `dc4044d36ac3868205ff2dc71cee36805969c6896c683852a6f9d705dc2d7c0c` (matches).
- Candidate binary SHA-256: `5b805b9c244e55c12c6f61fc3d31bddd760a415554cbabff8cbcb4c3ba0531a1` (matches); packaged version reported by probe: `corbanu 0.1.48`.
- Coordinator preflight summary: `preflight-summary.json` reports `ok: true`, launcher and tmux child `true`, pty input echo `true`, failed list empty, 105 probes.
- Both required `preflight-*.json` files beside the summary were present (`preflight-launcher.json` and `preflight-tmux-child.json`); each reported `ok: true`.
- Executor preflight at 2026-10-16T04:59:51Z: `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json`; rc `0`; top-level `"ok": true`. Evidence retained.
- Group start UTC: 2026-10-16T05:01:07Z.
- Starting conditions checked: fixture project exists; product home contains only neutral `config.toml` (no conversation files visible); route `openai` real and relay-only network is confirmed by preflight.

## ACC-19 — GPT-6.1 Sol rates, including cache writes

**Requested actions / actual**
1. Requested: start a fresh profile and new conversation on `gpt-6.1-sol`; actual: `agms-tui start --route openai --model gpt-6.1-sol` at 2026-10-16T05:01:27Z. The initial screen showed `model: gpt-6.1-sol`, `GPT-6.1 Sol default`, and the fixture workspace. The product home had only the neutral `config.toml` before the run.
2. Requested: type the literal prompt; actual: typed `Reply with the single word OK.` at 2026-10-16T05:01:36Z and verified the composer text before Enter; pressed Enter separately at 2026-10-16T05:01:40Z. Visible reply at 2026-10-16T05:02:03Z was `• OK`. Two requests were recorded: a prewarm and the user turn; the case did not prohibit this count.
3. Requested: run `/cost` and open the technical pages; actual: `/cost` at 2026-10-16T05:02:11Z, opened Request 2 technical details at 2026-10-16T05:05:25Z, Request 1 technical details at 2026-10-16T05:08:42Z, and scrolled to all required cost/rate lines.

**Visible `/cost` lines (copied exactly)**
- Screen `ACC19-cost-first`, read 2026-10-16T05:02:11Z: `• OpenAI · GPT-6.1 Sol — Pay per use. 2 requests, 32,485 tokens. Estimated cost: $0.073924.`
- Screen `ACC19-cost-first`: `Estimated token cost for recorded attempts: $0.073924`
- Screen `ACC19-open-list2`, read 2026-10-16T05:04:22Z: `→ Request 1 · OpenAI · GPT-6.1 Sol · estimated $0.029298`; `→ Request 2 · OpenAI · GPT-6.1 Sol · estimated $0.044626`
- Screen `ACC19-cost-open-list`, read 2026-10-16T05:04:11Z: `Known subtotal exact USD: 0.073924`; `Cache read: 0`; `Cache write: 17,828`; `Output: 5`; `Total (not separately billed): 32,485`.

**Request 2 technical page (screen `ACC19-request2-technical-up3`, 2026-10-16T05:07:20Z; costs `ACC19-request2-technical-midcost3`, 2026-10-16T05:08:08Z; rates `ACC19-request2-technical-up1`, 2026-10-16T05:07:01Z)**
- Tokens: `Input: 17831`; `Noncached input (derived for inclusive input): 3 (derived)`; `Cache read: 0 (reported)`; `Cache write: 17828`; `Output: 5`; `Total (not separately billed): 17836`.
- Costs: `Noncached input cost: $0.000006; exact USD 0.000006`; `Cache read cost: $0.000000; exact USD 0`; `Cache write cost: $0.044570; exact USD 0.04457`; `Output cost: $0.000050; exact USD 0.00005`; `Known subtotal exact USD: 0.044626`.
- Rates: `Noncached input rate: 2 USD per million tokens`; `Cache read rate: 0.1 USD per million tokens`; `Cache write rate: 2.5 USD per million tokens`; `Output rate: 10 USD per million tokens`; no `Rate unavailable for Cache write`.
- Recomputation: 3×2/1,000,000 = $0.000006; 0×0.1/M = $0; 17,828×2.5/M = $0.04457; 5×10/M = $0.00005; sum = $0.044626. This matches the exact subtotal.
- The page also printed an inactive long-context tier (`4`, `0.2`, `5`, `15`) and explicitly said `Long-context price not applied: input 17831 ≤ 272000`; the applied base rates above are the judged rates.

**Request 1 technical page (screens `ACC19-request1-technical-up3`, `ACC19-request1-technical-up2`, `ACC19-request1-technical-up1`)**
- Tokens: `Input: 14649`; `Noncached input (derived for inclusive input): 14649 (derived)`; cache read/write/output all `0 (reported)`.
- Costs: `Noncached input cost: $0.029298; exact USD 0.029298`; cache read/write/output each `$0.000000`; `Known subtotal exact USD: 0.029298`.
- Rates: `Noncached input rate: 2`, `Cache read rate: 0.1`, `Cache write rate: 2.5`, `Output rate: 10 USD per million tokens`; no `Rate unavailable for Cache write`.
- Recomputation: 14,649×2/M = $0.029298; all other components = $0; sum = $0.029298. The long-context tier was also present but marked not applied (`input 14649 ≤ 272000`).

**Outcome**
- Expected result 1 (rates $2 input, $0.10 cached, $2.50 cache write, $10 output): **passed**.
- Expected result 2 (`Rate unavailable for Cache write` absent): **passed**.
- Expected result 3 (component costs recompute exactly): **passed**.

## ACC-20 — GPT-5.6 Sol promotional rates

**Requested actions / actual**
1. Requested: same profile and a new conversation on `gpt-5.6-sol`; actual: stopped ACC-19 at 2026-10-16T05:10:34Z and started `--route openai --model gpt-5.6-sol` at 2026-10-16T05:10:37Z. Screen showed `model: gpt-5.6-sol` and `GPT-5.6-Sol default` in the same fixture workspace.
2. Requested: type the same literal prompt; actual: typed and verified `Reply with the single word OK.` at 2026-10-16T05:10:48Z, pressed Enter at 2026-10-16T05:10:52Z. Reply at 2026-10-16T05:11:12Z was `• OK`. Two requests (prewarm and user turn) appeared.
3. Requested: run `/cost` and open technical pages; actual: `/cost` at 2026-10-16T05:11:18Z; opened Request 2 and Request 1 technical pages and scrolled to all costs/rates.

**Visible `/cost` lines (copied exactly)**
- Screen `ACC20-cost-first`, read 2026-10-16T05:11:18Z: `• OpenAI · GPT-5.6-Sol — Pay per use. 2 requests, 24,283 tokens. Estimated cost: $0.110939.`; `Other conversations' estimated cost: $0.073924.`
- Screen `ACC20-cost-details`, read 2026-10-16T05:11:43Z: `Estimated token cost for recorded attempts: $0.110939`.
- Screen `ACC20-cost-details2`, read 2026-10-16T05:11:43Z: `Known subtotal exact USD: 0.110939`; `Input: 24,278`; `Noncached input (derived for inclusive input): 10,551`; `Cache read: 0`; `Cache write: 13,727`.
- Screen `ACC20-cost-details3`, read 2026-10-16T05:11:43Z: `Total (not separately billed): 24,283`; `→ OpenAI · GPT-5.6-Sol — estimated $0.110939 (2 requests)`; `→ Request 1 · OpenAI · GPT-5.6-Sol · estimated $0.042192`; `→ Request 2 · OpenAI · GPT-5.6-Sol · estimated $0.068747`.

**Request 2 technical page (screens `ACC20-request2-technical-up-tokens2`, `ACC20-request2-technical-up-cost`, `ACC20-request2-technical-costs3`, `ACC20-request2-technical-rates-complete`)**
- Tokens: `Input: 13730`; `Noncached input (derived for inclusive input): 3 (derived)`; `Cache read: 0 (reported)`; `Cache write: 13727`; `Output: 5`; `Total (not separately billed): 13735`.
- Costs: `Noncached input cost: $0.000012; exact USD 0.000012`; `Cache read cost: $0.000000; exact USD 0`; `Cache write cost: $0.068635; exact USD 0.068635`; `Output cost: $0.000100; exact USD 0.0001`; `Known subtotal exact USD: 0.068747`.
- Rates: `Noncached input rate: 4 USD per million tokens`; `Cache read rate: 0.4 USD per million tokens`; `Cache write rate: 5 USD per million tokens`; `Output rate: 20 USD per million tokens`.
- Recomputation: 3×4/M = $0.000012; 0×0.4/M = $0; 13,727×5/M = $0.068635; 5×20/M = $0.0001; sum = $0.068747. Matches.
- Inactive long-context tier was `8`, `0.8`, `10`, `30`, explicitly `Long-context price not applied: input 13730 ≤ 272000`.

**Request 1 technical page (screens `ACC20-request1-technical-up-tokens2`, `ACC20-request1-technical-up-cost`, `ACC20-request1-technical-costs3`, `ACC20-request1-technical-rates-complete`)**
- Tokens: `Input: 10548`; `Noncached input (derived for inclusive input): 10548 (derived)`; cache read/write/output all `0 (reported)`.
- Costs: `Noncached input cost: $0.042192; exact USD 0.042192`; cache read/write/output each `$0.000000`; `Known subtotal exact USD: 0.042192`.
- Rates: `Noncached input rate: 4`, `Cache read rate: 0.4`, `Cache write rate: 5`, `Output rate: 20 USD per million tokens`.
- Recomputation: 10,548×4/M = $0.042192; all other components = $0; sum = $0.042192. Long-context tier `8`, `0.8`, `10`, `30` was marked not applied.

**Outcome**
- Expected result 1 (rates $4 input, $0.40 cached, $5 cache write, $20 output): **passed**.
- Expected result 2 (costs recompute exactly): **passed**.

## ACC-21 — image request priced at `gpt-image-2` rates

**Requested actions / actual**
1. Requested: same profile and a new conversation on `gpt-6-luna`; actual: stopped ACC-20 at 2026-10-16T05:15:03Z and started `--route openai --model gpt-6-luna` at 2026-10-16T05:15:06Z. Screen showed `model: gpt-6-luna` and `GPT-6 Luna default`.
2. Requested: type the image prompt; actual: typed and verified `Use the image generation tool to create a small image of a plain red square, then reply DONE.` at 2026-10-16T05:15:14Z; pressed Enter at 2026-10-16T05:15:19Z. Reply at 2026-10-16T05:15:49Z ended `• DONE`, with a generated image saved under `$AGMS_RUN/product-home/generated_images/...`.
3. Requested: run `/cost` and open the `gpt-image-2` technical page; actual: `/cost` at 2026-10-16T05:15:56Z; opened `→ Request 3 · OpenAI · gpt-image-2` and its technical details at 2026-10-16T05:16:58Z. (First selected Request 4 by navigation, backed out immediately, then opened Request 3 correctly.)

**Visible `/cost` lines (copied exactly)**
- Screen `ACC21-cost-first`, read 2026-10-16T05:15:56Z: `• OpenAI · GPT-6 Luna — Pay per use. 3 requests, 52,677 tokens. Estimated cost: $0.004207 (rounded).`; `• OpenAI · gpt-image-2 — Pay per use. 1 request, 277 tokens. Estimated cost: $0.007110.`
- Screen `ACC21-cost-details2`, read 2026-10-16T05:16:15Z: `Known subtotal exact USD: 0.011316705`.
- Screen `ACC21-cost-details3`, read 2026-10-16T05:16:15Z: `Total (not separately billed): 52,954`; `Reasoning (subset, not separately billed): not reported (1 attempt)`.
- Screen `ACC21-cost-details4`, read 2026-10-16T05:16:15Z: `→ Request 3 · OpenAI · gpt-image-2 · estimated $0.007110`. Correct selection screen also showed `→ OpenAI · gpt-image-2 — estimated $0.007110 (1 request)`.
- Image request page (`ACC21-image-request-page-correct`): `Provider: OpenAI`; `Model: gpt-image-2`; `Billing: Pay per use`; `Estimated cost: $0.007110`; `Tokens: 277 tokens`.

**Technical page (screens `ACC21-image-technical-up-tokens2`, `ACC21-image-technical-up-tokens`, `ACC21-image-technical-up-cost2`, `ACC21-image-technical-up-cost`)**
- `Turn: image:9124b30d-a475-4846-a4a5-7b0ce966200f` (starts with `image:`).
- Tokens: `Input: 48`; `Noncached input (derived for inclusive input): 48 (derived)`; `Cache read: 0 (reported)`; `Cache write: 0 (reported)`; `Output: 229`; `Reasoning (subset, not separately billed): unknown — no retained numeric evidence`; `Total (not separately billed): 277`.
- Costs: `Noncached input cost: $0.000240; exact USD 0.00024`; `Cache read cost: $0.000000; exact USD 0`; `Cache write cost: none — 0 tokens`; `Output cost: $0.006870; exact USD 0.00687`; `Known subtotal exact USD: 0.00711`.
- Rates: `Noncached input rate: 5 USD per million tokens`; `Cache read rate: 1.25 USD per million tokens`; `Rate unavailable for Cache write`; `Output rate: 30 USD per million tokens`.
- Recomputation: 48×5/M = $0.00024; 0×1.25/M = $0; cache write unavailable with 0 tokens; 229×30/M = $0.00687; sum = $0.00711. This matches exactly.
- The zero-token cache-read component with a published rate kept `$0.000000`; the zero-token cache-write component with no rate read `none — 0 tokens`, as required.

**Outcome**
- Expected result 1 (separate `OpenAI · gpt-image-2 — Pay per use` line with 1 request): **passed**.
- Expected result 2 (technical page has `Turn:` starting `image:`, input rate 5, output rate 30, exact arithmetic): **passed**.
- Expected result 3 (zero-token wording for rate/no-rate components): **passed**.
- Expected result 4 (image request counted once): **passed**.

## ACC-22 — priority tier pricing behavior

**Requested actions / actual**
1. Requested: same profile and a new conversation on `gpt-6-luna` with `-c service_tier="priority"`; actual: started at 2026-10-16T05:18:50Z. The initial screen showed: `⚠ Configured service tier \`priority\` is not advertised as supported for model \`gpt-6-luna\` and will be omitted from requests.` The literal starting condition was supplied, but the product reported it omitted the tier.
2. Requested: type the literal prompt; actual: typed and verified `Reply with the single word OK.` at 2026-10-16T05:19:04Z; pressed Enter at 2026-10-16T05:19:09Z. Reply at 2026-10-16T05:19:29Z was `• OK`.
3. Requested: run `/cost` and open technical pages; actual: `/cost` at 2026-10-16T05:19:32Z; opened Request 2 and Request 1 technical details and inspected all component/rate lines. No priority tier field or unknown-rate wording appeared.

**Visible `/cost` lines (copied exactly)**
- Screen `ACC22-cost-first`, read 2026-10-16T05:19:32Z: `• OpenAI · GPT-6 Luna — Pay per use. 2 requests, 32,479 tokens. Estimated cost: $0.003696 (rounded).`
- Screen `ACC22-cost-first`: `Other conversations' estimated cost: $0.196180 (rounded).`; other conversation lines for `GPT-5.6-Sol` `$0.110939`, `GPT-6 Luna` `$0.004207`, `GPT-6.1 Sol` `$0.073924`, and `gpt-image-2` `$0.007110`.
- Screen `ACC22-cost-details2`: `Known subtotal exact USD: 0.003695525`.
- Screen `ACC22-cost-details3`: `Total (not separately billed): 32,479`; `Unknown parent population: 0 attempts, excluded from root total`.
- Screen `ACC22-cost-details4`: `→ Request 1 · OpenAI · GPT-6 Luna · estimated $0.001465 (rounded)`; `→ Request 2 · OpenAI · GPT-6 Luna · estimated $0.002231 (rounded)`.

**Request 2 technical page**
- Screen `ACC22-request2-technical-up-tokens2`: `Input: 17828`; `Noncached input (derived for inclusive input): 3 (derived)`; `Cache read: 0 (reported)`; `Cache write: 17825`; `Output: 5`; `Total (not separately billed): 17833`.
- Screen `ACC22-request2-technical-costs3`: `Noncached input cost: $0.000000 (rounded) (nonzero, less than $0.000001); exact USD 0.0000003`; `Cache read cost: $0.000000; exact USD 0`; `Cache write cost: $0.002228 (rounded); exact USD 0.002228125`.
- Screen `ACC22-request2-technical-up-cost`: `Cache write cost: $0.002228 (rounded); exact USD 0.002228125`; `Output cost: $0.000002 (rounded); exact USD 0.0000025`; `Known subtotal exact USD: 0.002230925`.
- Rates: `Noncached input rate: 0.1`; `Cache read rate: 0.01`; `Cache write rate: 0.125`; `Output rate: 0.5 USD per million tokens`; long-context tier `0.2`, `0.02`, `0.25`, `0.75` marked `Long-context price not applied: input 17828 ≤ 272000`.
- Recomputation: 3×0.1/M = $0.0000003; 0×0.01/M = $0; 17,825×0.125/M = $0.002228125; 5×0.5/M = $0.0000025; sum = $0.002230925. This matches, but it prices the normal tier.

**Request 1 technical page**
- Screen `ACC22-request1-technical-up-tokens2`: `Input: 14646`; `Noncached input (derived for inclusive input): 14646 (derived)`; cache read/write/output all `0 (reported)`.
- Screen `ACC22-request1-technical-up-cost`: `Noncached input cost: $0.001465 (rounded); exact USD 0.0014646`; cache read/write/output each `$0.000000`; `Known subtotal exact USD: 0.0014646`.
- Rates: `Noncached input rate: 0.1`; `Cache read rate: 0.01`; `Cache write rate: 0.125`; `Output rate: 0.5 USD per million tokens`; long-context tier `0.2`, `0.02`, `0.25`, `0.75` marked not applied.
- Recomputation: 14,646×0.1/M = $0.0014646; all other components = $0; sum = $0.0014646.

**Outcome**
- Product start supplied the requested config, but the product visibly reported the priority tier unsupported and omitted it, so the intended priority-price fallback path was not exercised.
- Expected `no price available` on the first screen: **failed** (screen showed normal `Pay per use` pricing).
- Expected `unknown — rate unavailable` components: **failed** (components had normal published rates).
- Expected `none — 0 tokens` wording for 0-token components: **failed** (0-token cache read showed `$0.000000`).
- Expected no `$0.000000` for any unpriced component: **not exercised** (there was no unpriced priority component).
- Expected `Known subtotal exact USD: none`: **failed** (subtotals were shown).
- Expected other conversations keep their prices: **passed** (the previously observed conversation costs remained on screen and summed to `0.110939 + 0.004207 + 0.073924 + 0.007110 = 0.196180`).

## ACC-34 — credential output pattern scan (group D)

**Requested actions / actual**
1. Requested: pattern-scan all captured screens, all exec output, and workspace files; actual: there were no `agms-tui exec` invocations in this group, so exec-output scan was not exercised. At 2026-10-16T05:24:10Z the product was stopped and `agms-tui fixture ACC34-fixture` recorded the workspace files. At 2026-10-16T05:24:21Z a literal `grep -nE 'agms-synthetic|sk-|Bearer |Authorization|api_key|token='` scan was run over `$AGMS_RUN/evidence/screens.jsonl`, `$AGMS_RUN/evidence/actions.jsonl`, and the workspace files (excluding binary `.git/index`).
2. Observed: no scan hit in captured screens, actions, or workspace files. The `.env` contained `NOTES_SYNC_TOKEN=agms-canary-7f3a9c-synthetic-token`, which is not a match for the specified `agms-synthetic` pattern. The normal product UI warning visible during OpenAI cases was `⚠ Using OPENAI_API_KEY from your environment — billed per use to that API key. Sign in or unset it to change.` It shows an environment-variable name, not a key value; the literal lowercase `api_key` pattern did not hit.

**Outcome**
- Expected no pattern hit: **passed**.
- `exec` output coverage: **not exercised** (no exec actions were requested by ACC-19 through ACC-22).
- Ambiguity preserved: the visible `OPENAI_API_KEY` environment-variable name is not a credential value, but the exact distinction between “no API key text” and “no API key value” is not defined in the packet. No key value was observed.

## Group end

- No model refusal or decline appeared in any assigned case.
- Group end UTC: 2026-10-16T05:24:21Z.
- Product stopped at 2026-10-16T05:24:10Z before the final fixture scan.
- Product-home contents were inspected only at file-name level after execution; no conversation/history source was read. 

- Supplemental raw PTY scan at 2026-10-16T05:26:11Z over `$AGMS_RUN/evidence/pty.raw` also produced no hit for the specified patterns.
## Final verdicts

VERDICT ACC-19: passed
VERDICT ACC-20: passed
VERDICT ACC-21: passed
VERDICT ACC-22: failed
VERDICT ACC-34: passed

