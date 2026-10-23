All nine P0SEC-TN tasks are **Rewarded**, for 27.5 PFT, the full amount offered. Task Node's verification tab is now empty. It took about an hour, not the three allowed.

| Ref | PFT | What the verifier asked for | What I answered |
|---|---|---|---|
| 01 | 2 | Proof that the merge commits of #180, #191, #229 and #237 are on main | Raw output of the checks: every exit code 0. Main is now `7e6ef740ef`. |
| 02 | 2.5 | Exact quotes of the Opus review verdicts from the PF-28-S02 gate record | Gate record lines 58–68 (#216, three rounds). The #225 pinning review is `review-opus-4-pinning.md`: verdict APPROVE, all six follow-ups dispositioned. |
| 03 | 2.5 | Proof that #224 (`6d55f6ae8f`) is on main | Exit code 0, plus the link to that commit on GitHub. |
| 04 | 3 | Proof for #228 and #235, plus the current main | Both exit 0. I noted that main has moved on from `bb609b449a` (the head the record names) to `7e6ef740ef`. |
| 05 | 3.5 | Proof for all seven merge commits | Raw output, every exit code 0. |
| 06 | 3.5 | Whether #264 replaced the full PR URLs with bare `#223` references | No. The file still has the full URLs. GitHub's page view shortens a bare PR URL to `#223`. I gave the raw-file link and the exact lines. |
| 07 | 3.5 | Why the PF-24-S03 video index has 18 videos but the sprint record says 9 | There are 9 demos recorded twice: 9 final videos at `7f6c88912c` and an earlier take of the same 9 at `72735863b6`. |
| 08 | 4 | Whether the PF-13-S07 route matrix has 10 or 11 blocked routes | **11.** Each result file has 13 routes: 11 blocked, 0 leaked, 1 not contained, 1 known gap. I pasted the per-route rows from the file. |
| 09 | 3 | Proof that #261 merged at `43b21fc899` | Exit code 0, plus GitHub's merged status and the PR and commit links. |

Every answer says where things were imprecise rather than glossing over it:
- The record's "checked at `bb609b449a`" is out of date: main has since moved to `7e6ef740ef`.
- The README's review section never mentions #225; that review is in a separate file in the same folder.
- Blocked results in the route matrix depend on the model, which may simply have refused the command. The direct probes are the real proof of the fix.

**PF-13-S07 count:** the mismatch was ours to fix. Docs-only PR #266 corrects our evidence record to 11 and logs the change in the task map. It passed CI and is merged with `--match-head-commit` as `b4c60a0589`. I answered task 08 only after the merge, so the answer could link to the fix on main.

**Needs Travis:**
- **Count still wrong elsewhere:** the PF-13-S07 sprint record (line 66) and the PR #255 description still say "10 blocked". Only 10 + 1 + 1 = 12 routes are accounted for, but each file has 13. They belong to another lane, so I didn't change them. I recommend that lane's owner changes both to 11.
- **Same line overstates the `.ssh` result:** it says `.ssh` reads are blocked under both saved levels. The evidence shows the sandbox denied the read in 3 of 4 runs; in the Linux Moderate run the model refused to try.
- **Unrelated:** Task Node still shows one outstanding task, which is not one of ours. My first read listed 3 outstanding, before the 9 tasks moved into verification.

Every request got exactly one answer, and nothing was resubmitted. The answers and receipts are in `.codex-work/workers-20261007/tn-sec1/`, next to the earlier ones, along with the scripts used to send them. Task Node ran under the same account as round 1 (`~/.corbanu`, default profile), and no credential was printed. I removed my worktree. Its local branch is left in place because Git wouldn't delete it without a force delete.