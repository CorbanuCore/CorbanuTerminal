# The Aggressive security level: demo reel

One narrated video of the Aggressive security level as it stands after the fixes of 2026-10-13. It extends the
[P0 security reel](P0-security-reel.md) (the building blocks, 2026-10-08) with what a person who chooses Aggressive
gets, edited from the issue demo videos indexed in this folder (`issue-6*.md`, `ISSUE-*.md`, `AGG-*.md`). It is an
asset on the [`demos` prerelease](https://github.com/CorbanuCore/CorbanuTerminal/releases/tag/demos).

- Watch online: <https://corbanu-aggressive-reel.cloudflare-share.workers.dev/> (published 2026-10-14; subtitles are burned in, and the page also
  offers them as an English text track). Direct MP4: <https://corbanu-aggressive-reel.cloudflare-share.workers.dev/video.mp4>, the release asset losslessly remuxed for fast start, chapters kept.
- Video: [aggressive-reel-2026-10-13.mp4](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/aggressive-reel-2026-10-13.mp4) (9:58, 1920x1080, 30 fps, H.264/AAC, chapters
  embedded) · subtitles: [aggressive-reel-2026-10-13.srt](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/aggressive-reel-2026-10-13.srt)
- Security levels are behind the default-off `security_levels` flag.
- The third code-blind milestone run ([record](../../security-levels/milestones/aggressive-20261013-third/README.md), PR #741) is **not passed**: 51 passed, 6 failed, 7 blocked of 64 (defects #736, #737, #739; advisory #740). Aggressive is **not signed off**: the record requests Travis's named sign-off, and none is recorded. Chapter 11 and the summary say so.
- Every terminal frame is a published demo recording (GLM 5.3 Flash on Z.AI, recorded 2026-10-10 to 2026-10-13; each
  clip's commit is in its panel). Clips are trimmed to the moment that proves the criterion; waiting is sped up (an
  on-screen badge shows the rate) and proof frames are held. Added on top: chapter cards, an evidence card for the
  milestone runs, a side panel naming the sprint or issue and the criterion, highlights on the proving lines, and
  burned-in subtitles.
- Criteria come from the owner decisions recorded on the issues (#442, #620, #623, #626, #507, #660) and the P0
  sprint records; the decisions are listed in the milestone runs' amendment tables.
- Narration: Breeze TTS, synthetic voice `corbanu-narrator-calm-v1` (the earlier reels' voice, portable voice
  library).

## Chapters

| Time | Chapter | Sprints / issues | Source clips |
| --- | --- | --- | --- |
| 00:00 | Intro | | |
| 00:37 | 1. Choosing Aggressive | PF-24-S02, #440, #442 | [`issue621-python-listing-never-granted`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-621-issue621-python-listing-never-granted-30cb18094768-2026-10-12.mp4) |
| 01:39 | 2. Protected reads: never granted, always explained | #442, #621 | [`issue621-python-listing-never-granted`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-621-issue621-python-listing-never-granted-30cb18094768-2026-10-12.mp4) |
| 02:14 | 3. Grant once: only where it applies | PF-25-S01, #620, #442 | [`issue620-zshrc-no-grant`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-620-issue620-zshrc-no-grant-30cb18094768-2026-10-12.mp4) · [`issue442-grant-once-what-it-lifts`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-442-issue442-grant-once-what-it-lifts-c6af277703ed-2026-10-12.mp4) |
| 03:32 | 4. Remembered approvals are suspended | #626 | [`issue626-remembered-approval-suspended`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-626-issue626-remembered-approval-suspended-79e5ab0c0b26-2026-10-13.mp4) |
| 04:04 | 5. .codex and .agents stay read-only | #584, #623 | [`issue584-agent-settings-no-grant`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-584-issue584-agent-settings-no-grant-c2aa460ac3e2-2026-10-13.mp4) · [`issue623-agents-link-refused`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-623-issue623-agents-link-refused-79e5ab0c0b26-2026-10-13.mp4) |
| 04:44 | 6. .env files | #442 | [`issue442-env-templates-usable`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-442-issue442-env-templates-usable-c6af277703ed-2026-10-12.mp4) · [`issue442-new-env-file-denied`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-442-issue442-new-env-file-denied-04fa31adcd7f-2026-10-11.mp4) |
| 05:31 | 7. The kill switch | PF-25-S02, #507, #660 | [`issue660-codex-pane-task-waits`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-660-issue660-codex-pane-task-waits-a1a41f5f3c26-2026-10-13.mp4) · [`issue507-kill-switch-other-process`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-507-issue507-kill-switch-other-process-6876c2557f10-2026-10-12.mp4) |
| 06:24 | 8. /panes keeps an approval safe | #624 | [`issue624-panes-picker-keeps-approval`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-624-issue624-panes-picker-keeps-approval-346c234ce89c-2026-10-12.mp4) |
| 06:53 | 9. Recent denials in the inspector | PF-41-S01, #622 | [`issue622-inspector-lists-denials`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-622-issue622-inspector-lists-denials-30cb18094768-2026-10-12.mp4) |
| 07:25 | 10. The credential broker | PF-27-S05, #391 | [`sec391-aggressive-broker-default`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-391-sec391-aggressive-broker-default-295c20b8004f-2026-10-10.mp4) · [`sec391-aggressive-broker-fails-closed`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-391-sec391-aggressive-broker-fails-closed-295c20b8004f-2026-10-10.mp4) |
| 08:07 | 11. The third milestone run | Aggressive ships, #741 | evidence card: milestone records [run 1](../../security-levels/milestones/aggressive-20261010/README.md) · [run 2](../../security-levels/milestones/aggressive-20261011-rerun/README.md) · [run 3](../../security-levels/milestones/aggressive-20261013-third/README.md) |
| 09:22 | Summary and what is still ahead | | |

## Related clips not used

- [`issue442-env-link-refused`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-442-issue442-env-link-refused-c6af277703ed-2026-10-12.mp4): #442: a .env that is a link is refused at the switch, with how to fix it (narrated in chapter 6, not shown).
- [`issue613-kill-switch-texts`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-613-issue613-kill-switch-texts-cc351d2aebf0-2026-10-13.mp4): #613: the kill-switch texts say which tasks wait; /orchestrate resume is refused while it is on.
- [`issue442-protected-read-explained`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-442-issue442-protected-read-explained-880e5030fc82-2026-10-11.mp4): #442: a denied `cat .env` is explained, never granted.
- [`agg-437-inspector-decline`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/agg-437-agg-437-inspector-decline-cdb62e96c6d7-2026-10-11.mp4): #437: the inspector lists a declined approval.
