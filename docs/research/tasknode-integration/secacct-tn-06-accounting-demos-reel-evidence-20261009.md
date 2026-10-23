# Task Node evidence record: SECACCT-TN-06

Compile the SECACCT-TN-06 Cost Accounting Reel Evidence Record. Task `task_2fe7008e72409e148dc44f7c176c3a10`, request `req_fefe3831866a488ed1930cdfdd8145748fa57c4118d17085b0610519266542d7`. Every PR below is merged to `main` in CorbanuCore/CorbanuTerminal; each merge commit was checked with `git merge-base --is-ancestor <sha> origin/main` at `3254a302fd` (2026-10-09). Repository quotes are copied verbatim from the files named above them (relative links re-pointed to this file's location); PR-description quotes are verbatim from `gh pr view <n> --json body` on 2026-10-09. Task map: [tasknode-task-map.md](../../../qa/initiative-control/p1-security-and-accounting/tasknode-task-map.md).

**Status: merged and published.** #332 re-recorded the PF-60-S03 demo videos at `7a11f9068bdf` and added four covering the acceptance fixes; #336 indexes a narrated 6:30 `/cost` reel on the GitHub `demos` release, also published on a Cloudflare watch page. **Open, not claimed:** the synthetic Breeze narration voice (`corbanu-narrator-calm-v1`) is not yet listening-approved by a person, and Breeze's research/non-commercial terms for a public release await Travis's confirmation.

Notes on the reel's text, which reflects the state at recording time: its index and narration say PF-60-S03's owner acceptance is not claimed and PF-60-S05 is planned; PF-60-S03 has since been accepted (SECACCT-TN-05) and S05 shipped (SECACCT-TN-07). The index's link to the PF-60-S03 sprint record pointed at `docs/sprints/current/`, which #335 moved to [docs/sprints/archive/](../../sprints/archive/portfolio-agent-cost-accounting/pf-60-s03-inspectable-run-and-campaign-totals.md); the docs PR that adds this record re-points that one link (the quote below shows the corrected link). The mp4's last chapter is titled "Summary"; the index row says "Summary and what is still ahead".

| PR | Merge commit on main | Merged (UTC) | Title |
| --- | --- | --- | --- |
| [#332](https://github.com/CorbanuCore/CorbanuTerminal/pull/332) | `ec1c3b5796` | 2026-10-09T06:24:55Z | docs(qa): PF-60-S03 demos at current main, plus four acceptance-fix demos |
| [#336](https://github.com/CorbanuCore/CorbanuTerminal/pull/336) | `9c36c54808` | 2026-10-09T08:01:46Z | docs(qa): agent cost accounting /cost demo reel |

Verification output (`gh pr view <n> --json state,baseRefName,mergeCommit,statusCheckRollup`, then `git merge-base --is-ancestor`; "checks" counts the PR's final check conclusions):

```text
#332 MERGED base=main merge=ec1c3b5796 checks: SKIPPED=13 SUCCESS=21; git merge-base --is-ancestor ec1c3b5796 3254a302fd -> exit 0
#336 MERGED base=main merge=9c36c54808 checks: SKIPPED=13 SUCCESS=21; git merge-base --is-ancestor 9c36c54808 3254a302fd -> exit 0
```

## Reel

- Video: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/agent-cost-accounting-reel-2026-10-09.mp4
- Subtitles: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/agent-cost-accounting-reel-2026-10-09.srt (299 lines)
- Cloudflare watch page: https://corbanu-accounting-reel.cloudflare-share.workers.dev/ (video: https://corbanu-accounting-reel.cloudflare-share.workers.dev/video.mp4), deployed 2026-10-09T07:50Z

`ffprobe -v error -show_entries format=duration,size:stream=codec_type,codec_name,width,height,r_frame_rate -of compact <mp4 URL>`:

```text
stream|codec_name=h264|codec_type=video|width=1920|height=1080|r_frame_rate=30/1
stream|codec_name=aac|codec_type=audio|r_frame_rate=0/0
stream|codec_name=bin_data|codec_type=data|r_frame_rate=0/0
format|duration=390.000000|size=32492599
```

390.0 s = 6:30, 1920x1080, 30 fps, H.264 + AAC. `ffprobe -show_entries chapter=start_time:chapter_tags=title -of csv=p=0 <mp4 URL>` (7 chapters, matching the 7 rows of the index's chapter table: 00:00, 00:33, 02:00, 03:13, 04:28, 05:11, 05:46):

```text
0.000000,Intro
33.567000,1. A conversation and its requests
120.633000,2. History: past days and ranges
193.567000,"3. Robustness: concurrent runs, deletes and clock skew"
268.167000,"4. Honest gaps: missing usage, no price"
311.867000,5. Developer-only today
346.167000,Summary
```

Cloudflare copy: `curl -sL <url> | shasum -a 256` gives `9a43a26508393041bd1aa14a5a9151dca79fc5620fea143fe418b59aa6cbf03c` for both the GitHub release asset and the Cloudflare `video.mp4` (32492599 bytes).

## HTTP checks

`curl -s -o /dev/null -I -L -w %{http_code}` on every `releases/download/demos/` URL in the reel index (9 of 9 returned 200) and the Cloudflare page and video:

| Status | Asset |
| --- | --- |
| 200 | `agent-cost-accounting-reel-2026-10-09.mp4` |
| 200 | `agent-cost-accounting-reel-2026-10-09.srt` |
| 200 | `pf-60-s03-pf60s03-concurrent-exec-then-cost-7a11f9068bdf-2026-10-08.mp4` |
| 200 | `pf-60-s03-pf60s03-cost-drilldown-and-scope-7a11f9068bdf-2026-10-08.mp4` |
| 200 | `pf-60-s03-pf60s03-cost-history-ranges-7a11f9068bdf-2026-10-08.mp4` |
| 200 | `pf-60-s03-pf60s03-default-build-no-cost-7a11f9068bdf-2026-10-08.mp4` |
| 200 | `pf-60-s03-pf60s03-deleted-conversations-7a11f9068bdf-2026-10-08.mp4` |
| 200 | `pf-60-s03-pf60s03-missing-usage-next-step-7a11f9068bdf-2026-10-08.mp4` |
| 200 | `pf-60-s03-pf60s03-range-in-progress-7a11f9068bdf-2026-10-08.mp4` |
| 200 | charset=utf-8 (`text/html;`) |
| 200 | https://corbanu-accounting-reel.cloudflare-share.workers.dev/video.mp4 (`video/mp4`) |

PF-60-S03 demo index (20 assets, all 200; rows marked superseded are the earlier takes):

| Demo | Commit | Length | Asset | HTTP |
| --- | --- | --- | --- | --- |
| `pf60s03-cost-drilldown-and-scope` | `7a11f9068bdf` | 43s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-cost-drilldown-and-scope-7a11f9068bdf-2026-10-08.mp4 | 200 |
| `pf60s03-cost-drilldown-and-scope` | `7a11f9068bdf` | 43s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-cost-drilldown-and-scope-7a11f9068bdf-2026-10-08.cast | 200 |
| `pf60s03-cost-history-ranges` | `7a11f9068bdf` | 32s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-cost-history-ranges-7a11f9068bdf-2026-10-08.mp4 | 200 |
| `pf60s03-cost-history-ranges` | `7a11f9068bdf` | 32s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-cost-history-ranges-7a11f9068bdf-2026-10-08.cast | 200 |
| `pf60s03-concurrent-exec-then-cost` | `7a11f9068bdf` | 27s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-concurrent-exec-then-cost-7a11f9068bdf-2026-10-08.mp4 | 200 |
| `pf60s03-concurrent-exec-then-cost` | `7a11f9068bdf` | 27s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-concurrent-exec-then-cost-7a11f9068bdf-2026-10-08.cast | 200 |
| `pf60s03-missing-usage-next-step` | `7a11f9068bdf` | 20s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-missing-usage-next-step-7a11f9068bdf-2026-10-08.mp4 | 200 |
| `pf60s03-missing-usage-next-step` | `7a11f9068bdf` | 20s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-missing-usage-next-step-7a11f9068bdf-2026-10-08.cast | 200 |
| `pf60s03-deleted-conversations` | `7a11f9068bdf` | 30s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-deleted-conversations-7a11f9068bdf-2026-10-08.mp4 | 200 |
| `pf60s03-deleted-conversations` | `7a11f9068bdf` | 30s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-deleted-conversations-7a11f9068bdf-2026-10-08.cast | 200 |
| `pf60s03-range-in-progress` | `7a11f9068bdf` | 34s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-range-in-progress-7a11f9068bdf-2026-10-08.mp4 | 200 |
| `pf60s03-range-in-progress` | `7a11f9068bdf` | 34s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-range-in-progress-7a11f9068bdf-2026-10-08.cast | 200 |
| `pf60s03-default-build-no-cost` | `7a11f9068bdf` | 23s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-default-build-no-cost-7a11f9068bdf-2026-10-08.mp4 | 200 |
| `pf60s03-default-build-no-cost` | `7a11f9068bdf` | 23s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-default-build-no-cost-7a11f9068bdf-2026-10-08.cast | 200 |
| `pf60s03-cost-drilldown-and-scope` (superseded) | `5ba0c02f6705` | 43s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-cost-drilldown-and-scope-5ba0c02f6705-2026-10-08.mp4 | 200 |
| `pf60s03-cost-drilldown-and-scope` (superseded) | `5ba0c02f6705` | 43s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-cost-drilldown-and-scope-5ba0c02f6705-2026-10-08.cast | 200 |
| `pf60s03-cost-history-ranges` (superseded) | `5ba0c02f6705` | 32s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-cost-history-ranges-5ba0c02f6705-2026-10-08.mp4 | 200 |
| `pf60s03-cost-history-ranges` (superseded) | `5ba0c02f6705` | 32s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-cost-history-ranges-5ba0c02f6705-2026-10-08.cast | 200 |
| `pf60s03-concurrent-exec-then-cost` (superseded) | `5ba0c02f6705` | 25s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-concurrent-exec-then-cost-5ba0c02f6705-2026-10-08.mp4 | 200 |
| `pf60s03-concurrent-exec-then-cost` (superseded) | `5ba0c02f6705` | 25s | https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-concurrent-exec-then-cost-5ba0c02f6705-2026-10-08.cast | 200 |

## Index excerpts

Verbatim, `qa/demos/index/accounting-reel.md` lines 1-22:

> # Agent cost accounting: /cost demo reel
>
> One narrated video of the agent cost accounting work (`/cost`), edited from the current-main PF-60-S03 demo videos in
> [PF-60-S03.md](../../../qa/demos/index/PF-60-S03.md) (the rows recorded at `7a11f9068bdf`, not the superseded ones). It is an asset on the
> [`demos` prerelease](https://github.com/CorbanuCore/CorbanuTerminal/releases/tag/demos).
>
> - Video: [agent-cost-accounting-reel-2026-10-09.mp4](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/agent-cost-accounting-reel-2026-10-09.mp4) (6:30, 1920x1080, 30 fps, H.264/AAC, chapters
>   embedded) · subtitles: [agent-cost-accounting-reel-2026-10-09.srt](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/agent-cost-accounting-reel-2026-10-09.srt)
> - `/cost` is developer-only: it exists only in debug builds with the `developer-accounting` feature. Chapter 5 shows
>   the default build without it.
> - Every terminal frame is a published GLM 5.2 recording. Clips are trimmed to the moment that proves the acceptance
>   criterion; waiting is sped up (an on-screen badge shows the rate) and proof frames are held. Added on top: chapter
>   cards, one evidence card (the independent acceptance reconciliation), a side panel naming the sprint and criterion,
>   highlights on the proving lines, and burned-in subtitles. Seeding is stated on each source clip's title card (cut
>   from the reel) and in the narration where a seeded screen is highlighted.
> - Criteria: the [PF-60-S03 sprint record](../../sprints/archive/portfolio-agent-cost-accounting/pf-60-s03-inspectable-run-and-campaign-totals.md)
>   and the [independent acceptance](../../../qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/README.md)
>   with its three re-runs. Clock-behind turns (criterion 15) are cited from re-run 3, not shown in a clip.
> - Not claimed: PF-60-S03's owner acceptance, and [PF-60-S05](../../sprints/current/portfolio-agent-cost-accounting/pf-60-s05-collection-correctness-and-billing-basis.md)
>   (declared billing basis for subscription vs pay-per-use work), which the narration calls planned, not done.
> - Narration: Breeze TTS, synthetic voice `corbanu-narrator-calm-v1` (the P0 reel's voice, portable voice library).
>   Not yet listening-approved by a person.

Verbatim, `qa/demos/index/accounting-reel.md` lines 24-34:

> ## Chapters
>
> | Time | Chapter | Source clips |
> | --- | --- | --- |
> | 00:00 | Intro | |
> | 00:33 | 1. A conversation and its requests | [`pf60s03-cost-drilldown-and-scope`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-cost-drilldown-and-scope-7a11f9068bdf-2026-10-08.mp4) · evidence card: [independent acceptance](../../../qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/README.md) reconciliation |
> | 02:00 | 2. History: past days and ranges | [`pf60s03-cost-history-ranges`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-cost-history-ranges-7a11f9068bdf-2026-10-08.mp4) · [`pf60s03-range-in-progress`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-range-in-progress-7a11f9068bdf-2026-10-08.mp4) |
> | 03:13 | 3. Robustness: concurrent runs, deletes and clock skew | [`pf60s03-concurrent-exec-then-cost`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-concurrent-exec-then-cost-7a11f9068bdf-2026-10-08.mp4) · [`pf60s03-deleted-conversations`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-deleted-conversations-7a11f9068bdf-2026-10-08.mp4) |
> | 04:28 | 4. Honest gaps: missing usage, no price | [`pf60s03-missing-usage-next-step`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-missing-usage-next-step-7a11f9068bdf-2026-10-08.mp4) · [`pf60s03-cost-drilldown-and-scope`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-cost-drilldown-and-scope-7a11f9068bdf-2026-10-08.mp4) |
> | 05:11 | 5. Developer-only today | [`pf60s03-cost-history-ranges`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-cost-history-ranges-7a11f9068bdf-2026-10-08.mp4) · [`pf60s03-default-build-no-cost`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s03-pf60s03-default-build-no-cost-7a11f9068bdf-2026-10-08.mp4) |
> | 05:46 | Summary and what is still ahead | |

Verbatim, `qa/demos/index/PF-60-S03.md` lines 1-4:

> # PF-60-S03 demo videos
>
> Recorded with `scripts/demo_video.py`; see [the demo SOP](../../../qa/demos/README.md).
> Videos are assets on the [`demos` prerelease](https://github.com/CorbanuCore/CorbanuTerminal/releases/tag/demos).

## Open, not done

- Narration voice not yet listening-approved by a person.
- Breeze research/non-commercial terms on a public release: Travis to confirm.
- The reel shows a delete with the clock behind the ledger; that a turn with the clock behind completes is narrated, citing re-run 3, not shown. Some history in the clips is seeded and says so.
