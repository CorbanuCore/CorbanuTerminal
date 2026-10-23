The reel is finished and uploaded to the `demos` release, and the docs PR is merged. It runs 11:51, which is inside the 8–12 minute target.

- **Video:** https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/p0-security-levels-reel-2026-10-08.mp4 (1920x1080, 30 fps, chapters embedded in the file)
- **Subtitles:** the same link ending in `.srt`
- **Index doc:** `qa/demos/index/P0-security-reel.md` lists the chapters and links every source clip. PR #274 merged as `40f4acafe8` after all checks passed.

**Chapters**
| Time | Chapter | Sprints |
| --- | --- | --- |
| 00:00 | Intro | |
| 00:30 | 1. Secretless launch and the credential broker | PF-27-S02, PF-27-S04 |
| 01:38 | 2. The secret output gate | PF-28-S01, PF-28-S02 |
| 02:34 | 3. URL policy and connection pinning | PF-33-S01, PF-33-S02 |
| 03:51 | 4. Protected mode: preflight and migration | PF-29-S01, PF-29-S02 |
| 05:15 | 5. Untrusted content and taint | PF-30-S01, PF-30-S03, PF-23-S02 |
| 06:22 | 6. Memory, level floors and inheritance | PF-30-S02, PF-23-S03, PF-24-S03 |
| 07:19 | 7. The /security picker | PF-24-S02 |
| 08:28 | 8. The effective security inspector | PF-41-S01 |
| 09:04 | 9. Temporary grants | PF-25-S01 |
| 09:45 | 10. Revocation and the kill switch | PF-25-S02 |
| 10:38 | 11. Saved-level qualification | PF-13-S07, PF-23-S03 |
| 11:19 | Summary | |

The reel uses 26 clips. All were recorded with GLM 5.2 except the evidence card in chapter 11. Each clip is trimmed to the moment that proves its criterion; waiting is sped up with an on-screen speed badge, and the proof frame is held. A side panel names the sprint and the criterion, and an amber box marks the lines that prove it. The terminal footage itself is not altered.

**Checks I ran**
- **Highlights:** all 43 boxes land on the right lines. I checked every proof moment in snapshots and fixed four that were on the wrong frame or line.
- **Narration:** a speech-to-text pass found every scripted word, with no omissions or repeats. Subtitles are timed to the actual spoken words.
- **Sync:** the audio is 0 ms off at every point I measured.
- **Secrets and paths:** no key-shaped strings and no `/Users/` home paths in any clip used, the subtitles or the generated graphics. The `home/` paths on screen are the throwaway test profiles.
- **Build check:** the HyperFrames check passes with no errors, and all text passes the contrast check.

**Needs your review**
1. **The narrator's voice.** I picked voice A (a calm woman, 143 words per minute) from three auditions, but I chose it on the transcript and pace, not by listening. Please play `.codex-work/demo-reel-20261008/voice/auditions.html`. Listen especially for how "Corbanu" is pronounced; the speech-to-text heard "Corbano". The saved voice package records that no person has approved it yet.
2. **Usage terms.** Breeze's terms allow research and non-commercial use only. I treated an internal QA reel as in scope, but it now sits on a GitHub release, so please confirm that's fine.
3. **Four clips cut to fit 12 minutes:**
   - Inspector taint-and-denials. The live-Aggressive inspector clip still shows those rows, but they are empty.
   - Grant until expiry. The revoke clip shows a listed grant.
   - Kill switch off.
   - PF-13-S07 "hello". Both PF-13-S07 recordings end before showing their result, so chapter 11 uses PF-23-S03's saved-level clip plus a card summarising the gate record's route results.
4. **Things visible on screen:**
   - The worktree paths under `/Volumes/CorbanuDrive/...`.
   - In the reflected-credential clip (PF-28-S02), a macOS per-user temp folder name under `/var/folders/...`. It isn't a home path or a secret.
5. **"Moderate"** appears as "not available yet" in the picker clips, and the narration says so.

The work folder is `.codex-work/demo-reel-20261008/`. It holds the script draft (`script/SCRIPT.md`), the build pipeline and the final files. My worktree is removed and no cargo target folder was created.