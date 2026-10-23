Fixed and merged: **PR #369**, merge commit `9e5ec2b6cc`. #361 is closed with the evidence below, and the follow-ups are in a new issue, **#368**.

**Real check:** one small `gpt-5.6-luna` request on the `openai-api-key` vault key, run in a disposable home with a developer-accounting build. OpenAI reported 9,600 input tokens on the prewarm, then 3 uncached input + 17,328 cache writes + 5 output on the turn. My recompute at the published rates is **$0.0062586**, and `/cost` shows "Known subtotal exact USD: 0.0062586" — equal to the micro-dollar. The key scan found it in 0 files.

**What changed**
- **OpenAI rates** now match OpenAI's pricing page and model pages, read 2026-10-09 (source extract saved in `qa/portfolio/agent-cost-accounting/issue-361/`):
  - gpt-5.6-luna: $0.20 / $0.02 / $1.20 per million (was $1 / $0.10 / $6).
  - gpt-5.6-terra: $2 / $0.20 / $12.
  - gpt-5.6-sol: $4 / $0.40 / $20.
  - Now priced, previously not: gpt-6-luna, gpt-6-astra, gpt-5.5, gpt-5.4, gpt-5.4-mini, gpt-5.3-codex, gpt-5.2.
- **Cache writes** are priced on every GPT-5.6 and GPT-6 model. This also covers the `openai` Chat Completions route; a real request confirmed that route reports cache writes as part of the prompt.
- **Long prompts:** I found that OpenAI charges 2x input and 1.5x output for the whole request above 272K input tokens. Luna, Terra and Sol allow 372K, so without this a long request would get the wrong price. That rate is now stored with each request's price record and applied when the input is above the threshold. If the input count is unknown, the request shows as having no price.
- **Fast/Priority, Flex and Batch** still show "no price", never the Standard rate.
- **Sol's $4/$20 is a promotion** OpenAI offers "at least through November 21, 2026". After 2026-11-21 23:59:59 UTC, Sol API-key requests show "no price" until someone re-reads the page.
- **History is never re-priced:** each request keeps the rates it was recorded with.
- **Refresh tool:** the price catalogue (`models.json`) is hand-maintained, not generated, and there was no refresh tool. I added `scripts/check_openai_api_prices.py`, which compares the catalogue with OpenAI's page.

**Other pay-per-use routes**
- **zai, deepseek, anthropic API key:** match their official pages.
- **openrouter:** left out. The price depends on which upstream provider serves each request (kimi-k3 input ranges from $0.65 to $4.50), so no single catalogue rate is right. Mismatches are listed in #368.
- **Moonshot/Kimi platform and BigModel:** the catalogue has no prices for these routes, so they show "no price" rather than a wrong one. Also in #368.

**One thing to know:** the first OpenAI request on a GPT-5.6, GPT-6, 5.5 or 5.4 model upgrades the cost ledger to a new format, on either an API key or ChatGPT login. An older developer build sharing that state database will then refuse the ledger and stop recording costs. Nothing is lost. This is recorded in the QA README and the PR, and release notes should carry it.

**Gate**
- **Tests:** focused tests pass on macOS with and without developer-accounting, including the exact $0.00625965 recompute from the re-run capture, a cache-write case and a long-context case. They were re-run after merging main.
- **Linux clippy** `-D warnings` on the RTX box is clean in both modes, on the final code.
- **Review:** Opus 5.5 High via the installed `corbanu exec` (read-only), two rounds, both "approve with nits". I fixed all five Minor findings; the round-1 output is in `.codex-work/workers-20261002/fix361-review1/` and round 2 in `fix361-review2/`.
- **PR checks:** all green (27 passed, 9 skipped) before merging with `--match-head-commit`. Docker Hub rate-limit failures were re-run, and I merged main once more to pick up its CI fix.

**Still open** (all in #368):
- Re-read Sol's price before Nov 21.
- Add gpt-6.1-sol to the catalogue.
- Add DeepSeek holidays after 2026-10-07.
- OpenAI doesn't publish a cached-input rate above 272K for gpt-5.5/5.4, so those cache reads stay unpriced.
- Four small tests the reviewer suggested.

**Cleanup:** the build target, worktree, scratch directory and everything I created on the RTX box are removed. The local branch `fix-361-openai-prices` is still in the main checkout: it's merged, but `git branch -d` refused because that checkout isn't on main, and I didn't force-delete it.