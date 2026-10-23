# Named accounts per provider: PF-84 demo reel

One narrated video of PF-84 "named accounts per provider" (sprints S01–S06), edited from the PF-84 demo videos indexed
in [PF-84-S02.md](PF-84-S02.md) … [PF-84-S06.md](PF-84-S06.md) and the follow-up fixes ([ISSUE-545.md](ISSUE-545.md),
[ISSUE-555.md](ISSUE-555.md), [issue-653.md](issue-653.md)). It is an asset on the
[`demos` prerelease](https://github.com/CorbanuCore/CorbanuTerminal/releases/tag/demos).

- Watch online: <https://corbanu-named-accounts-reel.cloudflare-share.workers.dev/> (published 2026-10-14; subtitles are burned in, and the page also
  offers them as an English text track). Direct MP4: <https://corbanu-named-accounts-reel.cloudflare-share.workers.dev/video.mp4>, the same file as the release asset.
- Video: [named-accounts-reel-2026-10-13.mp4](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/named-accounts-reel-2026-10-13.mp4) (8:12, 1920x1080, 30 fps, H.264/AAC, chapters
  embedded) · subtitles: [named-accounts-reel-2026-10-13.srt](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/named-accounts-reel-2026-10-13.srt)
- Everything shown is behind the default-off `named_accounts` flag. The owner accepted S01–S06 as limited testing:
  ChatGPT sign-ins and Amazon Bedrock were verified against mock servers with fake tokens and keys only (see the
  [S06 closure](../../../docs/sprints/archive/unified-provider-auth/pf-84-s06-aws-and-chatgpt-accounts.md)). The
  flag-removal prerequisites (live ChatGPT and Bedrock checks, re-checks, an isolated code-blind run) are in the
  [plan's PF-84 section](../../../docs/plans/completed/main-2026-10-14-unified-provider-auth.md); the reel says they remain.
- Every terminal frame is a published demo recording (GLM 5.3 Flash on Z.AI, or GPT-5.4 against the OpenAI and
  Bedrock mocks). Clips are trimmed to the moment that proves the criterion; waiting is sped up (an on-screen badge
  shows the rate) and proof frames are held. Added on top: chapter cards, one evidence card, a side panel naming the
  sprint and criterion, highlights on the proving lines, and burned-in subtitles.
- Rename and remove have no demo clip; chapter 1's evidence card summarises the independent acceptance verdicts
  (S04-3, S04-4, S04-10, S04-13; S06 A8 and B3). Per-account rate-limit figures are not shown: they come only from
  ChatGPT sign-ins, which read "not available" with the mock; the narration says so.
- Not used: `pf84-595` (its daemon-reuse path was replaced by #658, and it shows a home path), and S01's home-override
  clips (worker homes, not named accounts).
- Criteria: the PF-84 sprint records in [docs/sprints/archive/unified-provider-auth/](../../../docs/sprints/archive/unified-provider-auth/)
  and the independent acceptance READMEs under [qa/provider-auth/pf-84/](../../provider-auth/pf-84/).
- Narration: Breeze TTS, synthetic voice `corbanu-narrator-calm-v1` (the earlier reels' voice, portable voice
  library). Not yet listening-approved by a person.

## Chapters

| Time | Chapter | Source clips |
| --- | --- | --- |
| 00:00 | Intro | |
| 00:42 | 1. Adding and managing accounts | [`pf84-add-account`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s04-pf84-add-account-7a1ab9bc9ddb-2026-10-10.mp4) · evidence card: independent acceptance [S04](../../provider-auth/pf-84/independent-acceptance-s04-20261011/README.md), [S06](../../provider-auth/pf-84/independent-acceptance-s06-20261011/README.md), [S06 slice B](../../provider-auth/pf-84/independent-acceptance-s06b-20261013/README.md) |
| 01:51 | 2. Each account uses only its own credential | [`pf84-account-isolation`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s02-pf84-account-isolation-dcbe51a94e33-2026-10-10.mp4) · [`pf84-exec-account`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s03-pf84-exec-account-047d9e10392d-2026-10-10.mp4) |
| 02:46 | 3. Switching accounts mid-session | [`pf84-switch-account`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s04-pf84-switch-account-7a1ab9bc9ddb-2026-10-10.mp4) · [`pf84-switch-follow`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s06-pf84-switch-follow-40cbb034ec7e-2026-10-12.mp4) |
| 03:42 | 4. Resume and fork keep the account | [`pf84-resume-recorded-account`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-545-pf84-resume-recorded-account-218c0d3650bd-2026-10-11.mp4) · [`pf84-555-fork-account`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-555-pf84-555-fork-account-ae55961c40fd-2026-10-11.mp4) |
| 04:24 | 5. Amazon Bedrock accounts are AWS profiles | [`pf84-aws-profile-account`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s06-pf84-aws-profile-account-bffe8d916c0b-2026-10-11.mp4) |
| 05:06 | 6. ChatGPT sign-ins and the default login | [`pf84-chatgpt-account`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s06-pf84-chatgpt-account-4faac3c1ef78-2026-10-11.mp4) · [`pf84-named-session-keeps-default-login`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s06-pf84-named-session-keeps-default-login-064dad64b873-2026-10-12.mp4) · [`issue658-named-session-backend-account`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s06-issue658-named-session-backend-account-63075b25428e-2026-10-13.mp4) |
| 06:23 | 7. Usage per account | [`pf84-status-account-tokens`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s05-pf84-status-account-tokens-20528c9d785e-2026-10-11.mp4) |
| 07:00 | 8. Choosing the connection in /model | [`issue653-model-route-choice-permissive`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-653-issue653-model-route-choice-permissive-8a9efe1a9390-2026-10-13.mp4) |
| 07:35 | Summary and what is still ahead | |

## Cut for length

- [`pf84-bedrock-from-zai`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s06-pf84-bedrock-from-zai-ca46cf7ba80a-2026-10-12.mp4): PF-84 #572: a named Bedrock account reached from a Z.AI session through /model.
- [`pf84-609-default-row-names-account`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-609-pf84-609-default-row-names-account-a46013dbb25c-2026-10-13.mp4): PF-84 #609: the default row names the account its status is about.
