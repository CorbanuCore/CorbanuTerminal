PF-60-S05 is archived. The docs-only PR is **#373**, merged into main as merge commit `50cf25df9e`. All checks were green when I merged: 21 passed, 12 skipped, 0 failed, 0 pending. I merged with `--match-head-commit 221a452ccf`.

**Checker output**

| Check | Branch | After merging current main into it | Main after merge |
|---|---|---|---|
| `python3 docs/plans/check.py` | `plans: active 3/3; available slots 0` | same | same |
| `python3 docs/sprints/check.py` | `sprints: current 102; archived 152`, no errors | `current 101; archived 153` | `current 101; archived 153` |

- The plans unit tests (6) and sprints unit tests (24) pass, and `git diff --check` is clean.
- No checker refused anything.
- The count changed after merging main because the PF-27-S08 archive (#364) landed while my PR was open.

**What was archived**
- **Record:** it moved to `docs/sprints/archive/portfolio-agent-cost-accounting/` with status `completed`. A new "Closure — 2026-10-09" section contains:
  - Travis's acceptance and both waivers, word for word as you gave them;
  - the evidence links: the first run (#355) and the re-run (#362), each with its code-blind review, plus the slice PRs and merge commits and the demo index;
  - the known gaps, marked "carried, not discharged";
  - every follow-up you listed, including the two items recommended for S04.
- **Results:** every Verification AC now has a recorded result. AC10's OpenAI API-key estimate is marked as waiver (a) and AC11's isolation gate as waiver (b).
- **Plan:** the current-state text, the S05 row in the worktree table and the S05 sprint-map row now point to the archived record. The S05 worktree stays in the plan's front matter because the plans checker requires one, and the text marks it as released.
- **Index and navigation:** the accounting row in `docs/sprints/index.md` is updated, and the S05 line is removed from `mkdocs.yml`.

**CI hiccup:** Codespell and python-sdk failed on several attempts because Docker Hub returned 429 and 504 errors. #369 hit the same failures. I re-ran only the failed jobs until they passed and didn't change anything to get them through.

**Left alone because it's outside S05's lines**
- **Two stale links:** S03's archived record still links to S05's old `current/` path in two places (its Closure and Remaining sections). Those links are now broken, and it's a one-line fix if you want it.
- **Historical paths:** the TaskNode evidence files under `docs/research/tasknode-integration/` still name S05's `current/` path. I kept them as historical evidence, as was done for S03.
- **Wording:** #370 merged while my PR was open. The record still says image generation is "being added", matching the accepted wording.

My worktree is removed and there was no build output to clean up.