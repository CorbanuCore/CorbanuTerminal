#320 is fixed and merged as PR #327 (merge commit `7a11f906`). It's closed with the evidence below.

**The fix**
- `spawn_protected` now uses #321's approach instead of a second mechanism. I moved #321's holder process into one shared piece of code (`codex_process_hardening::HandleHolder`) that the logon launcher and the broker start both use.
- The broker's stdout write end is never inheritable in Core's own handle table. It's copied into a holder process that never runs and that no other process can open, and the broker is started as that holder's child.

**Regression test** (`sec_win_320_protected_stdout_never_reaches_other_children`): 300 protected spawns raced against ordinary child processes, counting children that held a protected stdout pipe.

| | Elevated | Normal session (medium) |
|---|---|---|
| Real Windows 11, before | 25 of 102 (fail) | 32 of 102 (fail) |
| Real Windows 11, after | 0 of 319 | 0 of 323 |
| CI `windows-security-probes`, after | 0 of 252 | 0 of 237 |

The test is in the `windows-security-probes` job as its own step, so a failure in the #307 test can't stop it from running.

**Gate**
- **Real machine, after the fix, both sessions:**
  - The process-hardening probes for PF-27-S06 and PF-27-S07 pass.
  - The #307 launcher race still finds 0 of 965 children holding a pipe.
  - The broker tests pass.
  - The #295 launcher probe passes.
- **Clippy:** Windows and Linux (on the RTX box) are clean with `-D warnings` for the three changed crates: process-hardening, windows-sandbox and network-proxy.
- **CI:** 34 checks passed and 9 were skipped; nothing failed.
- **Review:** the Opus 5.5 High review approved it with no blocking issues. I fixed its comment-accuracy nits and moved the #320 test into its own CI step. The other low findings are open (see below).
- **Machine quirk:** some PF-27-S06 and PF-27-S07 probes failed at first on this machine on main as well. That's the machine setup, not this change; the final full run passed them in both sessions.

**Open, low priority (from the review)**
- No test checks that the holder process can't be opened. If someone later swapped in the default permissions, a process running as the same user could take the broker's stdout pipe out of it, and no current test would fail. Worth a follow-up probe.
- If Core crashes in the few milliseconds before the holder is ended, a suspended holder is left that a non-admin user can't kill. Existing code can already leave a suspended broker behind the same way, and the holder only keeps a dead pipe end.
- Every broker start now sets a different parent process, which security software often flags as "PPID spoofing". It may need a release note or a check against Defender for Endpoint.

**Cleanup:** On Windows I deleted `C:\CorbanuQA\win6` and the two scheduled tasks (`w6-high`, `w6-medium`). The temporary user `w6logon` was removed in the earlier run, and nothing is left in Temp. The Mac worktree and branch are gone, and the RTX clone was already removed. `C:\CorbanuQA\target` and `C:\CorbanuQA\win5` are untouched, and I didn't touch #300, #301, #304, #323 or #326. Windows logs are in `.codex-work/workers-20261002/sec-win6-logs/` and the review is in `sec-win6-review-320/`.