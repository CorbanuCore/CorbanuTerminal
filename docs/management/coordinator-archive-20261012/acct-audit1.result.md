I wrote the audit to `.codex-work/workers-20261002/acct-audit1.result.md` (72 lines). Travis is right: about 160 accounting commits landed between 2026-09-15 and 2026-10-06, and all of them are already on main. They went in through PR #138, which merged the whole integration branch on 2026-10-04 (45 checks passed). The integration branch has nothing left that isn't on main.

**The paperwork is behind the code:**
- The plan file's text last changed on 09-15. It still says S03 is a draft and collection is off, and it lists three worktrees that no longer exist.
- The S03 sprint file is missing about 70 commits from 09-20 to 10-03: capture coverage, pricing fixes, provider-stated charges, the `/cost` screen rework and the lock-contention fixes. None of that is in its Done list. It also still cites the old pre-rebase SHAs for the acct-scope-62 fix.
- The test and review results for the two contention fixes (acct-failure-63, acct-sweep-64) exist only in the worker result files in `.codex-work`, not in `qa/`.

**Sprint by sprint:**

| Sprint | Status |
|---|---|
| S01 (contract and fixtures) | Done and archived; Travis approved the defaults |
| S02 (ledger, capture on four wire paths) | Done and archived, on main |
| S03 (inspector, ranges, developer-only switch, coverage, pricing, stated charges, `/cost` screens, scope wording, contention fixes) | Merged but not gated |
| S04 (acceptance) | Not started |

What S03 still needs:
- Travis's answer on `acct-s03-acceptance-20260917`, which was never answered.
- One independent review of the work from 09-20 to 10-02. The two contention fixes were already reviewed: "approve with nits".
- Tests for the no-usage, stale-estimate and current-command cases.
- A next step for the unavailable-backend case, and a label that tells missing usage apart from a missing price.
- The ledger and plan updates above, plus demo videos.

Accounting only runs in debug builds with the `developer-accounting` feature, so nothing reaches users yet.

**"Native Anthropic accounting failed":** no GitHub issue was ever filed. The fixes are on main. Since the 10-03 fix, the only real occurrence is a 10-04 run by a worker still on the old binary. The RTX binary already includes the fix. Anything started before 10-03 needs a restart.

**Local-only commits:** nothing needs recovering.
- `wip/acct-activation-26-partial` `ec06d2e434` (CorbanuTerminal, Mac): a 09-16 work-in-progress commit, replaced by the activation work already on main.
- `cf73b8c1f7`, `4946e1e49d`, `a370fc67cb` (Mac, not on any branch): the pre-rebase acct-scope-62 commits. They landed on main as `eb0d2e3424`, `fd90ddbb74` and `704650ccd8`.
- Five `workstream/accounting-*` branches are 1–2 commits ahead of main, but every commit already has an identical copy there.
- On the RTX box:
  - Every branch, worktree and bundle is on main, except `m047b.bundle`, which is a release fix rather than accounting.
  - The `catalog` worktree has 4 uncommitted catalogue-pricing files from 09-22. A version of that work is already on main.

**Open issues and PRs:** #127 (Grok 4.7 long-prompt pricing understates cost) and its mitigation PR #135, which is mergeable; #129 (OpenRouter cap reconciliation); #124 and #126, which affect spend but not the ledger; and #28, a stale July billing PR that #138 replaces.

**Demo videos I'd record now** (on a disposable home, never the real keychain):
1. `/cost` after a mocked turn: provider and model lines, then into a request and its technical details, then Back/Esc. Then an empty conversation showing other conversations and the missing-price next step. The existing `cost_scope.rs` tmux scenario already sets this up.
2. A past day, and a date range grouped by day or week, on seeded data.
3. Three concurrent `corbanu exec` runs all finishing cleanly, then `/cost` on one of them. This shows the "accounting failed" error is gone.

Stated charges need a mocked or paid provider, so they come later. S04 has nothing to demo yet.

I changed nothing apart from writing the result file. I did run `git fetch origin` once, which only updates the remote-tracking refs, and everything on the RTX box was read-only.