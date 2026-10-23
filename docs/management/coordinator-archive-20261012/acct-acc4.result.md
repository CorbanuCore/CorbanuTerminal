PF-60-S04 passes on current main: every criterion I could run with real keys through the TUI passed, and every pay-per-use total recomputed exactly. The evidence is in **PR #397**, which is merged (`d5004cfedf`) after all 21 checks passed. I tested commit `b1e20a8ec6`; its Rust code is identical to the commit the lane qualified (`1e9cd464b9`).

| # | Criterion | Verdict | Evidence |
|---|---|---|---|
| 1 | OpenAI `gpt-5.4` parent with GLM 5.3 Flash, Kimi k3 and Claude children | PASS | 11 attempts (7 parent, 4 children); `/cost` total 0.10371118 equals my recompute; Kimi and Claude shown as "Covered by your subscription" |
| 1b | GLM 5.3 Flash parent with Kimi and Claude children | PASS | Z.AI 0.00300274, exact; Kimi and Claude kept as subscription |
| 2 | `/side` recorded under its conversation | PASS | Recorded on the parent's thread as `side:prewarm:…` and `side:…`; both amounts exact |
| 3 | Memory consolidation recorded under its conversation | PASS | `consolidation:…` on session A's thread; 14,946 tokens, matching the provider's figure |
| 4 | `exec --ephemeral` excluded | PASS | Exactly one warning and no ledger rows; the same command without `--ephemeral` recorded one row |
| 5 | Unpriced request shows "Price: none recorded" | PASS | One line on six pages; no Price ID or source on any of them; priced pages keep the full block |
| 6 | Cancellation | PASS | Shows "usage incomplete" and "at least $0.162512"; the provider never reported usage for that request |
| 7 | Missing price | PASS | Fast-tier requests show "no price available"; the known subtotal is unchanged |
| 8 | Duplicate retry | PASS | 1 request, 2 attempts; attempt 2 links to attempt 1; tokens (5,538 / 34) counted once and match the proxy log |
| 9 | Reopen with `/resume` | PASS | Earlier requests identical; one new paid prewarm ($0.025) recorded |
| 10 | Kill -9 and restart | PASS | Requests 1–15 identical; total 0.22105268, exact |
| 11 | Historical inspection | PASS | 2026-10-07, 2026-09-28 and 2026-10-09 equal per-day sums taken straight from the ledger |
| 12 | `/cost` date bounds and refusals | PASS | Day, week and month ranges match the ledger at the displayed rounding; future day, reversed range, future start, bad grouping, malformed date and before-retention are each refused with a reason |
| 13 | Every pay-per-use total recomputes | PASS | 17 OpenAI and 5 Z.AI requests, from the provider's own usage at published prices |
| 14 | Waiver (a) re-check (OpenAI estimate) | PASS | `gpt-5.4` exact; `gpt-5.6-luna` and `-terra` (the models S05 failed on) exact to 0.06130286, cache writes included |
| 15 | #368 price issues documented only | PASS | Listed in the qualification; the rates I saw equal the published prices (a code-blind run can't prove nothing changed) |
| 16 | Subscription never counted as spent | PASS | Every view keeps Kimi and Claude out of the pay-per-use total |
| 17 | Focused tmux suite | PASS | 71 run, 71 passed, 24 skipped |
| 18 | Billing basis per route | PASS | Kimi Code and Claude plan are subscription; OpenAI API key and Z.AI are pay per use; a custom route reads "set in your config". DeepSeek and GLM 5.2 were not re-run. |
| — | ChatGPT login, image generation, realtime, TensorCash | NOT VERIFIABLE | No ChatGPT login swap; no image tool on the API-key route (S05's finding, not re-checked); realtime is voice-only; `agtico/tensorcash` doesn't resolve |

- **Review:** the code-blind reviewer (Opus 5.5 High, read-only) concluded "Supported with corrections". Its recomputed numbers all matched, and I applied its corrections. They tightened the waiver wording, recorded the isolation deviation, and added the missing audit files.
- **Defect filed:** #396, display only. A page for a request with no price still shows "Cache write cost: $0.000000" for 0 tokens, and the user-declared basis line has nested parentheses. No total is affected.
- **Deviation:** like the lane run and S05, this run used disposable homes and the no-keychain setting but was not inside an enforced isolation gate.
- **Spend:** about $0.42 on the OpenAI key and about $0.006 on Z.AI. Kimi and Claude ran on their subscriptions.
- **Key scan:** all four keys give 0 hits in the diff, the evidence folder and the remote scratch.
- **Cleanup:** the remote scratch (build output, homes, logs) and the local worktree are deleted. The local branch is still there because Git won't delete it without force; it is harmless.

**Recommendation for Travis:** accept PF-60-S04, noting the isolation deviation as you did for S05's waiver (b). Lift waiver (a): OpenAI API-key estimates are now exact to the micro-dollar for `gpt-5.4`, `gpt-5.6-luna` and `gpt-5.6-terra`, cache writes included. A tier with no catalogue price shows "no price" rather than a wrong figure. Not exercised: long-context, Batch/Flex and other OpenAI models.