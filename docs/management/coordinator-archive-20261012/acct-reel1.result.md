The accounting reel is finished and uploaded to the `demos` release, and the docs PR is merged. It runs 6:30, inside the 4–7 minute target. It is not on Cloudflare.

- **Video:** https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/agent-cost-accounting-reel-2026-10-09.mp4 (1920x1080, 30 fps, chapters embedded in the file)
- **Subtitles:** the same link ending in `.srt`
- **Index doc:** `qa/demos/index/accounting-reel.md` lists the chapters and links every source clip. PR #336 merged as `9c36c54808`.

**Chapters**
| Time | Chapter | What it shows |
| --- | --- | --- |
| 00:00 | Intro | Says up front that `/cost` is a developer-only feature today |
| 00:33 | 1. A conversation and its requests | Drill-down from a conversation to a request to its technical details; Esc goes back the same way. An empty conversation still names the day's other requests. A card shows the independent check: every total matched a recalculation to the micro-dollar |
| 02:00 | 2. History: past days and ranges | A past day; ranges by day and by week; a month that includes today shows "In progress" and "Cost so far"; the range and day pages give the same retention date |
| 03:13 | 3. Robustness | Three runs at the same time, each counted once with no "accounting failed"; a conversation deleted with the clock 2 hours behind the ledger is still counted on the day page |
| 04:28 | 4. Honest gaps | Missing usage: the next step is to check the bill from Z.AI, the conversation's own provider. No price: the total reads "at least" and the next step names that provider's bill |
| 05:11 | 5. Developer-only today | The default build has no per-request cost history, and `/cost` is an unrecognized command there |
| 05:46 | Summary | Says PF-60-S03 is still waiting for acceptance, and that PF-60-S05 (declaring per provider whether it bills by subscription or by use) is planned, not done |

I waited for PR #332 to merge before finalizing. All seven clips are the current-main PF-60-S03 recordings; no other accounting sprint has demo videos. The script draft is `script/SCRIPT.md` in the work folder.

**Checks I ran**
- **Highlights:** all 26 boxes land on the right lines at their first and last frame. I checked this against the final MP4, and fixed one box that sat on a frame where the page had already closed.
- **Narration:** a speech-to-text pass found every scripted word, with no omissions or repeats. Subtitles are timed to the spoken words of each narration clip. Timing them from a transcript of the whole track had put them about 1.5 s early.
- **Sync:** the audio is 0 ms off at every point I measured.
- **Secrets and paths:** no key-shaped strings and no `/Users/` paths in the clips, subtitles or generated graphics. The `/Volumes/...` worktree paths are visible, as in the P0 reel.
- **Build check:** the HyperFrames check has no errors; 79/79 text checks pass the contrast check. The newer version (I updated it from 0.8.140 to 0.8.143) adds 56 warnings about how the project file is organised; they don't affect the video.
- **PR CI:** one unrelated Ubuntu tmux test failed (`tmux_current_exact_replacement_is_persisted_before_deactivation`) on this docs-only PR. It passed on one rerun.

**Needs your review**
1. **The narrator's voice.** It is the P0 reel's voice (`corbanu-narrator-calm-v1`), and still no person has approved it by listening. I tried spelling "Corbanu" phonetically, but speech-to-text heard "Korribanu", so I went back to the plain spelling, which it hears as "Corbanu".
2. **Clock behind the ledger.** The video shows a delete with the clock behind the ledger, not a turn. That a turn with the clock behind completes is only narrated, citing the independent re-run on Linux.
3. **Seeded data.** Some history and two of the other conversations in the clips are seeded test data; the narration says so where it is highlighted.
4. **Usage terms.** Breeze's terms allow research and non-commercial use only; same open question as for the P0 reel.

The work folder `.codex-work/accounting-reel-20261009/` holds the build pipeline and the final files. My worktree is removed and no cargo target folder was created. The local branch `docs/accounting-reel-20261009` is still there, because a normal delete was refused and I didn't force it.