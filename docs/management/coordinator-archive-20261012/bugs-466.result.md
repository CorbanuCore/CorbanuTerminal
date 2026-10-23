All four issues are fixed, each in its own PR, and all four PRs are merged with every check green. GitHub closed the issues automatically on merge, and I posted the evidence on each issue.

| Issue | PR | Merge commit |
|---|---|---|
| #467 `.env` reported as not isolated under `/tmp` | #479 | 83f8eb800e |
| #466 approved command stuck on "Running" | #487 | 07c280d770 |
| #468 /spawn Max saves the main model | #484 | 8d14383ca0 |
| #446 flaky Windows test | #478 | 0f498f72e4 |

## What changed

- **#467:** The sandbox was denying `.env` all along; only the launch check's report was wrong. It now compares the two paths after resolving `/tmp` → `/private/tmp`, the same way the sandbox treats them. A path no rule denies is still reported. The 4 security unit tests that failed with `TMPDIR=/tmp` now pass.
- **#466:** The case in the report (an approved command the sandbox stops at start) was already fixed on main by the #443 follow-up (492a40066b). I confirmed that in tmux. The same symptom remained for an approved command that fails to start for another reason, e.g. an approved escalation refused by the protected launch. I reproduced that on main, and Core now ends those commands as failed. A declined command still gets only its "Did not run" line, with no duplicate.
- **#468:** Max or Ultra under "More reasoning…" now spawns the agent (or opens the new pane) with that effort. The main model is neither changed nor saved, and Ultra keeps its usage warning.
- **#446:** The test no longer changes the folder owner when the folder is already owned by the user, which is the call that was failing. It now asserts the folder is user-owned before checking the refusal, so it proves at least as much as before.

## Gate evidence

- **Regression tests:** each PR adds them. The #466 and #467 tests fail with the fix disabled. The #468 test exercises a new code path, so it can't compile on the old code; tmux covers that case instead.
- **Focused tests:**
  - #467: 88 TUI security tests pass with `TMPDIR=/tmp` and the default.
  - #466: 81 core integration and 662 core lib tests pass.
  - #468: 184 popup tests pass.
- **Linux clippy `-D warnings` on glitch:** clean, plain and developer-accounting, on every pushed head. macOS clippy is clean too.
- **Independent Opus 5.5 High reviews:** every PR ends at APPROVE.
  - #467 took three rounds. Round 1 caught a real problem in my first version: it could report a path as protected when the macOS sandbox would still let it be read. I replaced it.
  - #466 took two rounds; round 1 found that decisions by the automatic approval reviewer weren't covered. Fixed.
  - #468 and #446 were approved first time.
- **tmux functional checks (GLM 5.3 Flash, disposable homes, keyring off):**
  - #468: the Troll spawned with `reasoning_effort="max"` under both Permissive and Aggressive, and config.toml was unchanged.
  - #466: before the fix, the cell stayed "◦ Running touch ../escalated.txt" after the turn ended. After it, the cell shows "• Ran … Protected launch refused".
  - Screen captures are in `.codex-work/fix466-run/evidence/`.
- **#446 stability:** the `pf_27_s06 probes (windows-2022)` job passed 4 out of 4 runs (the original plus 3 reruns).

## Open

- **#488 (new, not fixed):** found by the #467 review and present on main before my change. On macOS, if the workspace is opened through a link deeper than `/tmp`, the launch check can say a credential file is protected when the sandbox still lets agent commands read it. The issue lists two options:
  - Make the check mirror exactly how the macOS sandbox applies its denials.
  - Change the sandbox itself, which would be a product initiative.
- **Cleanup:** my worktrees, build targets (62 GB locally, 24 GB on glitch), the glitch clone and the disposable homes are removed. My glitch clippy logs are kept in `~/corbanu-rtx/fix466-logs/`.