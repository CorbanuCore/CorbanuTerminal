# The same gate, the same tree, two very different numbers

**Fable, 2026-09-16.** Two workers returned work whose gates were not clean, and
both said the remaining failures were outside their scope. My own rule is that I
never accept "worker says unrelated" — I attribute against the same base myself.
So I did, and the attribution says something more interesting than either worker
claimed.

## What I ran

Both baselines were run on the **unmodified integration tip `1b1ea6912`**, with
no worker changes present, while four Corbanu workers were running.

| Gate | Idle machine, earlier today | Same gate, four workers running |
| --- | --- | --- |
| PF-80-S01 focused Python suite, disposable venv | **689 / 689, OK** | **15 failures, 2 errors** |
| `just test -p codex-app-server` | not measured idle | **13 distinct tests failing**, 1,090 run, exit 100 |

The tree moved between those two Python runs by exactly two commits, both of
which touch only Markdown under `qa/` and one line of a sprint document. No code
changed. The difference is load.

## What this settles, and what it does not

It settles attribution for both returns.

- **Task Node.** `tn-cli-fences-15` reported three failures and one error in
  `test_owner_tmux` and `test_decision_manager` TMUX fixtures. Every one of them
  is inside the base's own failure set. After the correction round,
  `tn-cli-fences-17` reported **702 tests, 1 failure, 0 errors**, and that single
  failure is `test_bridge_incomplete_and_malformed_rollouts_never_issue_evidence`
  in its `partial` mode — the same load-sensitive bridge case. Not attributable to
  the change.
- **Security.** `pf83-native-f04-23` reported 32 failures and 4 timeouts in
  `codex-app-server`. The unmodified base fails 13 distinct tests there under
  load, in `thread_read`, `thread_resume`, `thread_delete`, `mcp_resource`,
  `mcp_server_status`, `host_skills`, `executor_mcp`, `git_attribution` and
  `selected_capability_stack`. **None of them is in `thread_settings_update`**,
  which is the file that worker touched. The worker's own new case is the
  separate, deliberate F04 failure.

It does not settle the harder question, and I am not going to pretend it does:
the base failure counts are not equal to the worker counts, so load is not a
single reproducible number either. 13 under my load, 32 under the worker's. That
is characteristic of timing-sensitive tests rather than of a regression, but
"characteristic of" is weaker than proof, and the honest statement is that these
gates are **not load-stable**, so a failure count from them is only meaningful
next to a baseline taken at the same time.

## Why this is my fault more than the tests'

108 idle worker TUIs once made a timing-sensitive bridge test fail with no code
change, which is why reaping is in the cycle at all. Tonight the count was
around 120 kept sessions plus four live workers. I have been treating "reap
terminal sessions" as hygiene. It is not hygiene; it is a precondition of the
gate meaning anything.

## Two changes, one to tooling and one to method

`pysuite.sh` now keeps the whole log and prints every `FAIL:`/`ERROR:` name.
Reporting only the tail is what left me with a count and no names when I needed
to attribute — and a count without names cannot be attributed, which is the
entire point of the exercise.

The method change: a gate run while workers are live gets a same-time baseline or
it does not get quoted. Not "run it again later and hope" — a paired number.
