# RETURN — tn-cli-fences-19

Allocation digest: `47cd8bba4682c80638cf0bbd606b22c4d748627a9aecf965814aa478778937be`; claim `332e1f13-7c1c-4c89-8a8d-cd6ab4b643c1`; runtime gpt-6-astra/high.
Base verified: `ee16cd310fc0c7445cc75a91698d3a0ab751b719`; frozen brief SHA-256 verified with `shasum -a 256`: `e92f0bf1fe183be2ae1b7783b52d28c03a5123e604e9cfdcd9be3792cbf3d5ea`.
Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/tasknode-cli-fences-20260916`.
Class: bounded correction within PF-80-S01. Product heading **Internal delivery control — TO BUILD**: “Task Node receives only explicitly mapped, supported progress”. Existing plan `initiative-delivery-control.md` is active; sprint PF-80-S01 is in_progress. Manager retains shared coordinate reconciliation and qualification gates.

## Corrections
- Distinct `tasknode.QueueLockTimeout(ValueError)` is raised by `locked` at the existing two-second CLI deadline. Explicit catches: `tasknode.main`; `tick.refresh` around enqueue and around flush; `tick.tick`; `tick.__main__`.
- Tick publishes the actual blocker without counting a valid mapped report as rejected, preserves it alongside invalid-record diagnostics, skips flush after enqueue contention, records unhealthy status, and exits **2**. Exact stderr: `Task Node refresh refused: another Task Node operation is holding the queue; lock path: <state>/.outbox.lock` plus newline. Task Node CLI retains exit **2** and the prefix `Task Node command refused:`.
- Corrected `test_tick_cli_lock_wait_is_bounded_and_recovers` now checks real held-flock expiry, exact refusal/path, exit 2, rejected_runs 0, matching health/writeback errors, unchanged queue/index, and recovery after release. New `test_tick_cli_flush_lock_expiry_is_not_credential_recovery` covers actual flush acquisition with synthetic credentials, no transport, the same refusal/exit and post-release recovery. Child finally blocks assert CLI_CONTEXT resets on failure; existing library-blocking tests remain.
- The same tick entry point used by activate.py's service now reports exit 2 to that service. This is intentional, as required by the frozen fence.
- Tick counts through `delivery_status`, retaining the invalid fallback and uncertainty wording. New `test_pending_record_with_crash_intent_publishes_uncertain` proves persisted pending plus intent/no result publishes uncertain with reconciliation/no-auto-retry words and leaves existing bytes unchanged.
- `delivery_status` treats unreadable/malformed result receipts as uncertain. New `test_cli_preview_with_damaged_result_retains_payload_and_uncertainty` checks truncated JSON, invalid UTF-8 and non-object receipts through both prepare/preview CLI subprocesses: exit 0, immutable payload visible, worded reconciliation/no-auto-retry reason, no authorization/network writes or file changes.
- Retry preserves existing `error` wording and attempts for uncertain records; blocked behavior remains unchanged. New `test_cli_uncertain_retry_preserves_recorded_explanation` checks the exact words, pending/3 and unchanged event. All previous tests retained (30 named Task Node methods and 3 tick methods); four new methods.

## Validation
Read `docs/development/test-isolation.md` before tests. Disposable root R: `qa/initiative-control/pf-80-s01/.tn-cli-fences-19.zGgD9w` under this worktree; fresh venv, removed after evidence preservation.
Created using `env -i PATH=/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin HOME=<absolute-R> TMPDIR=/private/tmp PYTHONDONTWRITEBYTECODE=1 /opt/homebrew/bin/python3 -m venv <absolute-R>/venv`; installed under the same environment with `<absolute-R>/venv/bin/python -m pip install --no-deps -r scripts/initiative_control/requirements.txt`.
Installed application dependencies exactly: markdown-it-py==3.0.0, mdurl==0.1.2, slack-sdk==3.44.1; additional bootstrap package pip==26.0.1 only.
Test prefix: the same empty environment plus `CODEX_HOME=<absolute-R>/profile CORBANU_HOME=<absolute-R>/profile PFTERMINAL_HOME=<absolute-R>/profile CORBANU_TEST_DISABLE_NATIVE_KEYRING=1 PYTHONPATH=<worktree>/scripts/initiative_control <absolute-R>/venv/bin/python`.
Initial `-m unittest test_tasknode test_tick test_preparation test_control -v`: 96 tests, one new-test error, 25.513s, exit 1. The test wrongly assumed a fresh crash record already has an error field; corrected to a separate fixture with retained wording. Raw compressed log: `cli-fences-19-focused.log.gz`; uncompressed SHA-256 `3bc5b8de486fc95ac73f36f500572ddb8372bc1650ee68cf844876825fa37004`.
Final `-m unittest discover -s scripts/initiative_control -p 'test_*.py' -v`: **706 tests, 3 failures, 1 error**, 471.526s, exit 1. All **97 affected tests passed**. Raw compressed log: `cli-fences-19-suite-final.log.gz`; uncompressed SHA-256 `301419a5d097480931bdab252a719ce36bd303dfad2b22c3b98332c6e9ebd140`.
Remaining failure/error names:
- `test_decision_manager.ManagerTests.test_tmux_handoff_end_to_end_mixed_transport_refusals_and_one_unlock`
- `test_decision_manager.ManagerTests.test_tmux_ingress_during_ack_preserves_native_final_fence`
- `test_owner_tmux.TmuxTests.test_bridge_incomplete_and_malformed_rollouts_never_issue_evidence (mode='partial')`
- `test_owner_tmux.TmuxTests.test_bridge_incomplete_and_malformed_rollouts_never_issue_evidence (mode='malformed')`

Plan/sprint checkers passed (active 3/3; current 115, archived 127); diff whitespace check passed. Changes: tests **91 added / 6 deleted = 97**; implementation **33 added / 7 deleted = 40**; receipt **31 added**; total non-test text **71 changed**, below 90. Two binary gzip logs preserve raw attempts; generated environment removed. Only authorized paths changed.
No brief finding was contradicted. Fresh crash records need not contain an error field; preservation applies when words already exist. No live profile, credential prompt, real credential read, live posting, push, Rust test or workspace formatter. This worker return is implementation evidence, not independent functional/TUI acceptance, human-test readiness or release qualification; those gates remain manager-owned.
