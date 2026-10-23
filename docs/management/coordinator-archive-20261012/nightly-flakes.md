# Worker brief: nightly lane — remaining TUI/login failures

You are an Opus 5.5 High worker. Work autonomously; do not ask questions.

## Context
- The nightly cold-Bazel lane (`nightly-ci.yml`) is down to one failing target per Bazel test job, but it is a
  different target each run, which smells like flakiness/timing under the 4-core runners:
  - Run 37289848993 (branch `codex/nightly-final-20261005`, which is now merged to main as #162):
    - gnu: `//codex-rs/tui:tui-unit-tests` FAILED in 5 of 12 shards. Failing tests seen in the log include
      `app::tests::embedded_provider_key_save_is_visible_to_one_bulk_status_refresh`,
      `chatwidget::claude_code_login::tests::claude_cli_owns_token_exchange_and_status_verification`,
      `app::tests::safety_buffering::safety_retry_*` (check exactly which belong to which job).
    - native Windows: `//codex-rs/login:login-unit-tests` FAILED in 421.6s (timeout?), with
      `auth::manager::tests::revision_tracked_external_bearer_*` and
      `unauthorized_recovery_uses_external_refresh_for_bearer_manager` in the log.
    - musl was still running; check its result.
  - Previous rounds: run 37255823666 (gnu: app-server wine executor_skills — now fixed by #162; Windows:
    rmcp-client timing — fixed in #159), run 37238248300.
  - Logs: `gh api repos/CorbanuCore/CorbanuTerminal/actions/jobs/<job-id>/logs`; job ids from
    `gh api "repos/CorbanuCore/CorbanuTerminal/actions/runs/<run>/jobs?per_page=100"`.
- Previous workers' reports: `.codex-work/workers-20261002/nightly-fixes.result.md`, `ci-bazel-test2.result.md`.
- The scheduled nightly on main fires ~09:17 UTC daily (GitHub often delays it by hours).

## Do
1. For each failing test, find the real cause (shared process state, timing assumptions, test ordering under Bazel
   sharding, slow key derivation in unoptimized builds, etc.). Fix the root cause or give the target an appropriate
   timeout/size following repo conventions (`unit_test_timeout`, shard counts) — never weaken assertions.
2. Branch `codex/nightly-flakes-20261005` off `origin/main`; PR to main; checks green.
3. Validate with dispatched nightly runs on your branch
   (`gh workflow run nightly-ci.yml -R CorbanuCore/CorbanuTerminal --ref <branch>`). Cap: 2 rounds. Don't merge.
4. If the provider errors out, leave a short progress note in
   `/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/nightly-flakes.progress.md` as you go.

## Never
Never print secrets. `rm -f` is blocked by policy. No force-push. Don't change CI timeouts in workflows.

## Final report (your last message)
Failure → cause → fix (commit) → verification; PR URL; dispatched-round results.
