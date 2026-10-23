Read-only audit. Do not change anything, push, merge, or touch credentials. Read sec-common.md (binding).
Workstream: active plan "2. Accounting — unified agent cost and usage" (docs/plans/active/portfolio-agent-cost-accounting.md
in CorbanuCore/CorbanuTerminal). Travis believes a lot of accounting work happened recently, later than the plan file's
2026-09-14 date. Establish the true state FEATURE BY FEATURE, SPRINT BY SPRINT:
- the plan's features and their sprints (docs/sprints/current, docs/sprints/archive), sprint statuses, gate records, qa/;
- merged PRs (`gh pr list -s merged --search ...`, keywords: accounting, cost, usage, ledger, native anthropic, pricing,
  budget, throttle), their merge dates and which sprint each belongs to;
- LOCAL unmerged work: scan every git worktree (`git worktree list` in /Volumes/CorbanuDrive/Corbanu and its repos),
  local branches with commits not on origin/main, stashes, .codex-work dirs (e.g. manager-accounting-retention-*,
  corbanu-acct), and the RTX box (ssh ambient@100.99.88.49, read-only) for accounting branches/commits;
- open issues/PRs about accounting (incl. the recurring "Native Anthropic accounting failed" errors and their fixes).
Output a table: feature -> sprint -> status (done+evidence / merged-not-gated / local-only / not started) -> evidence links
-> what's missing (tests, review, demo video). List local-only commits precisely (repo, branch, sha, summary).
Recommend which sprints are complete enough to get demo videos now. Write it to
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/acct-audit1.result.md (under 120 lines).
