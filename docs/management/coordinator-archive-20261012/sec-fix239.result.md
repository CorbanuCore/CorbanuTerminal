I finished the #239 fix: PR #244 is merged into main (merge commit `824ce30b05`) and #239 is closed. Under Moderate, credential locations are now unreadable from the start of the session. Before, they were only protected after untrusted content arrived.

**Picking up GLM's work:** I continued in GLM's worktree `sec-round4-issue239` on branch `fix/pf-13-s07-issue-239-files-read`. Its edits had broken an existing test (`pf_30_s03_approval_is_bound_to_taint_and_policy`), so I repaired that and updated two other tests that the new behaviour changed.

**What changed:**
- **Moderate now matches Aggressive here.** Commands, file tools and Codex Apps uploads can't read Corbanu home stores, other Corbanu homes or known `$HOME` credential paths (`~/.ssh`, `~/.aws`, …) from the first command. Permissive and flag-off sessions are unchanged, as decided.
- **Users can still get past the block for one command.** The first fix left Moderate users with no way to approve a command that needs to run outside the sandbox, so the reviewer blocked it twice. Now, before untrusted content arrives, such a command always asks the human, even where it used to run without asking (full access with on-request approvals, exec-policy allow rules). Cached approvals, hooks and automatic review can't approve it. Approving lifts the protection for that one command, unless untrusted content arrived while the human was deciding. With approvals off, the command is refused.
- **Ordinary files are not covered.** The qualification probe read `$HOME/canary.txt`, and a file like that is still readable at every level; only known credential locations are blocked.

**Gate:**
- **Focused tests:** `just test -p codex-core` with the security and approval filters passed 215/215, including new `issue_239_*` tests (unit tests plus a macOS end-to-end test).
- **Full suite:** `just test -p codex-core` passed 3903 of 3907. The 4 failures look unrelated to this change: `config_schema_matches_fixture` fails on main too (I checked on the RTX box), and the other three (two skills tests and one compaction test) don't touch any code this PR changed.
- **Linux clippy** on the RTX box (`-p codex-core --tests -D warnings`): clean on the final head `7beff6f122`.
- **Opus 5.5 High review:** rounds 1 and 2 asked for changes; round 3 approved. I made its small follow-up fixes (keeping the model's justification on the prompt, more test cases) after that approval, and those weren't re-reviewed. The reviews are in `.codex-work/workers-20261002/fix239-review/`.
- **PR CI:** all green.
- **Not done:** no GLM tmux run or videos, since Z.AI is out of credit.

**Still open:**
- **#245** (new, pre-existing): a command's extra permissions can override these blocks one path at a time. That feature is off by default.
- After untrusted content arrives, a Moderate command that isn't classed as protected but needs to run outside the sandbox still can't get approved. This was already true before this PR.
- Known gap, noted in the PF-13-S07 evidence README: before untrusted content arrives, typing into a shell that was started without the protection doesn't ask first.
- The Aggressive/Moderate re-run of the qualification attack tests and true-TUI proof still wait for Z.AI credit.

Report: `.codex-work/workers-20261002/sec-fix239.result.md`. The worktree, its build directory and the RTX clone and build directory are removed.