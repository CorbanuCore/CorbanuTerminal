# Group B functional execution report

This is an automated functional execution report, not human acceptance.

## Identity, isolation, and setup

- Executor probe: `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?` returned `rc=0`; the saved JSON has top-level `"ok": true`.
- Coordinator files checked: `preflight-summary.json`, `preflight-launcher.json`, and `preflight-tmux-child.json`. The summary has `"ok": true` and `"failed": []`.
- Product binary: `/opt/agms/pkg/bin/corbanu`, SHA-256 `5b805b9c244e55c12c6f61fc3d31bddd760a415554cbabff8cbcb4c3ba0531a1`; probe reported `corbanu 0.1.48`.
- Packet digests verified: design `f750b006046cb5036f92fd6f4f8d910ccd8aa33f88b33c72ffefd0f6cc01bfd6`; amendments `2a73b6503429dc5a6d989b313e996a9041f5dc60b2c130e329dd39e1d71f105e`; answers `dc4044d36ac3868205ff2dc71cee36805969c6896c683852a6f9d705dc2d7c0c`.
- Group start UTC: 2026-10-16T03:32:38Z. The `ledger-notes` fixture was present under `$AGMS_RUN/fixture/workspace`. The product profile directory was reset to empty at 2026-10-16T03:34:11Z.
- The product was driven only with `agms-tui`. Prompt text and Enter were separate actions and the typed text was checked on screen before Enter.
- Model declines/refusals: none observed.
- Group end UTC: 2026-10-16T03:49:04Z.
- D3 file: `D3 NOT GRANTED: do not run ACC-35; record it as not checked live; covered by test`.

## ACC-08 — `kimi-k3` pricing

### Requested versus actual actions

Requested: fresh profile; new Moonshot conversation with route `moonshot`, model `kimi-k3`, provider `kimi`; exactly one prompt; `/cost`; open each request and technical page.

Actual:
1. 2026-10-16T03:35:03Z (`ACC08-start`): product showed composer for `kimi-k3 via kimi default`. A directory-trust menu appeared during startup; `1. Yes, continue` was accepted with Enter.
2. 2026-10-16T03:35:10Z (`ACC08-prompt-typed`): typed `Reply with the single word OK.`; screen matched exactly. Sent it with Enter at the same UTC second.
3. 2026-10-16T03:36:29Z (`ACC08-after60`): reply screen showed `• OK`. The first `agms-tui wait 'OK'` had matched the prompt text already containing `OK`, so the exact moment the reply first appeared is not isolated; the screen at 03:36:29 is the first isolated confirmation.
4. 2026-10-16T03:36:38Z (`ACC08-cost-first`): opened `/cost`; first-screen summary was captured.
5. Initial request drilldown attempts (`Down` + `Enter`, scrolling, Esc/reopen `/cost`, and `C-m`) did not visibly change the page. Later, after ACC-09, I used `/resume` to reopen the ACC-08 conversation without sending a model turn, opened `/cost`, scrolled to its `—— Open ——` section, opened Request 1, and opened Technical details. This was neutral navigation only. Consequently, the request/technical pages were captured later than the requested immediate capture: Request at 2026-10-16T03:46:11Z and technical through 2026-10-16T03:47:29Z. The recorded request and costs were unchanged.
6. No prompt was retried and no other model turn was sent.

### `/cost` and request/technical evidence

From `ACC08-cost-first` at 2026-10-16T03:36:38Z:

```text
• kimi · kimi-k3 — Pay per use. 1 request, 11,931 tokens. Estimated cost: $0.036213.
No other conversation recorded requests on this day.
Estimated token cost for recorded attempts: $0.036213
```

From `ACC08-cost-open-bottom` at 2026-10-16T03:46:02Z:

```text
→ kimi · kimi-k3 — estimated $0.036213 (1 request)
→ Request 1 · kimi · kimi-k3 · estimated $0.036213
```

From `ACC08-request-top` at 2026-10-16T03:46:11Z:

```text
Provider: kimi
Model: kimi-k3
Billing: Pay per use
Estimated cost: $0.036213
Tokens: 11,931 tokens
Request: 0f63f838-d81b-4c34-9a9d-a1118942df7d
```

From `ACC08-technical-line25` through `ACC08-technical-line39` at 2026-10-16T03:47:26–03:47:29Z:

```text
Billing basis: pay per use
Input: 11896
Noncached input (derived for inclusive input): 120 (derived)
Cache read: 0 (reported)
Cache write: 11776
Output: 35
Reasoning (subset, not separately billed): 19
Total (not separately billed): 11931
Noncached input cost: $0.000360; exact USD 0.00036
Cache read cost: $0.000000; exact USD 0
Cache write cost: $0.035328; exact USD 0.035328
Output cost: $0.000525; exact USD 0.000525
Known subtotal exact USD: 0.036213
Noncached input rate: 3 USD per million tokens
Cache read rate: 0.3 USD per million tokens
Cache write rate: 3 USD per million tokens
Output rate: 15 USD per million tokens
```

The technical page also showed `Attempt: 453c114f-693f-480e-858a-0ca7114311a9` and admission time `2026-10-16T03:35:18.031Z`.

### Recomputation

- Noncached input: `11896 − 0 − 11776 = 120`.
- Noncached input cost: `120 × 3 / 1,000,000 = 0.000360`.
- Cache read cost: `0 × 0.3 / 1,000,000 = 0`.
- Cache write cost: `11776 × 3 / 1,000,000 = 0.035328`.
- Output cost: `35 × 15 / 1,000,000 = 0.000525`.
- Known subtotal: `0.000360 + 0 + 0.035328 + 0.000525 = 0.036213`.

### Expected-result outcomes

- First screen names `kimi · kimi-k3`, is Pay per use, and has an estimated cost: **passed**.
- Rates are 3 / 0.3 / 3 / 15 USD per million tokens: **passed**.
- Noncached input equals input minus cache read minus cache write: **passed**.
- Each component cost equals tokens × rate, and the exact known subtotal equals the headline: **passed**.
- Cache read and cache write are numeric; neither is unknown or not reported: **passed**.
- No `at least`, `lower bound`, or `+ unknown` appears in this group's captured screens: **passed**.

Observation/ambiguity: immediate technical capture did not occur because the first cost drilldown method did not work; it was completed later through neutral `/resume` navigation without an additional model turn.

VERDICT ACC-08: passed

## ACC-35 — optional second Kimi turns

D3 was not granted: `D3 NOT GRANTED: do not run ACC-35; record it as not checked live; covered by test`. No second Moonshot turn was sent.

VERDICT ACC-35: blocked

## ACC-09 — `kimi-k2.7-code` pricing with provider `moonlab`

### Requested versus actual actions

Requested: same profile; stop; start `--route moonshot --provider-name moonlab --model kimi-k2.7-code`; new conversation; exactly one prompt; `/cost`; open request/technical pages.

Actual:
1. 2026-10-16T03:39:31Z: stopped the ACC-08 terminal and started the requested command.
2. 2026-10-16T03:39:44Z (`ACC09-start`): first screen showed `kimi-k2.7-code via moonlab default`.
3. 2026-10-16T03:39:45Z (`ACC09-prompt-typed`): typed `Reply with the single word OK.`; screen matched exactly. Sent it with Enter at the same UTC second.
4. 2026-10-16T03:40:52Z (`ACC09-reply-after60`): reply screen showed `• OK`. The screen was checked after 60 seconds; the exact moment the reply first appeared is not isolated.
5. 2026-10-16T03:40:59Z (`ACC09-cost-first`): opened `/cost`.
6. 2026-10-16T03:42:18Z (`ACC09-request-page`): opened Request 1.
7. 2026-10-16T03:42:35Z (`ACC09-technical-top`) through 2026-10-16T03:44:29Z (`ACC09-technical-costs-cache`): opened and captured Technical details.
8. No prompt was retried and no second model turn was sent.

### `/cost` and request/technical evidence

From `ACC09-cost-first` at 2026-10-16T03:40:59Z:

```text
• moonlab · kimi-k2.7-code — Pay per use. 1 request, 11,496 tokens. Estimated cost: $0.010991 (rounded).
Other conversations on this day, not included above: 1 request in 1 conversation.
• kimi · kimi-k3 — Pay per use. 1 request, 11,931 tokens. Estimated cost: $0.036213.
Other conversations' estimated cost: $0.036213.
```

From `ACC09-request-page` at 2026-10-16T03:42:18Z:

```text
Provider: moonlab
Model: kimi-k2.7-code
Billing: Pay per use
Estimated cost: $0.010991 (rounded)
Tokens: 11,496 tokens
Request: fcec2bea-073b-4a2e-b5cd-f348b08e8852
```

From `ACC09-technical-costs-cache` through `ACC09-technical-costs-rates` at 2026-10-16T03:44:14–03:44:29Z:

```text
Input: 11473
Noncached input (derived for inclusive input): 11473 (derived)
Cache read: 0 (reported)
Cache write: 0 (reported)
Output: 23
Reasoning (subset, not separately billed): 20
Total (not separately billed): 11496
Noncached input cost: $0.010899 (rounded); exact USD 0.01089935
Cache read cost: $0.000000; exact USD 0
Cache write cost: $0.000000; exact USD 0
Output cost: $0.000092; exact USD 0.000092
Known subtotal exact USD: 0.01099135
Noncached input rate: 0.95 USD per million tokens
Cache read rate: 0.19 USD per million tokens
Cache write rate: 0.95 USD per million tokens
Output rate: 4 USD per million tokens
```

The technical page also showed `Attempt: e06d802f-b1e7-4c70-beed-a94e3975a5ac` and admission time `2026-10-16T03:39:52.246Z`.

### Recomputation

- Noncached input: `11473 − 0 − 0 = 11473`.
- Noncached input cost: `11473 × 0.95 / 1,000,000 = 0.01089935`.
- Cache read cost: `0 × 0.19 / 1,000,000 = 0`.
- Cache write cost: `0 × 0.95 / 1,000,000 = 0`.
- Output cost: `23 × 4 / 1,000,000 = 0.000092`.
- Known subtotal: `0.01089935 + 0 + 0 + 0.000092 = 0.01099135`.

### Expected-result outcomes

- First screen names `moonlab` and `kimi-k2.7-code`, is Pay per use, and has an estimated cost: **passed**.
- Rates are 0.95 / 0.19 / 0.95 / 4 USD per million tokens: **passed**.
- ACC-08 arithmetic and completeness checks hold, including when zero cache counts are reported: **passed**.
- Other conversations lists `kimi · kimi-k3` once at exactly the last observed ACC-08 total: **passed**.
- No `at least`, `lower bound`, or `+ unknown` appears in this group's captured screens: **passed**.

Observation: when ACC-08 was reopened for late technical capture, the current session model was `kimi-k2.7-code`; the product warned that the older transcript had been recorded with `kimi-k3`. The ACC-08 `/cost` page still showed `kimi · kimi-k3`.

VERDICT ACC-09: passed

## ACC-34 — no credential is printed

### Requested versus actual actions

Requested: pattern-scan every captured screen, all exec output, and workspace files for `agms-synthetic`, `sk-`, `Bearer `, `Authorization`, `api_key`, and `token=`.

Actual:
- Starting UTC: 2026-10-16T03:48:07Z.
- Scanned the captured screen/action/PTY/fixture evidence files for this group and recursively scanned `$AGMS_RUN/fixture/workspace` with the six patterns.
- Recorded final fixture `ACC34-final` before the final scan at 2026-10-16T03:49:04Z.
- There was no `exec` output in this assigned group.

### Evidence

Scan result for captured screens/actions/PTY/fixture evidence and workspace files: **no matches** for any requested pattern. The only synthetic-looking fixture value observed was the pre-existing fixture line `NOTES_SYNC_TOKEN=agms-canary-7f3a9c-synthetic-token`, which does not match the requested patterns.

### Expected-result outcome

- No API key, bearer token, sign-in token, or Authorization header text appears in the group's captured screens/output/workspace scan: **passed**.

VERDICT ACC-34: passed
