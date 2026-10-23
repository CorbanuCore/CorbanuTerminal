# Worker brief: last nightly failure + confirm the scheduled nightly on main is green

You are an Opus 5.5 High worker. Work autonomously; do not ask questions.

## Context
- PR #159 (nightly cold-Bazel fixes) merged to main at ~07:27 UTC 2026-10-05. Its report:
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/nightly-fixes.result.md`.
- After its last dispatched round (run 37255823666), two failures remained:
  1. gnu Bazel `//codex-rs/app-server:app-server-all-wine-exec-test`, test
     `executor_skills::selected_executor_root_exposes_plugin_skill_and_forwards_budget_warning` — failed 3/3 in round
     3 (1/17 in round 2), only under Wine-exec; the normal app-server suite passes it. Possibly related to #160
     (skills catalog source-keyed dedupe, merged between rounds 2 and 3) — unconfirmed.
  2. Windows native lane `rmcp_client::tests::active_time_timeout_pauses_while_elicitation_is_pending` — widened in
     `e40f7a3`, not yet exercised by a nightly round.
- The scheduled `nightly-ci.yml` run on main fires at 09:17 UTC.

## Do
1. Root-cause failure 1. Read the test and what Wine-exec changes (how the executor root / plugin skill paths and
   the budget warning surface under Wine). Check whether #160's source attribution or dedupe interacts with this
   test's expectations (e.g. a catalog now kept/dropped differently, or a warning fragment attributed to a different
   source). Reproduce under Wine if feasible (RTX box `ssh ambient@100.99.88.49`, Linux, fresh clone under
   `~/nightly-final-20261005/`; install wine if needed — clean up afterwards), or by dispatching `bazel.yml` with
   `cold_bazel=true` on a branch. Fix the real cause; don't weaken the test.
2. Branch `codex/nightly-final-20261005` off `origin/main`, open a PR, get its checks green.
3. Watch the 09:17 UTC scheduled nightly on main (`gh run list -R CorbanuCore/CorbanuTerminal -w nightly-ci`).
   Report every failing job/test. Then dispatch nightly on your branch
   (`gh workflow run nightly-ci.yml -R CorbanuCore/CorbanuTerminal --ref <branch>`) to confirm your fix plus the
   rmcp timing fix. Cap: 2 dispatched rounds. Don't merge.

## Never
Never print secrets. `rm -f` is blocked by policy. No force-push. Don't change CI timeouts. Don't touch issue #158's
skipped sandbox test (a separate worker owns it).

## Final report (your last message)
Root cause, fix (commits), PR URL and checks, scheduled-nightly-on-main result, dispatched-round results.
