# Worker brief: deflake the Ubuntu tmux smoke suite

You are an Opus 5.5 High worker. Work autonomously; do not ask questions.

## Context
- Repo CorbanuCore/CorbanuTerminal, workflow `.github/workflows/tmux-smoke.yml` runs
  `just test -p codex-tui --test all tmux --retries 0` (70 tests, serial, ~18–37 min on a 4-core runner).
- In the last day, three different tests each failed once and passed on rerun (see
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/flakes.txt`):
  1. `suite::provider_convergence::tmux_fresh_a_then_b_preserves_first_success_across_restart` —
     `timed out waiting for session "pf55-fresh-1" to exit` (95 s).
  2. `suite::provider_convergence::tmux_resumed_main_session_retains_exact_runtime_identity` —
     `Error: No such file or directory (os error 2)` after 25 s.
  3. A pf57 env-alias test — `timed out waiting for session "env-alias-2" to exit`.
  Job logs: use `gh run list -R CorbanuCore/CorbanuTerminal -w "tmux smoke" -L 30` and
  `gh api repos/CorbanuCore/CorbanuTerminal/actions/jobs/<id>/logs` for failed attempts; failed runs upload
  `tmux-artifacts/` (`gh run download`).
- Two of three are "session exit" waits: the shared tmux harness's exit wait is probably too short or races
  with the process actually exiting under load.

## Do
1. Find the shared harness code (search `timed out waiting for session` under `codex-rs/tui/tests/`), read the
   failure artifacts, and determine the real cause of each flake (timeout too tight vs. a real race vs. a product
   bug such as a slow shutdown path). Don't just bump timeouts blindly; if the exit wait should be event-driven or
   needs a longer budget under CI load, justify it from the artifacts.
2. For (2), find which file was missing and why (ordering/race in the test or product).
3. Fix on branch `codex/tmux-deflake-20261004` in a new worktree off `origin/main`. Stress-verify on Linux: the RTX
   box `ssh ambient@100.99.88.49` (fresh clone under `~/tmux-deflake-20261004/`, cleaned up afterwards); run the
   three tests repeatedly (e.g. 20× each) under parallel CPU load, before and after the fix, and record counts.
4. Open a PR to main with root causes and the stress numbers. Report when checks pass; do not merge.

## Never
Never print secrets. `rm -f` is blocked by policy. No force-push to shared branches. Don't weaken what the tests
assert.

## Final report (your last message)
Root cause per flake, fix, before/after stress counts, PR URL and check status.
