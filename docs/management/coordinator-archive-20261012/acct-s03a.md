You are the accounting-lane worker for PF-60-S03 (docs/sprints/current/portfolio-agent-cost-accounting/
pf-60-s03-inspectable-run-and-campaign-totals.md). Read sec-common.md (binding: disposable homes, never the real keychain,
CORBANU_TEST_NO_NATIVE_KEYRING=1 for fresh debug builds, disk hygiene) and acct-audit1.result.md in
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/ first.
Bring S03 up to date and close its remaining gaps, small reviewable PRs, each merged when green
(`gh pr merge N --merge --match-head-commit <sha>`; merge origin/main on conflicts, never force-push):
1. Paperwork: update the S03 sprint record Done list with the ~70 commits 09-20..10-03 (current main SHAs, not pre-rebase),
   move the acct-failure-63 / acct-sweep-64 test+review evidence from .codex-work into qa/, and refresh the plan file
   (docs/plans/active/portfolio-agent-cost-accounting.md) so it reflects reality (worktrees, collection developer-only).
2. Code: no-usage, stale-estimate and current-command regression tests; the unavailable-backend next step and a label that
   distinguishes missing usage from a missing price. Opus 5.5 High review per slice; Linux clippy on the RTX box.
3. One independent read-only Opus review of the 09-20..10-02 work (installed `corbanu exec`, claude-plan).
4. Demo videos per the repo tmux SOP with GLM 5.2 (provider zai) on a disposable home with developer-accounting enabled:
   (a) /cost after a mocked turn, drill into a request and its details, Back/Esc, then an empty conversation showing other
   conversations and the missing-price next step (cost_scope.rs tmux scenario); (b) a past day and a date range grouped by
   day/week on seeded data; (c) three concurrent `corbanu exec` runs finishing cleanly, then /cost on one. Upload to the
   `demos` release and index under qa/demos/index/PF-60-S03.md.
Do NOT archive S03 or mark it accepted: Travis still has to answer decision acct-s03-acceptance-20260917. Report precisely
what is done and what that decision should now be based on.
