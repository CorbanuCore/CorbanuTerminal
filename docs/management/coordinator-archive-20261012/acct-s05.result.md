I finished PF-60-S05 (option B) and set the sprint to `ready` on main, not archived. Six PRs merged, all green and approved. Two loose ends:
- **RTX cleanup:** the box stopped answering SSH partway through cleanup, so my files there are still in place (list at the end).
- **Unverified routes:** several need credentials that aren't in the vault and are recorded NOT VERIFIABLE.

**PRs**

| PR | What it does | Review |
|---|---|---|
| #334 | Records option B, writes AC12, unblocks the sprint | — |
| #337 | Ledger format: versioned, old ledgers read again, a newer format is refused | APPROVE (round 3) |
| #338 | Declared billing basis per provider, credential and route; the `billing` config key; no more guessing from the type of login | APPROVE (round 3) |
| #342 | `/cost` shows each basis, "Billing basis not declared" with a next step, and the overflow note wherever subscription work appears; every request records its basis and where it came from | APPROVE (round 3) |
| #344 | Guardian reviews recorded under the conversation they review; ephemeral sessions logged once as excluded; newer ledger turns collection off with one warning; compaction, web search and image generation still return results when the ledger fails | APPROVE (rounds 2 and 3; round 1 found a blocker, fixed) |
| #346 | Evidence, demo videos, defaults-table rows, sprint record set to `ready` | — |

**Gate evidence**
- **Tests (macOS, with / without `developer-accounting`):** core 321 / 316, tui 124 / 123, state 398, extensions plus state 416, model-provider-info 74.
- **Tests (Linux on the RTX box):** core 326 / 321, tui 124. This run caught a Linux-only failure in one of my new tests (an unrelated warning on Linux); fixed before merge.
- **Linux clippy `-D warnings`:** 0/0 on every slice's final head.
- **tmux run on GLM 5.2:** `/compact` with the ledger refusing writes finished with exactly one "could not record" warning, and `/cost` showed the three recorded requests.
  - The second injection (another process holding a database lock past the 15 s budget) didn't test accounting alone: the whole turn waited for the lock, then everything recorded normally. That case is covered by an existing automated test instead.
- **Real-provider checks** (`qa/portfolio/agent-cost-accounting/pf-60-s05/real-provider-checks-20261009.md`):
  - Z.AI and DeepSeek cost estimates match my recomputation at published prices exactly. DeepSeek was checked at both its peak and off-peak rates.
  - Z.AI's coding-plan routes record as subscription with $0 spent; the unknown route and the user override record as expected.
  - AC8: a real guardian review was recorded under the reviewed conversation, labelled as a review, and not counted twice.
  - AC9: `exec --ephemeral` logged exactly one exclusion warning.
- **Demo videos** (leak-scanned twice, indexed in `qa/demos/index/PF-60-S05.md`): subscription work not counted as spent, billing basis not declared, overflow note only with subscription work.

**For Travis**
- **NOT VERIFIABLE (credentials not in the vault):**
  - Kimi Code membership key: AC5, plus the Kimi parts of AC10 and AC12.
  - Claude plan login usable in a disposable home: the `claude-plan` part of AC10 and Claude work through panes (AC4).
  - ChatGPT login: the ChatGPT parts of AC4, AC10 and AC12, including the realtime row.
  - OpenAI API key: AC12's API-key route, and AC7 for real web search and image generation.
  - Anthropic API key and BigModel keys: their default routes.
- **ChatGPT realtime row:** I re-checked the Codex pricing page and it still says ChatGPT sign-in usage is part of the plan, but it doesn't mention realtime calls. The row needs a real ChatGPT check.
- **Follow-ups (in the sprint's Remaining list):**
  - `/side` conversations and memory consolidation are paid but still not recorded; they're only logged and listed in the audit. Recommend S04.
  - Setting `billing = "pay_per_use"` on a custom route upgrades the ledger to format 2, which older developer builds can't write to. Release builds aren't affected.
  - A request with no price but a recorded basis still shows a "Price source" line on its detail page.

**Still open**
- **Next step:** AC11, the independent code-blind acceptance, then Travis.
- **Cleanup:** local worktrees and both targets (about 190 GB) are removed. The `pf60-s05-indep-20261009` worktree isn't mine, so I left it.
- **RTX cleanup, pending:** I created these on the box and couldn't remove them:
  - the `~/corbanu-rtx/acct-s05-20261009` worktree;
  - the `acct-s05-target` and `acct-s05-target-feat` target directories;
  - the `acct-s05-*.sh` scripts and logs;
  - the `/tmp/acct-copy-*.sqlite` copies.