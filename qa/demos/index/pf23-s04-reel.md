# Chroot classification (PF-23-S04): functional-run reel

One narrated video of what the independent functional run of 2026-11-14 showed for PF-23-S04 (#1837 by
@citadelculture). It is an asset on the [`demos` prerelease](https://github.com/CorbanuCore/CorbanuTerminal/releases/tag/demos).

- Video: [pf23-s04-reel-2026-11-14.mp4](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf23-s04-reel-2026-11-14.mp4)
  (3:56, 1920x1080, 30 fps, H.264/AAC, chapters embedded) · subtitles:
  [pf23-s04-reel-2026-11-14.srt](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf23-s04-reel-2026-11-14.srt)
- Security levels are behind the default-off `security_levels` flag; the clips run under Moderate after untrusted
  content.
- Every terminal frame is a text capture (`agms-tui screen`) from the code-blind run itself, not a re-recording: the
  candidate main `1dea8b3b03` (SHA-256 `546a2440…344b`, the installed build) in disposable Tart macOS guests, synthetic
  data only, GLM 5.3 Flash as executor; chapter 1 is the local-model replay G4R (MODEL-DECLINE rule). Captures keep
  the screen's text but not its colours; each frame is held while it is narrated. `/Users/agmxN/…` paths are the
  guests' throwaway test accounts. Record:
  [functional-20261114.md](../../security-levels/sprints/PF-23-S04/functional-20261114.md).
- Added on top: chapter cards, a results card, a side panel naming the case and its criterion, highlights on the
  proving lines, and burned-in subtitles. Narration: Breeze TTS, voice `corbanu-narrator-calm-v1` (the approved
  narrator voice of the earlier reels).

## Chapters

| Time | Chapter | Cases | Source clip |
| --- | --- | --- | --- |
| 00:00 | Intro | | |
| 00:35 | 1. A known command behind chroot / | 1 (replay G4R) | [`pf23-s04-known-forms`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf23-func-pf23-s04-known-forms-1dea8b3b0304-2026-11-14.mp4) |
| 01:08 | 2. Code the host cannot read | 4, 7 (G4A) | [`pf23-s04-unseen-roots`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf23-func-pf23-s04-unseen-roots-1dea8b3b0304-2026-11-14.mp4) |
| 01:34 | 3. No extra prompt for harmless work | 6 (G4C) | [`pf23-s04-harmless-root`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf23-func-pf23-s04-harmless-root-1dea8b3b0304-2026-11-14.mp4) |
| 01:57 | 4. Refuse, cancel and resume | 2, 3 (G4A), 9 (G4C) | [`pf23-s04-refuse-cancel-resume`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf23-func-pf23-s04-refuse-cancel-resume-1dea8b3b0304-2026-11-14.mp4) · [`pf23-s04-resume`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf23-func-pf23-s04-resume-1dea8b3b0304-2026-11-14.mp4) |
| 02:26 | 5. What the run found | all; #2165, #2167 | results card from the record |
| 03:23 | Summary | | |

Not shown, because the run could not observe them: effects of an approved chroot command under Moderate or
Aggressive (their sandbox stops `sudo`), anything inside a root other than `/` on macOS, and Linux or Windows.
