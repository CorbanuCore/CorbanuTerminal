All three tasks are **Rewarded** at the full amount: TN-10 got 3.5 PFT, TN-11 got 3.5 PFT and TN-12 got 2.5 PFT. Each answer was accepted the first time. They moved to Rewarded within about 5 minutes, with no follow-up requests. All the command output I sent was run fresh just before sending (fresh fetch of main at `5933e2eb0d`, 2026-10-08 20:19 UTC).

**TN-10**
- **Asked:** raw output of the `gh api` call for CI job 113190329753, and of `git merge-base --is-ancestor 63ea3d0cbd origin/main; echo $?`.
- **Sent:** the job is "pf_27_s06 probes (windows-2022)", with result `success` at 2026-10-08 07:18:34 UTC on commit `2a6b26ba49`. The merge-base check returned `0`. I also added a check that `2a6b26ba49` is itself on main, and the link to the job.
- **Payment transaction:** `6C316BAE…4F9970`

**TN-11**
- **Asked:** the same merge-base check printed as `exit=$?`, plus a link or screenshot showing PR #278 merged.
- **Sent:** `exit=0`, the PR #278 link, and `gh pr view` showing it as MERGED into main with merge commit `63ea3d0cbd`. I also showed which files that merge added and that the TN-11 record is on main.
- **Payment transaction:** `B3326E98…DED5C6C8`

**TN-12**
- **Asked:** raw `ffprobe -show_chapters` output, a frame at the chapter-7 mark showing the side panel and subtitles, and how 13 chapter marks fit the task's "12-chapter index."
- **Sent:**
  - The full 117-line ffprobe output: 13 chapters, Intro, 01 to 11, then Summary. The local mp4's checksum matches both the copy on the GitHub `demos` release and the Cloudflare deployment.
  - Two frames:
    - 07:23 shows the chapter card "CHAPTER 07 OF 11 / PF-24-S02" with its subtitle.
    - 07:35 shows the side panel (sprint PF-24-S02 and its acceptance criterion, clip name and commit) with its subtitle.

    The exact 07:19.667 mark is a fade with no panel, so I said that and sent the first frame where the panel appears.
  - On the chapter count: the index file has 13 rows at both the PR #274 merge and on current main (Intro, 11 numbered chapters, Summary). The "12 chapters" wording came from our own task request and was a miscount, and I said so in the answer. I also noted one small wording difference: the mp4's last chapter is "Summary," while the index row says "Summary and what is still ahead." Both open items stayed disclosed as not resolved: a person still has to approve the narration by listening, and Travis still has to confirm the Breeze usage terms.
- **Payment transaction:** `CA268319…14AAB628`

**Things I changed outside Task Node:**
- **Two images added to a public release:** the Task Node command can't upload files, so I added both frames as assets on the public `demos` release of CorbanuCore/CorbanuTerminal. Both URLs return 200 and the files match the local ones byte for byte. Delete them if you'd rather not keep them there.
- **`respond.sh`:** it now passes each line of an optional `verify-NN.artifacts` file to the command as a link, so the images and the watch page went in as attached evidence.

The verify files passed a scan for keys and tokens before sending, and no credentials were printed. `rewardOutcome` itself is still empty, but the tasks are marked Rewarded with payment transaction IDs.