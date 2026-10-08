# Independent review of the 2026-09-20..10-02 accounting body (2026-10-08)

Reviewer: installed `corbanu exec`, `claude-opus-5-5-plan` on `claude-plan`, reasoning high, read-only sandbox
(`reviewer.txt`), against origin/main `63ea3d0cbd`. Prompt: `review-prompt.md`. Report: `review.md`.
The 10-03 contention commits were excluded (reviewed in acct-failure-63 / acct-sweep-64).

**Verdict: REQUEST CHANGES.** Findings are open; none were fixed in this pass except part of #8.

| # | Severity | Finding (short) | Status |
|---|---|---|---|
| 1 | Blocker | `kimi-code` (subscription endpoint, env API key) is priced as pay-per-use spend because the API-key branch infers "pay per use" from the auth mode (`aae4ba660b`) | Open; product decision on per-provider billing model. Reviewer's reading of Kimi's terms not independently verified |
| 2 | Major | Plan work with no catalogue price has no plan-basis price record, so `/cost` calls it "Pay per use · no price available" (ChatGPT-plan models without billing rows, priority tier, image/realtime, panes bridge) | Open |
| 3 | Major | `5bae03414e` changed serialized price/estimate bytes without a rules version: ledgers written before 09-21 fail validation and close accounting; older builds cannot read newer `Patch` | Open, not reproduced: a copy of the live developer ledger (6,971 records) validated during acct-failure-63, so the trigger may be narrower |
| 4 | Major | Compaction, web search and image generation still fail the operation when observation fails after a paid response | Open |
| 5 | Major | Ephemeral sessions, including guardian review forks, are uncollected with no log line (`1aca553905`) | Open |
| 6-7 | Minor | OpenRouter missing cache-write stored as observed 0; Responses path reads cache writes on every route | Open |
| 8 | Minor | `/cost` wording: missing counts read as "no price"; subscription-only days point at a bill | **Fixed** in the PF-60-S03 code slice (usage-incomplete label, plan-only footer); "no price available" beside priced-zero attempts remains |
| 9 | Minor | Pane-bridge warn log includes `base_url` | Open |
| 10 | Minor | Any provider with a command login (`auth`) counts as subscription | Open |
| 11-14 | Nit | Global vs provider auth source; two sites price by wire name; pane report failure at debug only; holiday vs weekday for overnight windows | Open |
