This was a product bug, not a problem with the QA machine, and real users will hit it: anyone who runs Corbanu Terminal (TUI or `exec`) over SSH into Windows, or from a service. It's fixed and merged as PR #343 (`a141b3e7`), and issue #341 is closed.

**Root cause**
- **Runner:** over plain SSH, Corbanu runs on a non-interactive window station (`Service-0x0-<id>$`) that only the SSH user and Administrators can use.
  - The elevated sandbox starts its command runner as the sandbox user, and Windows only gives that user access on the console's window station (`WinSta0`), not the SSH one.
  - The runner crashes at startup (0xC0000142, System event 26), and Core gives up after 15 s with "timed out … connecting runner pipe-in".
- **Commands:** once the runner could start, its commands crashed the same way, because their desktop was named under `WinSta0`.
- **Not the reboots or a recent merge:**
  - The earlier gates that passed ran in the console session through scheduled tasks.
  - The same timeout appears over SSH on 2026-10-08, before any update (win-gate1).
  - It fails on 265172be (the S07 gate commit) and on main alike.
  - The old and new OpenSSH servers behave the same.
- **Same bug in OpenAI's Codex:** openai/codex#37722 and #46412.

**What I changed**
- **Grant:** when Core isn't on the console's window station, it now gives the sandbox user the least access its runner and commands need before starting the runner. I measured each right on the real machine; hooks, window creation, clipboard and permission changes are not granted.
- **Desktop names:** commands' desktops are named in the runner's own window station. The console session behaves exactly as before.
- **Errors:** if the grant fails, the launch still runs, and the reason is added to any timeout message instead of a bare 15 s timeout.
- **Tests:** two new regression tests reproduce the crash on main and pass with the fix. CI's expected count for these Windows sandbox tests (`sec_win_`) goes from 13 to 15, and the elevated CI step now fails if the new tests skip.
- **Docs:** `docs/sandbox.md` has a short "Over SSH" note.

**Gate**
- **Tests:** all 15 `sec_win_` tests pass elevated and at medium integrity, over SSH, in the console session and on CI. At medium the two new tests skip, because a normal session can't create a window station; the elevated runs exercise them.
- **Clippy:** Windows (`-D warnings`) is clean. Linux clippy on the RTX box was clean at the first commit; the box went offline after that, and PR CI's Linux clippy passed on the final commit.
- **Real machine:**
  - Over SSH, `corbanu sandbox` (whoami and PowerShell) works in read-only and workspace-write, with private and shared desktops.
  - `corbanu exec` with GLM 5.2 runs commands and writes a file at high and medium integrity.
  - The console session still works.
- **Opus 5.5 High review:** round 1 requested changes, all addressed; round 2 approved, and I fixed its minor points.

**Still open**
- **#345 (new):** the access is granted to the sandbox accounts, so the sandboxed commands hold it too, and it lasts until the SSH session ends. Narrowing it to the runner and its lifetime is tracked there.
- **Non-admin sandbox over SSH:** main fails in both modes. The default (private desktop) now works. With `sandbox_private_desktop = false` it still fails, as before; noted on #345.
- **Other failures sec-win7 saw:** the five `unified_exec` test failures are a test-setup problem, not this bug. One elevated test can't find the runner executable, and the others then fail on the lock it leaves poisoned; run without it, they pass.
- **Shared sandbox passwords:** my elevated setups reset the machine-wide sandbox passwords, so any older normal-session seed is stale. The s08 worker had already removed its seed and key.

**Cleanup:** I removed `C:\CorbanuQA\win8`, my CorbanuQA-win8-* tasks, the temporary local users (no profiles left), and the Mac worktree; `C:\CorbanuQA\target` is kept. The RTX box is offline, so `~/corbanu-rtx/sec-win8*` is still there to remove. The full write-up is in `sec-win8.result.md` in the workers folder, with the reviews in `sec-win8-review-341*/`.