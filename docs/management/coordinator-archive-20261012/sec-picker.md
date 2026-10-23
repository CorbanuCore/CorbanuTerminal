You are the TUI lane sprint worker for PF-24-S03 "Flagged /security picker: Permissive and Aggressive".
Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md first (binding), then the sprint file
docs/sprints/current/p0-security-levels/pf-24-s03-flagged-security-picker.md and the plan
docs/plans/active/p0-security-levels.md on origin/main (24242c1b0e or later).

1. Allocate the sprint: new worktree + branch off origin/main, fill owner/write_scope/worktree/branch/base in the
   sprint file, keep both checkers (docs/plans/check.py, docs/sprints/check.py) passing.
2. Implement every Remaining item. The Aggressive mapping table is the contract; if a row can't be enforced with
   existing controls, stop and report rather than shipping a partial Aggressive.
3. Another worker (sec-tui) is fixing bugs #174 (typed text approves an approval prompt), #175 and #176 in the
   approval/permission popup code at the same time. Don't edit those files beyond what the sprint needs; if you
   must, merge origin/main first and keep changes minimal. Rebase-free: merge origin/main into your branch.
4. Per-sprint gate (sec-common.md decision 5): focused tests, GLM 5.2 tmux functional run, one independent
   Opus 5.5 High review with findings dispositioned, and short end-to-end videos of each core feature using the
   demo SOP (qa/demos/README.md from PR #177 if merged; otherwise check that branch codex/demo-video-sop-20261006 for
   the script, or fall back to asciinema .cast). Store evidence under qa/security-levels/sprints/PF-24-S03/.
5. PR to main; merge when all checks pass. Update the sprint file and index row; archive only if every gate item
   is done.
Report as in sec-common.md.
