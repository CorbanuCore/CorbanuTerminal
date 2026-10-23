postmerge-ci is green on main again: run 37945252801 on a202df29c0, the merge of #350, passed every job. It had failed 84 runs in a row, from 9bac4915f3 on Oct 7; the last green was 80869748b2. The second PR, #353 (the red-main alert), is open and not merged yet because its V8 builds are still running.

All failures were in rust-ci-full's Linux Tests shards 1/4 and 2/4. The Windows clippy leg from #317, musl clippy and shards 3/4 and 4/4 passed in every run. I went through the logs of all 84 runs.

| Failing test(s) | First bad commit | Type | Fix |
|---|---|---|---|
| `stable_precomputed_exports_match_schema_fixtures` (53 runs) | 4f09d7af99 (#249 added a field to `TurnContextItem`) | stale fixture | regenerated with `just write-app-server-schema` — #350 |
| `pass_mode_runs_exec_with_aggressive_enforced` (every run with tests) | da75ee8676 (#231) | wrong test setup, Linux only | see below — #350 |
| `unified_exec_formats_large_output_summary` (18 runs, 3 more passed on retry) | random, first seen at 8a8afb2384 (docs-only merge) | real race | see below — #350 |
| about 120 app-server/exec/tui tests in 10 runs; 1–3 app-server plugin/rate-limit tests in many others | random | real bug | see below — #350 |
| `config_schema_matches_fixture` (64 runs) | 9bac4915f3 (#230) | stale fixture | already fixed on main by #338 |
| `pf_23_s02_persistence_files_become_read_only`, `pf_30_s01_source_envelopes…` (7 runs) | 6b1c8b8873 | Linux regression | already fixed on main by #250 |

- **Nested pass-mode test:** #231 made the nested run require the "Aggressive-homes" registry folder whenever it sits inside the workspace. In real use the original Aggressive session creates that folder; the test never did. It now creates it, so the run still checks the folder exists and is read-only. macOS hid the bug because its temp folders sit behind a `/var` symlink, so a path comparison never matched. The test now resolves that path, and without the fix it fails on macOS too.
- **Large-output race:** the command's end event got its output from a listener that subscribes after the process starts and drops chunks when it falls behind. On a busy runner the 1.3 MB output kept its start and end but lost the middle, so the "bytes omitted" marker was missing. The process's own output task now records the output before passing it on. A new test fails without the fix.
- **False "started by an agent command" refusals:** when the Linux sandbox blocks reads of a missing path like `~/.pfterminal`, bubblewrap creates an empty file at that path on the real disk, not just inside the sandbox. I reproduced this in an Ubuntu 24.04 container. If the sandbox is killed, the file stays. The nested-launch check treated that file as an Aggressive session's home, so every `codex-app-server`, `codex-exec` or `corbanu` launch for that user was refused. This would also hit a real Linux user after a sandbox crash. The fix skips any candidate home that isn't a folder; a home whose details can't be read still counts, because that is what a real sandbox block looks like. A new test covers it.

Nothing was skipped or ignored. The test count rose because tests that were refused at launch now actually run.

**Issues filed** (all of these passed on retry, so none caused a red run by itself):
- **#347:** `tmux_corbanu_env_aliases_restart_account_read_and_legacy_daemon_recovery` failed its first try in 19 of 84 runs.
- **#348:** the sandbox creating those placeholder files in the user's home. I added a second symptom found on my branch, a `bwrap: Can't create file at ~/.claude/.credentials.json` failure that passed on rerun.
- **#349:** nine other one-off flaky tests, with run IDs.

**Catching this earlier:**
- **Implemented (#353):** a job that keeps one issue labelled `postmerge-red` open while main is red, comments on it for each failed run and closes it when the newest main commit passes. Runs with a cancelled job are ignored. I split it out because editing `postmerge-ci.yml` makes the V8 check rebuild V8 from source; those four build jobs have been running about 3 hours. Merge with `gh pr merge 353 --merge --match-head-commit 41675ce1e3` once they pass.
- **Proposed only, because it costs PR minutes:** a PR-time check of the schema fixtures when the protocol, config or feature crates change. Both stale fixtures also fail on macOS, so even a macOS-only check would have caught them.

**Gate:**
- **macOS:** all four failing tests pass locally, along with the related app-server-protocol, security-level, nested-launch, unified-exec and TUI tests. The new tests fail with the fixes reverted. `just fmt`, `just fix` and clippy are clean.
- **Linux (RTX box):** not done. The box (100.99.88.49) became unreachable soon after I started a full Linux test shard on it and stayed down all session. It may have been overloaded by that run; I couldn't tell. My leftover `~/corbanu-rtx/ci-health-*` files there are not cleaned up. Linux evidence instead comes from two full-CI runs dispatched on the branch and the green postmerge run.
  - The first dispatch failed one test, `cold_root_resume…` (a 5-second timeout). It doesn't use the code I changed and passes locally.
  - The second dispatch passed after one rerun of shard 1, which hit the `~/.claude` placeholder race noted on #348.
- **Review:** one independent read-only review (Opus 5.5 High via `corbanu exec`) approved with minor findings, saved under `.codex-work/workers-20261002/ci-health-review/`. I fixed:
  - the alert job's handling of cancelled runs, plus possible duplicate or wrongly closed issues;
  - a test that would break on Windows;
  - one more placeholder case the fix missed;
  - a misleading code comment.
  
  I declined two small suggestions and gave the reasons in the PR.

My local worktrees and build folder are removed.