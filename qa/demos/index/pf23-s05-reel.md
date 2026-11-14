# Assignments after untrusted output (PF-23-S05): functional-run reel

One narrated video of what the independent functional run of 2026-11-14 showed for PF-23-S05 (#2133 by @araupooo).
It is an asset on the [`demos` prerelease](https://github.com/CorbanuCore/CorbanuTerminal/releases/tag/demos).

- Video: [pf23-s05-reel-2026-11-14.mp4](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf23-s05-reel-2026-11-14.mp4)
  (3:31, 1920x1080, 30 fps, H.264/AAC, chapters embedded) · subtitles:
  [pf23-s05-reel-2026-11-14.srt](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf23-s05-reel-2026-11-14.srt)
- Security levels are behind the default-off `security_levels` flag; the clips run under Moderate after untrusted
  content.
- Every terminal frame is a text capture (`agms-tui screen`) from the code-blind run itself, not a re-recording: the
  candidate main `1dea8b3b03` (SHA-256 `546a2440…344b`, the installed build) in disposable Tart macOS guests, synthetic
  data only, GLM 5.3 Flash as executor; chapter 2 is the local-model replay G5R. Captures keep the screen's text but
  not its colours; each frame is held while it is narrated. `/Users/agmxN/…` paths are the guests' throwaway test
  accounts. Record: [functional-20261114.md](../../security-levels/sprints/PF-23-S05/functional-20261114.md).
- Added on top: chapter cards, a results card, a side panel naming the case and its criterion, highlights on the
  proving lines, and burned-in subtitles. Narration: Breeze TTS, voice `corbanu-narrator-calm-v1` (the approved
  narrator voice of the earlier reels).

## Chapters

| Time | Chapter | Cases | Source clip |
| --- | --- | --- | --- |
| 00:00 | Intro | | |
| 00:37 | 1. Values that only arrive through an assignment | 12, 13, 14 (G5E) | [`pf23-s05-assignment-paths`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf23-func-pf23-s05-assignment-paths-1dea8b3b0304-2026-11-14.mp4) |
| 01:10 | 2. Temporary and overwritten values | 15, 16 (replay G5R) | [`pf23-s05-replay-forms`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf23-func-pf23-s05-replay-forms-1dea8b3b0304-2026-11-14.mp4) |
| 01:34 | 3. Only a real decision releases an action | 5 (G5A) | [`pf23-s05-claims`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf23-func-pf23-s05-claims-1dea8b3b0304-2026-11-14.mp4) |
| 01:50 | 4. Ordinary assignments still work | 17 (G5F) | [`pf23-s05-ordinary`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf23-func-pf23-s05-ordinary-1dea8b3b0304-2026-11-14.mp4) |
| 02:12 | 5. What the run found | all; #2166, #2167 | results card from the record |
| 02:58 | Summary | | |

Only prompts whose reason is "credential access" for a value that reached the command through an assignment show the
assignment logic; prompts drawn by the project script (`scripts/show-file.sh`) show only that nothing ran before a
decision (see the record's evidence-quality note). Case 11 has no discriminating frame and is not shown.
