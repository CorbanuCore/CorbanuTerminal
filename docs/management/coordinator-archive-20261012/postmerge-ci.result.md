`postmerge-ci` now starts. The fix is in PR #151 and all its checks pass. I haven't merged it.

**Root cause.** This repo's default workflow token is read-only (`default_workflow_permissions: read`), which means it has no `actions` permission. In `v8-canary.yml`, the `build` job asks for `permissions: { contents: read, actions: read }`. A called workflow can't ask for more token permission than the workflow calling it. `postmerge-ci` calls `v8-canary` without a `permissions` block, so it has no `actions` permission to pass on, and GitHub rejects the whole run before any job starts.

**Exact error.** GitHub doesn't give the message text through the API. The run's check suite has no check runs and no annotations, so the only result is `startup_failure`. `actionlint` didn't catch it either, so I found the cause by testing variants. This isn't new since 2026-09-12: all 105 `postmerge-ci` runs in this repo have ended in `startup_failure`. Upstream openai/codex has a broader default token, which is why it only breaks here. `nightly-ci` starts because nothing it calls asks for more than the default permissions.

**Fix.** One change in `postmerge-ci.yml`: the `v8-canary` job now grants `contents: read` and `actions: read`, with a comment explaining why. Nothing else changed and no timeouts were touched.

**Evidence** (from a throwaway branch `debug/postmerge-ci-20261004`, using copies of the workflow triggered by pushes to that branch):

| Variant | Run | Result |
|---|---|---|
| Unchanged copy | 37166466325 | `startup_failure` (same failure as on main) |
| Only the `v8-canary` call | 37166466175 | `startup_failure` |
| Only the `rust-ci-full` call (including its nested workflow) | 37166466301 | Started and jobs ran |
| Full copy with the fix | 37166511082 | Started: `rust-ci-full` jobs were running and `cargo shear` passed; `v8-canary / metadata` passed and its build jobs were skipped as expected |

I cancelled the debug runs once their jobs were executing, to free up runners. The debug branch and its worktree are deleted.

**Tests.**
- `actionlint` reports nothing on the edited `postmerge-ci.yml`.
- `python3 -m unittest discover -s .github/scripts -p 'test_*.py'` ran 53 tests; the only failures are the 4 known `test_setup_telegram` ones on macOS.

**PR:** https://github.com/CorbanuCore/CorbanuTerminal/pull/151 (branch `codex/fix-postmerge-ci-20261004`). 33 checks passed, 8 were skipped and none failed, including the full v8-canary builds; merge state is `CLEAN`. The first `postmerge-ci` run on `main` after merging will confirm the fix in place.