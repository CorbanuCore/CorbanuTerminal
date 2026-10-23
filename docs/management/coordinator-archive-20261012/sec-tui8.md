You are the TUI lane worker (round 8). Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md
first (binding: keychain rule, disk hygiene, per-sprint gate, Linux clippy on the RTX box) and sec-tui7.result.md.
Z.AI has credit again, so GLM 5.2 (provider zai) runs are possible.
1. PF-24-S02 (merged #253) was qualified with a mock model and its sprint record is left in `draft`. Run its GLM 5.2 tmux
   functional pass per the repo SOP, record short demo videos of confirm / cancel / downgrade / restart-now end to end,
   add them to the gate record and qa/demos index, fix the sprint record status, PR, merge when green, archive the sprint.
2. Implement PF-25-S01 "Temporary grant TUI", then PF-25-S02 "Revocation and kill-switch TUI"
   (docs/sprints/current/p0-security-levels/), each with its own PR and full per-sprint gate (including a GLM tmux run and
   short videos), merged when green and archived.
Another worker (sec-insp1) is building PF-41-S01, the security inspector, in new files; avoid its files.
Merge with `gh pr merge N --merge --match-head-commit <sha>`; if the sprint index conflicts, merge origin/main into your
branch (never force-push). Report as in sec-common.md.
