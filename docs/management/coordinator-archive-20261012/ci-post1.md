You are a CI-health worker. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding)
and the repo skill /Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/home/skills/pushing-ci-changes/SKILL.md.
Problem: the `postmerge-ci` workflow on `main` of CorbanuCore/CorbanuTerminal has FAILED on every one of the last 40 runs
(`gh run list -w postmerge-ci -b main -L 60`). Nobody noticed because PR checks don't run it. Latest example: run
37917695916 on d0544c1c91. Failing jobs there include rust-ci-full Tests shards 1/4 and 2/4 on ubuntu-24.04, with e.g.
`app-server-protocol schema_fixtures_tests::stable_precomputed_exports_match_schema_fixtures` and
`core suite::unified_exec::unified_exec_formats_large_output_summary`.
1. Enumerate EVERY failing job/test across the last several postmerge runs (all platforms/shards, including any Windows
   clippy leg added by #317), and find when each started failing (first bad commit; `gh run list` + logs, bisect locally
   if needed). Classify each: real regression, stale fixture/snapshot (e.g. schema fixtures not regenerated), flaky, or
   environment.
2. Fix them properly: regenerate fixtures with the repo's own generator (don't hand-edit), fix real regressions in code
   (not by weakening tests), deflake flaky tests deterministically, and file issues for anything you can't fix with
   evidence. Never skip/ignore a test to make CI green unless it's genuinely wrong, and then say why in the PR.
3. Propose (in the PR body) the cheapest way to catch this earlier, e.g. a small PR-time job for the schema-fixture
   check, or a scheduled alert when postmerge fails on main; implement it only if cheap.
Gate: the failing tests pass locally (macOS) and on the RTX box (Linux, ssh ambient@100.99.88.49) where applicable; one
independent Opus 5.5 High review via installed `corbanu exec` (read-only); PR(s) to main; merge when green
(`gh pr merge N --merge --match-head-commit <sha>`; never force-push; never rm -f). Then confirm the next postmerge-ci run
on main is green (or list exactly what still fails and why). Other workers are merging accounting (acct-s05) and Windows
(sec-win8, PR #343) changes concurrently; merge origin/main often. Report: each failure, its cause, its fix, the PR numbers,
and the post-fix postmerge run result.
