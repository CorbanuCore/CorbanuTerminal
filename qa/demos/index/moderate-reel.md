# The Moderate security level: demo reel

One narrated video of the Moderate security level, signed off by Travis on 2026-10-20 (D61), and the user-facing
fixes shipped since: #1171, #1172, #1175, #1135, #1096 and #1185 (#1190, small `/cost` display fixes, is not in the
recorded build `2f9d05c51e` and is not shown). It is an asset on the
[`demos` prerelease](https://github.com/CorbanuCore/CorbanuTerminal/releases/tag/demos).

- Watch online: <https://corbanu-moderate-reel.cloudflare-share.workers.dev/> (published 2026-10-21; subtitles are burned in, and
  the page also offers them as an English text track). Direct MP4: <https://corbanu-moderate-reel.cloudflare-share.workers.dev/video.mp4>,
  byte-identical to the release asset.
- Video: [moderate-reel-2026-10-21.mp4](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/moderate-reel-2026-10-21.mp4) (9:02, 1920x1080, 30 fps, H.264/AAC, chapters
  embedded) · subtitles: [moderate-reel-2026-10-21.srt](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/moderate-reel-2026-10-21.srt)
- Security levels are behind the default-off `security_levels` flag.
- Milestone: all 34 cases 30 passed and 4 accepted by owner decision ([record](../../security-levels/milestones/moderate-targeted2-20261020/README.md),
  PR #1037), signed off on 2026-10-20 (D61). The later runs do not change the sign-off: the local-model re-run (#1078:
  MOD-21 passed; MOD-09 failed on item 7, whose behaviour D72 accepts (prompt wording #1077), and on item 5, #1087),
  the second local-model re-run
  ([record](../../security-levels/milestones/localmodel-rerun2-20261021/README.md), #1191: MOD-09 items 5 and 7 passed, item 7 as D72 amends it) and the Tart parity run
  (#1149, which found #1139 and #1146, fixed by #1171 and #1175). Chapter 8 shows the runs and the limits accepted at
  sign-off.
- Every terminal frame was recorded on 2026-10-21 for this reel with `scripts/demo_video.py` (GLM 5.3 Flash on Z.AI,
  disposable profiles, synthetic credentials only), on the installed signed build `2f9d05c51e` (SHA-256 `a4f9804b…`).
  It is a developer build (Cargo `dev` profile with debug assertions, `developer_marker: true` in its INSTALL.json), so
  `CORBANU_TEST_NO_NATIVE_KEYRING=1`, which `demo_video.py` requires, kept the profiles off the macOS keychain, as with
  the SOP's debug builds.
  The two "before" clips in chapter 9 use the previous signed build `aa35e3f936`, also a developer build, recorded with
  the same isolation. The clips are indexed in
  [MOD-REEL.md](MOD-REEL.md); their specs are `qa/demos/specs/modreel-*.toml` plus the existing `issue747`, `issue750`,
  `issue839` and `issue840` specs.
- Clips are trimmed to the moment that proves the protection; waiting is sped up (an on-screen badge shows the rate)
  and proof frames are held. Added on top: chapter cards, two evidence cards, a side panel naming the issue and the
  criterion, highlights on the proving lines, and burned-in subtitles.
- Narration: Breeze TTS, synthetic voice `corbanu-narrator-calm-v1` (the earlier reels' voice, portable voice library).

## Chapters

| Time | Chapter | Sprints / issues / decisions | Source |
| --- | --- | --- | --- |
| 00:00 | Intro | | |
| 00:32 | 1. Choosing Moderate | #750, D4, D61 | [`issue750-moderate-choose-confirm-restart`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/mod-reel-issue750-moderate-choose-confirm-restart-2f9d05c51e42-2026-10-21.mp4) |
| 01:24 | 2. Normal work stays normal | MOD-06 | [`modreel-normal-work`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/mod-reel-modreel-normal-work-2f9d05c51e42-2026-10-21.mp4) |
| 01:44 | 3. Commands stay in the sandbox | #747, D3, MOD-07 | [`issue747-moderate-stays-in-sandbox`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/mod-reel-issue747-moderate-stays-in-sandbox-2f9d05c51e42-2026-10-21.mp4) |
| 02:09 | 4. Protected files | #748, #838, #841, MOD-03, MOD-10 | [`modreel-protected-files`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/mod-reel-modreel-protected-files-2f9d05c51e42-2026-10-21.mp4) |
| 02:53 | 5. Untrusted content | PF-30, PF-23, #900, #949, MOD-14, MOD-15 | [`modreel-untrusted-credential`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/mod-reel-modreel-untrusted-credential-2f9d05c51e42-2026-10-21.mp4) · [`modreel-approvals-off`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/mod-reel-modreel-approvals-off-2f9d05c51e42-2026-10-21.mp4) |
| 03:45 | 6. Rules saved under Permissive | #840, D6, MOD-08 | [`issue840-moderate-keeps-permissive-rule`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/mod-reel-issue840-moderate-keeps-permissive-rule-2f9d05c51e42-2026-10-21.mp4) |
| 04:11 | 7. Inspector and kill switch | PF-41-S01, #839, PF-25-S02, MOD-19, MOD-20 | [`issue839-inspector-lists-moderate-refusals`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/mod-reel-issue839-inspector-lists-moderate-refusals-2f9d05c51e42-2026-10-21.mp4) · [`modreel-kill-switch`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/mod-reel-modreel-kill-switch-2f9d05c51e42-2026-10-21.mp4) |
| 04:59 | 8. The milestone and its accepted limits | Moderate ships, #844, #971, #1037, D61 | evidence card: [first run](../../security-levels/milestones/moderate-20261014/README.md) · [targeted re-run](../../security-levels/milestones/moderate-targeted-20261016/README.md) · [second targeted re-run and sign-off](../../security-levels/milestones/moderate-targeted2-20261020/README.md) · [local-model re-run](../../security-levels/milestones/moderate-localmodel-20261020/README.md) · [second local-model re-run](../../security-levels/milestones/localmodel-rerun2-20261021/README.md) · [Tart parity run](../../security-levels/milestones/moderate-tart-parity-20261020/README.md) · evidence card: [sign-off addendum](../../security-levels/milestones/moderate-targeted2-20261020/README.md#sign-off-2026-10-20) |
| 06:33 | 9. Fixes since the sign-off | #1171, #1172, #1175, #1135, #1096, #1185 | [`modreel-1171-before`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/mod-reel-modreel-1171-before-aa35e3f93658-2026-10-21.mp4) · [`modreel-1171-git-write-named`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/mod-reel-modreel-1171-git-write-named-2f9d05c51e42-2026-10-21.mp4) · [`modreel-1172-before`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/mod-reel-modreel-1172-before-aa35e3f93658-2026-10-21.mp4) · [`modreel-1172-after`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/mod-reel-modreel-1172-after-2f9d05c51e42-2026-10-21.mp4) · [`modreel-1175-spawn-task`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/mod-reel-modreel-1175-spawn-task-2f9d05c51e42-2026-10-21.mp4) · [`modreel-1135-protected-launch`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/mod-reel-modreel-1135-protected-launch-2f9d05c51e42-2026-10-21.mp4) · [`modreel-1096-login-status`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/mod-reel-modreel-1096-login-status-2f9d05c51e42-2026-10-21.mp4) · [`modreel-1185-cost-proven`](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/mod-reel-modreel-1185-cost-proven-2f9d05c51e42-2026-10-21.mp4) |
| 08:35 | Summary and what is still ahead | | |

## Notes

- Chapter 9 shows "before" clips only for #1171 and #1172. For #1175 the previous build delivered an ordinary task the
  same visible way (the difference is how the child's model receives it), so the clip shows only the fixed build.
- The #1135 clip runs under Permissive with protected launch (`secretless_agent_launch`) on, the only setup #1135
  changed; Moderate and Aggressive already denied those entries.
- Paths on screen are the disposable demo directories under `/Volumes/CorbanuDrive/...`; every credential in the
  clips is synthetic (`~/.ssh/config` and vault-folder canaries, an API key used only by `corbanu login`), and the
  Z.AI key never appears.
