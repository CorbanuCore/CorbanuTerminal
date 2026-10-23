I've finished the code-blind acceptance of PF-60-S05. Everything that could be tested with the credentials in the vault works on real provider calls on macOS and Linux, and every cost I report matches my own recomputation from provider-reported usage to the micro-dollar. One part (AC7) passed only partly, and several parts couldn't be tested because their keys aren't in the vault.

I tested `origin/main` `d7846e29d5` (after #346 merged) with developer-accounting builds on both platforms. The independent reviewer (Opus 5.5 High, read-only) concluded "Supported, with corrections"; I applied the corrections and saved its report as `REVIEW.md`.

| AC | Verdict | Evidence |
|---|---|---|
| AC1 | PASS | With placeholder keys, 20 of 21 built-in providers recorded an attempt with the basis in the defaults table. The built-in `openai` never sends a request with a bad key, so its row rests on a passing unit test. |
| AC2 | PASS; mutation tests and ChatGPT login NOT VERIFIABLE | Bedrock with a command `auth` is pay per use; `kimi-code` with an environment key is subscription; an OpenAI API-key route is pay per use. |
| AC3 | PASS | The `billing` override applies to new turns only and the earlier turn keeps its exact cost. An invalid value is a config error naming the key. A custom provider with no declaration shows "Billing basis not declared" with the next step. |
| AC4 | NOT VERIFIABLE | Needs a ChatGPT login or Claude panes. Instead: subscription routes with no catalogue price show "Covered by your subscription", never "Pay per use", and aren't in the "no price" count. |
| AC5 | NOT VERIFIABLE | No Kimi Code key. A fake-key attempt is recorded as subscription. |
| AC6 | PASS, with caveats | I built the version before the 09-21 format change and wrote a ledger with it; the new build reads it and its token counts match. Real ledgers from 09-22 onward read the same as on the S03-accepted build. A newer-format ledger gives one warning and turns still succeed. No ledger from before 09-21 exists, so that check is recorded as absent. |
| AC7 | PARTIAL | Compaction finished with one gap warning when the ledger refused writes and when it was locked past its budget. With a read-only database, or another process holding it for about 2 minutes, compaction finished but the turn then failed in a request-limit check outside accounting (#351). I couldn't inject a validation failure or a failure after a paid response; web search and image generation need OpenAI keys. |
| AC8 | PASS | Approval reviews, including two running at once, are recorded under the reviewed conversation and its totals match exactly (0.0448102 / 0.012911 / 0.03890804). |
| AC9 | PASS | `exec --ephemeral` logs exactly one `accounting.excluded` warning; a normal run logs none; the 10-09 coverage audit lists it. |
| AC10 | PASS for pay per use; subscription providers NOT VERIFIABLE | Z.AI GLM 5.2 and DeepSeek match exactly. OpenRouter's billed figure equals the cost OpenRouter reports. `claude-plan`, ChatGPT and `kimi-code` couldn't be run for real. |
| AC11 | Done, with one deviation | I froze the test design before reading any S05 results. I ran on the host rather than inside the isolated sandbox the gate describes, as S03's executor did. |
| AC12 | PASS for Z.AI routes; other routes NOT VERIFIABLE | Real requests: the general URL is pay per use; the coding-plan URL and the Anthropic route are subscription; an unknown route is "not declared". The overflow note appears exactly where subscription work appears (65 of 65 views). Kimi, BigModel and OpenAI routes show the right basis with fake keys only. |

The review's findings that you asked about:
- **Subscription counted as spending:** fixed. Coding-plan work, the Anthropic route and the override never add to money spent.
- **Plan work without a price shown as pay per use:** fixed on every route I could reach.
- **Older ledgers:** they still read.
- **Compaction, web search and image:** compaction returns its result; web search and image are untested.
- **Guardian reviews and ephemeral sessions:** both handled.
- **Kimi Code (the original Blocker):** only verified with a fake key.

The record and evidence are in PR #355, merged as a docs-only PR once all checks were green; the key scan found nothing. I filed #351 (the request-limit check that ends the turn) and #352 (small `/cost` display mistakes, such as Ollama and LM Studio both showing as "gpt-oss"). Scratch homes, build folders and worktrees on both machines are removed. The QA runs cost about $1.17 at list prices.

**Recommendation for Travis:** accept PF-60-S05. The accounting does what the sprint asked: subscription work is never counted as money spent, unknown routes say so instead of guessing, old ledgers still read, and every total checked out. #351 is worth a small follow-up outside accounting so a stuck database doesn't end a turn. To close the untested parts, provide:
- a Kimi Code membership key;
- a ChatGPT login;
- an OpenAI API key;
- a Claude plan token that a test build is allowed to use.