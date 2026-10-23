S03's listed gaps are closed and four PRs are merged into main. S03 is still open and not accepted: the independent review of the 09-20..10-02 work came back REQUEST CHANGES, and it found problems in collection and pricing, not in `/cost` itself.

## PRs (all merged with `--merge --match-head-commit`, no force-pushes)
| PR | What it did |
|---|---|
| #283 | **Paperwork.** The S03 Done list now has the 171 accounting commits from 09-20 to 10-03, by theme, with their main SHAs. The full list is in `qa/.../pf-60-s03/commits-20260920-20261003.md`. The acct-scope-62 SHAs that pointed at pre-rebase commits are corrected. The acct-failure-63 and acct-sweep-64 results, reviews, test summaries and run logs are now under `qa/.../pf-60-s03/`. The plan file has a current-state section: one active worktree, collection developer-only, S03 in progress. |
| #291 | **Code (`/cost` only).** A request that has a price but is missing a token count now says "usage incomplete" instead of "no price available". Every state where the backend is unavailable now shows a "Next step:" line, using whichever command the build actually has; shipped-build wording is unchanged. Error messages from `/cost` name `/cost`. Regression tests cover no-usage, a stale estimate, the current command, the next-step lines and a day with only subscription work. |
| #297 | Three demo specs and their index, the independent review record, and the ledger updates. The demo script gained two steps that run off screen: one to seed data, one to reuse a session id. A small example program (`accounting_demo_seed`) seeds the data. |
| #305 | **Fixes #288 (closed).** Any pay-per-use request without a complete estimate now gets a next step that names its own provider. There are three cases: no price, priced only in part, and incomplete usage. A request whose reported counts are all zero gets no step. |

## Gate evidence
- **Tests:** `just test -p codex-tui …` passes on the final tree: 214/214 on the default build and 215/215 with `developer-accounting`. The existing `cost_scope` tmux test sometimes drops its first `/cost` keystroke and then passes on retry; that happened before these changes too.
- **Linux clippy on the RTX** (`-D warnings`, with and without the feature): clean on every head I pushed.
- **Opus 5.5 High review of each code PR:**
  - #291: REQUEST CHANGES, all 9 findings fixed, then APPROVE WITH NITS. Four of the five nits were fixed; the fifth is minor duplication.
  - #305: REQUEST CHANGES, then APPROVE WITH NITS. One edge case is left open and recorded in the sprint file: on a price sheet that sets cache writes to zero but leaves out another rate, a request priced only in part can be labelled "no price".
- **CI:** every check passed. On #297 the Linux credential canary failed once and passed when re-run; that PR only changed docs and the demo script.

## Demo videos
All three use GLM 5.2 on zai, a disposable home and a developer-accounting debug build. The data seeding is stated on the title card. Index: `qa/demos/index/PF-60-S03.md`.
- [`/cost` drill-down, Back/Esc, then an empty conversation with the missing-price next step](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-cost-drilldown-and-scope-5ba0c02f6705-2026-10-08.mp4)
- [A past day, then date ranges grouped by day and by week](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-cost-history-ranges-5ba0c02f6705-2026-10-08.mp4)
- [Three concurrent `corbanu exec` runs: all exit 0 with no accounting failures; then `/cost` on one](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-concurrent-exec-then-cost-5ba0c02f6705-2026-10-08.mp4)

These were recorded at `5ba0c02f67`, so they predate the #305 wording changes.

## Independent review of the 09-20..10-02 work
Installed `corbanu exec` on claude-plan, read-only. Verdict: **REQUEST CHANGES**. The record is in `qa/.../body-review-20261008/`, and every open item is in the S03 Remaining list.
- **Blocker:** `kimi-code` subscription work is recorded as money spent. I haven't checked the reviewer's reading of Kimi's terms myself.
- **Major:** plan work with no catalogue price shows as "Pay per use".
- **Major:** a 09-21 change to how stored prices are written wasn't versioned. I couldn't reproduce this; a copy of the live ledger passed validation on 10-03.
- **Major:** compaction, web search and image generation still fail after a paid response if accounting can't record it.
- **Major:** ephemeral sessions and guardian review forks aren't recorded, and nothing is logged about it.
- **Also:** 5 minor findings and 4 nits. The one about `/cost` wording is fixed.

## What `acct-s03-acceptance-20260917` should now be based on
1. **The light gate is met for S03's own deliverables:** the inspector, the date ranges, the scope wording and next steps. The evidence is the tests, Linux clippy, reviews and videos above.
2. **Another worker ran an independent functional acceptance (PR #292). Its verdict was "not accepted".** Since then:
   - #288 is fixed and closed.
   - #286 (deleted conversations erasing their spend) was addressed by #303.
   - #287 (the clock running behind the ledger) was addressed by #299.

   #286 and #287 are both still open as issues, and nobody has re-run those acceptance cases yet. The wording issues in #289 are open.
3. **The review's open findings are about collection and pricing, which is S02 territory, not the inspector.**

**Recommendation:**
- **Accept S03's inspector scope only after** #286 and #287 are re-run and pass, and only if #289 is either fixed or explicitly waived.
- **Put the Blocker and the four Majors in a new collection-correctness sprint before S04.** That needs your call on how each provider's billing model is declared.
- **Keep `/cost` developer-only.**

## Cleanup
- I removed my 64 GB build directory, the demo outputs and everything I created on the RTX box.
- I kept the source checkout `worktrees/pf60-s03-20261008`, now on `work/pf60-s03-20261008` at main, because the plan and sprint file name it as the active S03 checkout and S03 is still open.