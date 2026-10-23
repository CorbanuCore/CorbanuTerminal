Nothing has been submitted to Task Node yet and no rewards have been earned. No existing task fit this work, so I requested nine new ones. Task Node generated all nine, but they are still **Proposed** and can't take evidence until Travis accepts them.

**Inventory before I started (step 1):** 1 Proposed task (a PostfiatL1V2 node bug, unrelated), 2 Refused, 11 Rewarded, 0 Accepted, 0 waiting on verification. The only security task, `task_865f75c…` ("Corbanu workstream 1: PF-13 security…"), was rewarded on 2026-09-15, so I left it alone.

**New tasks for Travis to accept (one per track; 27.5 PFT offered in total):**

| Ref | Task | Sprints | PFT |
|---|---|---|---|
| P0SEC-TN-01 | `task_169f5d2052343f51777060e4ebb34e69` | PF-27-S04, PF-27-S02, PF-27-S05 | 2 |
| P0SEC-TN-02 | `task_1e60ecf27d3611270ba86aa46343226b` | PF-28-S01, PF-28-S02 | 2.5 |
| P0SEC-TN-03 | `task_94f31f7ebd6ba20445da0291ca77f945` | PF-33-S01, PF-33-S02 | 2.5 |
| P0SEC-TN-04 | `task_8ce36190b7954b6d5d4f1bcd691886cc` | PF-29-S01, PF-29-S02 | 3 |
| P0SEC-TN-05 | `task_b97153cdd989d5a800b05e1068c6dfe2` | PF-30-S01, PF-30-S02, PF-30-S03 | 3.5 |
| P0SEC-TN-06 | `task_22ba027b6f6a85dcd4cb47bf29888891` | PF-23-S01, PF-23-S02, PF-23-S03 | 3.5 |
| P0SEC-TN-07 | `task_627354898804e21841f81a559fe63462` | PF-24-S03, PF-24-S02, PF-25-S01 | 3.5 |
| P0SEC-TN-08 | `task_4c84685243b6fd68facf8c2667fbaa15` | PF-41-S01, PF-13-S07 (incl. #244, #255) | 4 |
| P0SEC-TN-09 | `task_8e353c0cb1ee43239c5a7691668087ab` | PF-25-S02 (#261 merged at 00:23Z) | 3 |

Each request was sent once with a tracking tag, and I checked the server for that tag before every send, so there are no duplicates. Every task description excludes the milestone VM run, human sign-off and flag removal.

**Mapping and records (step 5):** both docs-only PRs passed CI and are merged with `--match-head-commit`:
- **#262** (merge `319f7695f6`): adds the map at `qa/initiative-control/p0-security-levels/tasknode-task-map.md`. It links each task to its sprints, PRs with merge commits, gate records and videos. It also adds one evidence record per task under `docs/research/tasknode-integration/`, because every generated task asks for one. The records quote test counts and review verdicts word for word from the gate records and link all 109 demo videos, each of which returned HTTP 200.
- **#264** (merge `7e6ef740ef`): adds PF-25-S02, and changes the records to plain PR URLs, since the tasks require "clean" links.

**Once Travis accepts:** the evidence texts are written and final in `.codex-work/workers-20261007/tn-sec1/evidence-NN.md`. Running `submit.sh` there submits only for tasks that read Accepted and have no saved receipt. I ran it just now and it skipped all nine. Verification answers still have to be done after that.

**Disclosed in the evidence:**
- **PF-13-S07 count mismatch:** the evidence table counts 11 blocked routes per run, while the sprint record and #255 say 10; both show 0 leaked. Also, the `.ssh` read was blocked by the sandbox in 3 of 4 runs (in the Linux Moderate run the model refused to try it), and the Linux canary run used an older build.
- **Sprints merged but not archived:** PF-28-S02, PF-29-S01/S02, PF-33-S02 and PF-41-S01 are merged behind their flags, but their records stay open until the milestones.
- **Not in any task:** about a dozen smaller security fix PRs (for example #199–#232), which aren't among the 20 core sprints.

I used the Task Node setup from earlier rounds (`~/.corbanu`, default profile). Neither printed nor touched any credential, and the keychain was not involved.