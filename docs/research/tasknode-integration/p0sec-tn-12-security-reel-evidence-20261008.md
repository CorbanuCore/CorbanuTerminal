# Task Node evidence record: P0SEC-TN-12

Compile the P0SEC-TN-12 Security Reel Evidence Record. Task `task_2b6a9c4ef14b857f571f0aedd66f6a95`, request `req_ad39a7c3d2153a5ada063b6ea19fef0e8d2b7d1031c97c12ca59c22dbe6e39cd`. Every PR below is merged to `main` in CorbanuCore/CorbanuTerminal; each merge commit was checked with `git merge-base --is-ancestor <sha> origin/main` at `ad96c55cb5` (2026-10-08). Quoted blocks are copied verbatim from the records named above them (relative links re-pointed to this file's location).

Index: [qa/demos/index/P0-security-reel.md](../../../qa/demos/index/P0-security-reel.md). **Open, not claimed:** the synthetic Breeze narration (voice `corbanu-narrator-calm-v1`) is not yet listening-approved by a person (a speech-to-text pass heard "Corbanu" as "Corbano"), and Breeze's research/non-commercial terms for a reel on a public GitHub release await Travis's confirmation. Milestone gates are out of scope.

| PR | Merge commit on main | What |
| --- | --- | --- |
| #274 | `40f4acafe8` | P0 security-levels demo reel index |

PR links:

- #274: https://github.com/CorbanuCore/CorbanuTerminal/pull/274

## Reel

- Video: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/p0-security-levels-reel-2026-10-08.mp4
- Subtitles: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/p0-security-levels-reel-2026-10-08.srt (559 lines)
- Release: https://github.com/CorbanuCore/CorbanuTerminal/releases/tag/demos

`ffprobe -v error -show_entries format=duration,size:stream=codec_type,codec_name,width,height,r_frame_rate -of compact <mp4 URL>`:

```text
stream|codec_name=h264|codec_type=video|width=1920|height=1080|r_frame_rate=30/1
stream|codec_name=aac|codec_type=audio|r_frame_rate=0/0
stream|codec_name=bin_data|codec_type=data|r_frame_rate=0/0
format|duration=711.333333|size=75132902
```

Duration 711.33 s = 11:51. Embedded chapters (`ffprobe -show_chapters`): 13 (Intro, 01-11, Summary); starts 0, 30.467, 98.567, 154.633, 231.767, 315.033, 382.933, 439.667, 508.667, 544.167, 585.767, 638.833, 679.033 s, matching the index times.

## HTTP checks

`curl -s -o /dev/null -I -L -w %{http_code}` on every `releases/download/demos/` URL in the index: 32 of 32 returned 200 (reel mp4, srt, 26 chapter clips, 4 cut clips).

| Status | Asset |
| --- | --- |
| 200 | `p0-security-levels-reel-2026-10-08.mp4` |
| 200 | `p0-security-levels-reel-2026-10-08.srt` |
| 200 | `pf-13-s07-pf13s07-composition-hello-64137b71894f-2026-10-06.mp4` |
| 200 | `pf-23-s02-pf23s02-hook-write-blocked-93e149e066f0-2026-10-07.mp4` |
| 200 | `pf-23-s03-pf23s03-project-cannot-lower-a230f2082141-2026-10-07.mp4` |
| 200 | `pf-23-s03-pf23s03-stored-level-next-session-a230f2082141-2026-10-07.mp4` |
| 200 | `pf-24-s02-pf24s02-glm-cancel-e4d17dbdc6f0-2026-10-07.mp4` |
| 200 | `pf-24-s02-pf24s02-glm-confirm-e4d17dbdc6f0-2026-10-07.mp4` |
| 200 | `pf-24-s02-pf24s02-glm-downgrade-e4d17dbdc6f0-2026-10-07.mp4` |
| 200 | `pf-24-s03-pf24-child-inherits-7f6c88912c14-2026-10-06.mp4` |
| 200 | `pf-25-s01-pf25s01-grant-once-aee150e46ae7-2026-10-07.mp4` |
| 200 | `pf-25-s01-pf25s01-grant-until-expiry-aee150e46ae7-2026-10-07.mp4` |
| 200 | `pf-25-s02-pf25s02-kill-switch-off-be5d33660c7e-2026-10-07.mp4` |
| 200 | `pf-25-s02-pf25s02-kill-switch-restart-be5d33660c7e-2026-10-07.mp4` |
| 200 | `pf-25-s02-pf25s02-revoke-grant-be5d33660c7e-2026-10-07.mp4` |
| 200 | `pf-27-s02-pf27s02-protected-paths-03c9dfd9716d-2026-10-06.mp4` |
| 200 | `pf-27-s02-pf27s02-secretless-env-03c9dfd9716d-2026-10-06.mp4` |
| 200 | `pf-27-s04-pf27-broker-crash-fails-closed-1a4370992f5a-2026-10-05.mp4` |
| 200 | `pf-28-s01-pf28s01-persistence-clean-9261bd416f0d-2026-10-06.mp4` |
| 200 | `pf-28-s01-pf28s01-tool-output-gated-9261bd416f0d-2026-10-06.mp4` |
| 200 | `pf-28-s02-pf28s02-reflected-credential-10357ba50c80-2026-10-06.mp4` |
| 200 | `pf-29-s01-pf29s01-blocked-df875e9f123c-2026-10-06.mp4` |
| 200 | `pf-29-s01-pf29s01-isolated-after-restart-df875e9f123c-2026-10-06.mp4` |
| 200 | `pf-29-s02-pf29s02-migrate-and-save-821c0b8f7ef3-2026-10-06.mp4` |
| 200 | `pf-30-s01-pf30-labelled-tool-output-61b75ec43ad5-2026-10-05.mp4` |
| 200 | `pf-30-s02-pf30s02-memory-labelled-01a3a88330b9-2026-10-06.mp4` |
| 200 | `pf-30-s03-pf30s03-vault-needs-human-d31a15ae53cc-2026-10-06.mp4` |
| 200 | `pf-33-s01-pf33s01-dns-answers-b40018175d73-2026-10-06.mp4` |
| 200 | `pf-33-s01-pf33s01-redirect-reauthorized-b40018175d73-2026-10-06.mp4` |
| 200 | `pf-33-s02-pf33s02-pinned-dial-cde3d0f49988-2026-10-06.mp4` |
| 200 | `pf-41-s01-pf41-inspector-live-aggressive-99b530830976-2026-10-07.mp4` |
| 200 | `pf-41-s01-pf41-inspector-taint-denial-99b530830976-2026-10-07.mp4` |

## Index

Verbatim, `qa/demos/index/P0-security-reel.md` lines 1-15:

> # P0 security levels: sprint demo reel
>
> One narrated video of the P0 security-levels work, edited from the sprint gate videos indexed in this folder.
> It is an asset on the [`demos` prerelease](https://github.com/CorbanuCore/CorbanuTerminal/releases/tag/demos).
>
> - Video: [p0-security-levels-reel-2026-10-08.mp4](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/p0-security-levels-reel-2026-10-08.mp4) (11:51, 1920x1080, 30 fps, H.264/AAC) ·
>   subtitles: [p0-security-levels-reel-2026-10-08.srt](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/p0-security-levels-reel-2026-10-08.srt)
> - Every terminal frame is a published GLM 5.2 gate recording. Clips are trimmed to the moment that proves the
>   sprint's acceptance criterion; waiting is sped up (an on-screen badge shows the rate) and proof frames are held.
>   Added on top: chapter and evidence cards, a side panel naming the sprint and criterion, highlights on the
>   proving lines, and burned-in subtitles.
> - Narration: Breeze TTS, synthetic voice `corbanu-narrator-calm-v1` (portable voice library). Not yet
>   listening-approved by a person.
> - Scope: candidates recorded 2026-10-05 to 2026-10-07 (each clip's commit is in its panel). The milestone gates (code-blind VM run, human sign-off,
>   flag removal) are not part of this reel.

Verbatim, `qa/demos/index/P0-security-reel.md` lines 17-33:

> ## Chapters
>
> | Time | Chapter | Sprints | Source clips |
> | --- | --- | --- | --- |
> | 00:00 | Intro | | |
> | 00:30 | 1. Secretless launch and the credential broker | PF-27-S02, PF-27-S04 | [`pf27s02-secretless-env`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s02-pf27s02-secretless-env-03c9dfd9716d-2026-10-06.mp4) · [`pf27s02-protected-paths`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s02-pf27s02-protected-paths-03c9dfd9716d-2026-10-06.mp4) · [`pf27-broker-crash-fails-closed`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s04-pf27-broker-crash-fails-closed-1a4370992f5a-2026-10-05.mp4) |
> | 01:38 | 2. The secret output gate | PF-28-S01, PF-28-S02 | [`pf28s01-tool-output-gated`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-28-s01-pf28s01-tool-output-gated-9261bd416f0d-2026-10-06.mp4) · [`pf28s01-persistence-clean`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-28-s01-pf28s01-persistence-clean-9261bd416f0d-2026-10-06.mp4) · [`pf28s02-reflected-credential`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-28-s02-pf28s02-reflected-credential-10357ba50c80-2026-10-06.mp4) |
> | 02:34 | 3. URL policy and connection pinning | PF-33-S01, PF-33-S02 | [`pf33s01-redirect-reauthorized`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-33-s01-pf33s01-redirect-reauthorized-b40018175d73-2026-10-06.mp4) · [`pf33s01-dns-answers`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-33-s01-pf33s01-dns-answers-b40018175d73-2026-10-06.mp4) · [`pf33s02-pinned-dial`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-33-s02-pf33s02-pinned-dial-cde3d0f49988-2026-10-06.mp4) |
> | 03:51 | 4. Protected mode: preflight and migration | PF-29-S01, PF-29-S02 | [`pf29s01-blocked`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-29-s01-pf29s01-blocked-df875e9f123c-2026-10-06.mp4) · [`pf29s02-migrate-and-save`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-29-s02-pf29s02-migrate-and-save-821c0b8f7ef3-2026-10-06.mp4) · [`pf29s01-isolated-after-restart`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-29-s01-pf29s01-isolated-after-restart-df875e9f123c-2026-10-06.mp4) |
> | 05:15 | 5. Untrusted content and taint | PF-30-S01, PF-30-S03, PF-23-S02 | [`pf30-labelled-tool-output`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-30-s01-pf30-labelled-tool-output-61b75ec43ad5-2026-10-05.mp4) · [`pf30s03-vault-needs-human`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-30-s03-pf30s03-vault-needs-human-d31a15ae53cc-2026-10-06.mp4) · [`pf23s02-hook-write-blocked`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s02-pf23s02-hook-write-blocked-93e149e066f0-2026-10-07.mp4) |
> | 06:22 | 6. Memory, level floors and inheritance | PF-30-S02, PF-23-S03, PF-24-S03 | [`pf30s02-memory-labelled`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-30-s02-pf30s02-memory-labelled-01a3a88330b9-2026-10-06.mp4) · [`pf23s03-project-cannot-lower`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s03-pf23s03-project-cannot-lower-a230f2082141-2026-10-07.mp4) · [`pf24-child-inherits`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s03-pf24-child-inherits-7f6c88912c14-2026-10-06.mp4) |
> | 07:19 | 7. The /security picker | PF-24-S02 | [`pf24s02-glm-confirm`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s02-pf24s02-glm-confirm-e4d17dbdc6f0-2026-10-07.mp4) · [`pf24s02-glm-cancel`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s02-pf24s02-glm-cancel-e4d17dbdc6f0-2026-10-07.mp4) · [`pf24s02-glm-downgrade`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-24-s02-pf24s02-glm-downgrade-e4d17dbdc6f0-2026-10-07.mp4) |
> | 08:28 | 8. The effective security inspector | PF-41-S01 | [`pf41-inspector-live-aggressive`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-41-s01-pf41-inspector-live-aggressive-99b530830976-2026-10-07.mp4) |
> | 09:04 | 9. Temporary grants | PF-25-S01 | [`pf25s01-grant-once`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-25-s01-pf25s01-grant-once-aee150e46ae7-2026-10-07.mp4) |
> | 09:45 | 10. Revocation and the kill switch | PF-25-S02 | [`pf25s02-revoke-grant`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-25-s02-pf25s02-revoke-grant-be5d33660c7e-2026-10-07.mp4) · [`pf25s02-kill-switch-restart`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-25-s02-pf25s02-kill-switch-restart-be5d33660c7e-2026-10-07.mp4) |
> | 10:38 | 11. Saved-level qualification | PF-13-S07, PF-23-S03 | [`pf23s03-stored-level-next-session`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s03-pf23s03-stored-level-next-session-a230f2082141-2026-10-07.mp4) · PF-13-S07 gate record (evidence card) |
> | 11:19 | Summary and what is still ahead | | |

Verbatim, `qa/demos/index/P0-security-reel.md` lines 35-40:

> ## Cut for length
>
> - [`pf41-inspector-taint-denial`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-41-s01-pf41-inspector-taint-denial-99b530830976-2026-10-07.mp4) (PF-41-S01): The inspector shows taint and recent denials.
> - [`pf25s01-grant-until-expiry`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-25-s01-pf25s01-grant-until-expiry-aee150e46ae7-2026-10-07.mp4) (PF-25-S01): A grant until expiry is listed in /security.
> - [`pf25s02-kill-switch-off`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-25-s02-pf25s02-kill-switch-off-be5d33660c7e-2026-10-07.mp4) (PF-25-S02): Turning the kill switch off takes a deliberate choice, and keeps the level.
> - [`pf13s07-composition-hello`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-13-s07-pf13s07-composition-hello-64137b71894f-2026-10-06.mp4) (PF-13-S07): With every credential-boundary flag on, a normal coding task still works.
