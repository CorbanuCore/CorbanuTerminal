Travis needs to accept eight new Task Node tasks, 3.5 PFT each (28 PFT total). Nothing has been submitted. The map and evidence records are merged on main in docs-only PR #357 (merge `d9e851b380`), after all checks passed (21 passed, 12 skipped) and with `--match-head-commit`.

| Ref | Task id | Title | PFT |
|---|---|---|---|
| SECACCT-TN-01 | `task_95e3a96c397ef87cbbba10a3036f91ef` | Compile the PF-27-S06 and S07 Windows Gate Closure Evidence Record | 3.5 |
| SECACCT-TN-02 | `task_9e695438561fca22216b86eccb784f09` | Compile the SECACCT-TN-02 Seven-Fix Security Evidence Record | 3.5 |
| SECACCT-TN-03 | `task_ee52fe14f4a911a695cf1849a3d1d6e9` | Compile the SECACCT-TN-03 PF-27-S08 Broker Token Evidence Record | 3.5 |
| SECACCT-TN-04 | `task_7e2280a41804bcd4f1e98af473c78937` | Compile the SECACCT-TN-04 Eight-Fix Cost Accounting Evidence Record | 3.5 |
| SECACCT-TN-05 | `task_196d0de5eb848ffc237328b28aadfd02` | Compile the SECACCT-TN-05 PF-60-S03 Acceptance Evidence Record | 3.5 |
| SECACCT-TN-06 | `task_2fe7008e72409e148dc44f7c176c3a10` | Compile the SECACCT-TN-06 Cost Accounting Reel Evidence Record | 3.5 |
| SECACCT-TN-07 | `task_05f44e6ca71a4a02867e640f71ad72e5` | Compile the SECACCT-TN-07 PF-60-S05 Awaiting-Acceptance Evidence Record | 3.5 |
| SECACCT-TN-08 | `task_646f61b2412189a65917a65719be72d6` | Compile the SECACCT-TN-08 CI Health Three-Fix Evidence Record | 3.5 |

**What each task covers:**
- **01:** the PF-27-S06 gate rerun and its two defect fixes, PF-27-S07, and the archive of both.
- **02:** the six Windows follow-up fixes, Windows clippy, and the provider-key fix #313.
- **03:** what PF-27-S08 shipped in #333 only. It says plainly that the sprint is still in progress and waiting for Travis's key-path decision.
- **04:** the PF-60-S03 acceptance fixes plus #339.
- **05:** the PF-60-S03 independent acceptance runs and the archive.
- **06:** the accounting demo videos and the narrated reel, including the Cloudflare copy.
- **07:** PF-60-S05. It says plainly that the sprint is waiting for Travis's acceptance and is not archived.
- **08:** the three CI health fixes.

**Before requesting:**
- **Existing tasks:** P0SEC-TN-10..12 are Rewarded and nothing was waiting on verification. The only open task is the unrelated PostfiatL1V2 bug. The old accounting workstream task was rewarded on 2026-09-15, so I didn't touch it.
- **Merges:** all 43 PRs in these tasks are merged into main.
- **No duplicates:** each request went out once with its tracking tag, after checking for a saved receipt and checking the server for that tag.

**Changes from your list:**
- **More PRs in two tasks:** I added PF-27-S07's record and slices (#280–#284) to TN-01, and #291/#292 to TN-04/TN-05. None of these were in any earlier task.
- **Red-run count:** the postmerge-ci run list shows 83 red runs (79 failed, 4 cancelled), not 40. The #350 description says 84. The first two runs after #350 are green.
- **#331 review:** the Opus APPROVE exists only in the review output on this Mac (`sec-win7-review-300/`), not in the PR. The record says so.
- **Reel index link:** PR #357 also fixes one link in `qa/demos/index/accounting-reel.md` that still pointed at the PF-60-S03 sprint record's old location.
- **P0 map:** PR #357 also marks P0SEC-TN-10..12 Rewarded there.

**Open items, stated as not done in the records:** the PF-27-S08 key-path decision and acceptance criterion 5; Travis's acceptance of PF-60-S05; nobody has yet listened to approve the reel's narration voice; and Breeze's usage terms still need Travis's confirmation.

**After Travis accepts:** the final evidence is ready in `.codex-work/workers-20261009/tn-sec4/evidence-01.md` to `evidence-08.md`. `./submit.sh` there submits only tasks that read Accepted and have no saved receipt; I ran it just now and it skipped all eight. Verification answers then go through `respond.sh`, one answer per request, and the folder's README lists the steps.

- **Evidence size:** the evidence files are 16–46 KB, bigger than earlier rounds (22 KB at most), and I couldn't find a size limit anywhere.
- **If the server rejects one:** `submit.sh` keeps a receipt only when the server says OK. Anything else is saved as a failed file, so that task can be retried by hand.
- **Before submitting 03 or 07:** if PF-27-S08's key path is decided or PF-60-S05 is accepted first, the records need regenerating so they don't describe the old status.

I used the same Task Node account as the earlier rounds and printed no credentials. My worktree is removed. The local branch `docs/tasknode-round4` is still there, because Git wanted a force delete and I didn't force it.