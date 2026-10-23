You are the untrusted-content lane worker (round 8). Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md
first (binding: keychain rule, disk hygiene), then sec-taint7.result.md (PF-23-S02 archived via #243).

1. PF-23-S03 "Downgrade, restart, and inheritance enforcement"
   (docs/sprints/current/p0-security-levels/pf-23-s03-downgrade-restart-and-inheritance-enforcement.md), including the
   items moved into it earlier (revocation trigger from PF-27-S04, merging step and summaries under Aggressive, the
   level a past session ran under). PF-24-S02 (TUI lane) is waiting on this sprint; finish it first.
2. Then, as small bounded PRs, close PF-23-S02's open items where practical: the Linux sandbox test for full-access
   profiles (folder-rename above protected paths, missing home files, missing commondir, symlinked dotfiles) and
   OS-level coverage for submodule git folders, in-workspace hooks folders and new nested .git.
Per-sprint gate per sec-common.md (GLM 5.2 runs, one independent Opus 5.5 High review, demo videos). Linux clippy on
the RTX box. Keep PRs near the 800-line guidance by splitting. PR to main; merge when green. Report as in sec-common.md.
