# Agent cost accounting: /cost demo reel

One narrated video of the agent cost accounting work (`/cost`), edited from the current-main PF-60-S03 demo videos in
[PF-60-S03.md](PF-60-S03.md) (the rows recorded at `7a11f9068bdf`, not the superseded ones). It is an asset on the
[`demos` prerelease](https://github.com/CorbanuCore/CorbanuTerminal/releases/tag/demos).

- Video: [agent-cost-accounting-reel-2026-10-09.mp4](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/agent-cost-accounting-reel-2026-10-09.mp4) (6:30, 1920x1080, 30 fps, H.264/AAC, chapters
  embedded) · subtitles: [agent-cost-accounting-reel-2026-10-09.srt](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/agent-cost-accounting-reel-2026-10-09.srt)
- `/cost` is developer-only: it exists only in debug builds with the `developer-accounting` feature. Chapter 5 shows
  the default build without it.
- Every terminal frame is a published GLM 5.2 recording. Clips are trimmed to the moment that proves the acceptance
  criterion; waiting is sped up (an on-screen badge shows the rate) and proof frames are held. Added on top: chapter
  cards, one evidence card (the independent acceptance reconciliation), a side panel naming the sprint and criterion,
  highlights on the proving lines, and burned-in subtitles. Seeding is stated on each source clip's title card (cut
  from the reel) and in the narration where a seeded screen is highlighted.
- Criteria: the [PF-60-S03 sprint record](../../../docs/sprints/current/portfolio-agent-cost-accounting/pf-60-s03-inspectable-run-and-campaign-totals.md)
  and the [independent acceptance](../../portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/README.md)
  with its three re-runs. Clock-behind turns (criterion 15) are cited from re-run 3, not shown in a clip.
- Not claimed: PF-60-S03's owner acceptance, and [PF-60-S05](../../../docs/sprints/current/portfolio-agent-cost-accounting/pf-60-s05-collection-correctness-and-billing-basis.md)
  (declared billing basis for subscription vs pay-per-use work), which the narration calls planned, not done.
- Narration: Breeze TTS, synthetic voice `corbanu-narrator-calm-v1` (the P0 reel's voice, portable voice library).
  Not yet listening-approved by a person.

## Chapters

| Time | Chapter | Source clips |
| --- | --- | --- |
| 00:00 | Intro | |
| 00:33 | 1. A conversation and its requests | [`pf60s03-cost-drilldown-and-scope`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-cost-drilldown-and-scope-7a11f9068bdf-2026-10-08.mp4) · evidence card: [independent acceptance](../../portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/README.md) reconciliation |
| 02:00 | 2. History: past days and ranges | [`pf60s03-cost-history-ranges`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-cost-history-ranges-7a11f9068bdf-2026-10-08.mp4) · [`pf60s03-range-in-progress`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-range-in-progress-7a11f9068bdf-2026-10-08.mp4) |
| 03:13 | 3. Robustness: concurrent runs, deletes and clock skew | [`pf60s03-concurrent-exec-then-cost`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-concurrent-exec-then-cost-7a11f9068bdf-2026-10-08.mp4) · [`pf60s03-deleted-conversations`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-deleted-conversations-7a11f9068bdf-2026-10-08.mp4) |
| 04:28 | 4. Honest gaps: missing usage, no price | [`pf60s03-missing-usage-next-step`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-missing-usage-next-step-7a11f9068bdf-2026-10-08.mp4) · [`pf60s03-cost-drilldown-and-scope`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-cost-drilldown-and-scope-7a11f9068bdf-2026-10-08.mp4) |
| 05:11 | 5. Developer-only today | [`pf60s03-cost-history-ranges`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-cost-history-ranges-7a11f9068bdf-2026-10-08.mp4) · [`pf60s03-default-build-no-cost`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-default-build-no-cost-7a11f9068bdf-2026-10-08.mp4) |
| 05:46 | Summary and what is still ahead | |
