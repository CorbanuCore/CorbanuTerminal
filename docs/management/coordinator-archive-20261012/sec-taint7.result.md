Both are merged to main with all checks green: the GLM 5.2 memory summary fix (PR #238, `fbfe85e4f1`) and PF-23-S02 (PR #243, `6b1c8b8873`), which is now archived. I didn't start PF-23-S03, so PF-24-S02 is still waiting on it.

## What shipped
**PR #238: memory summaries with GLM 5.2**
- **Fix:** the parser now accepts an answer wrapped in one ```json or ``` code block. Text around the block, other tags, two blocks and unknown fields are still rejected.
- **Review catch:** my first version rejected answers whose values contained a code block; that's fixed and tested.
- **Result:** with GLM 5.2 the summary job now finishes "done / ok", and the log confirms GLM's answer was fenced.

**PR #243: PF-23-S02**, all behind the default-off `source_envelopes` flag, Moderate and Aggressive only.
- **Aggressive from the first command:** under Aggressive, Corbanu home and credential reads are blocked before any untrusted content arrives. An approval never lifts this; only a matching grant does.
- **Grants (`core/src/security/aggressive.rs`):** one check point for every sensitive surface.
  - A grant covers one exact command or process, for one agent and session, until its expiry or use limit, and only under the policy version it was issued in.
  - The kill switch, a level change or a missing policy cancels all grants.
  - Child agents never inherit them.
  - There's no screen to issue grants yet (PF-25-S01), so for now nothing lifts the Aggressive protections.
- **The four items moved in from PF-23-S01:**
  - **Upload race:** Codex Apps uploads now read the file through the protected sandbox, so a file swapped after the check can't be read. With no sandbox, the upload is refused.
  - **Writes:** files that run code later become read-only even with full access. That covers shell start-up files, login items, `~/.local/bin`, `~/.claude`, the repo's hooks, config, `commondir` and `.git` file, and project `.codex`/`.agents`. On macOS, renaming or removing a folder above one of them, or creating a missing parent of one, is also blocked.
  - **Command-text gap for writes:** the check now asks before writing, moving or removing any of those files.
  - **Older processes:** typing into a process started before untrusted content asks once under Moderate and needs a grant under Aggressive.

## Gate evidence
- **Tests:**
  - Focused tests: 99/99 after the final merge with main.
  - A real macOS sandbox test confirms the rename and missing-parent blocks.
  - Full `codex-core`: 3,897 of 3,902. Four failures also fail on main; the fifth is a flaky snapshot test that passes 3/3 when run alone.
- **Linux clippy** with warnings as errors on the RTX box: clean at the final head `8538f04968`.
- **Review (Opus 5.5 High):**
  - #238: two rounds, then approved.
  - PF-23-S02: five rounds, the last one approved. The first four found real bypasses, all fixed: renaming parent folders, removing protected links, creating missing parents, redirecting git through `commondir` or the `.git` file, and Linux placeholder files appearing in the real home folder.
  - Outputs are in `.codex-work/workers-20261002/{memfence-review1,2,pf23s02-review1..5}/`.
- **GLM 5.2 videos:**
  - [Memory summary completes](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/memory-fenced-json-memory-fenced-json-glm-49a9c877f141-2026-10-06.mp4)
  - [Aggressive blocks a hidden home read from the first command](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s02-pf23s02-aggressive-from-start-93e149e066f0-2026-10-07.mp4)
  - [Full access can't write a git hook](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s02-pf23s02-hook-write-blocked-93e149e066f0-2026-10-07.mp4)
  - [Writing ~/.zshrc asks first](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s02-pf23s02-persistence-write-asks-93e149e066f0-2026-10-07.mp4)
  - [Typing into an older shell asks once](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s02-pf23s02-older-process-asks-93e149e066f0-2026-10-07.mp4)
  - Grants and uploads have unit tests but no video: grants have no screen yet, and uploads need ChatGPT sign-in.

## Decision for you
On macOS, the rename block also applies outside the flag, to any sandbox policy that already blocks reads. That includes a user's own deny-read config and the Aggressive picker profile: folders above a blocked path can't be renamed, removed or created there. I kept it because it closes the same bypass for those denials, and recorded it in the gate record. If you'd rather keep everything strictly behind the flag, it's a small change.

## Still open
- **Linux, full-access profiles only:** renaming a folder above a protected path isn't blocked, and missing home files (such as `~/.bash_profile`), a missing `commondir` and symlinked dotfiles rely on the command-text check alone. A Linux sandbox test is still to do.
- **Only the command-text check covers:** submodule git folders, a hooks folder inside the workspace (such as `.husky`), and a new nested `.git`.
- **Adapters not built yet:** the broker, retrieval, browser-login and derived-data adapters must call `aggressive::admit` once they exist.
- **PR size:** #243 is about 2,700 added lines, mostly tests, which is over the 800-line guidance.

All worktrees and target folders (local and RTX) are removed. The report is saved as `.codex-work/workers-20261002/sec-taint7.result.md`, and the gate record is `qa/security-levels/sprints/PF-23-S02/gate.md`.