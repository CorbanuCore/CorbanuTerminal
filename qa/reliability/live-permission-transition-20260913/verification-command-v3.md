# PF83 cycle2 / v3 verification

Same worktree/base and Rust1.95 private cache as verification-command-v2.md.
Only source change relative to v2: app_server_session.rs supported-endpoint
guard before server-permission ownership marking plus one native regression.
No other product source change. Formatter ran before tests (fmt-v3-1.log);
known initially clean baseline-only formatter changes preserved and selectively
reverse-applied from fmt-v3-1-baseline-churn.patch. Diff-check passed.

Targeted run: same environment and just test -p codex-tui --locked --offline
--test-threads 1 -E 'test(permission_confirmation_unsupported_keeps_turn_permission_ownership)'.
tui-v3-attempt-1.log:1passed,4105skipped, nextestf2273616-06d8-4333-ab29-3fc4b57e66b4.

Combined final run: EXACT command from verification-command-v2.md, output
focused-v3-final-1.log. It includes the new regression via test(permission_confirmation).
Read actual final result; no pass is inferred from launch. freeze-v3.py generates
separate v3 source-only artifacts only after353passed and checks v1/v2 hashes,
scope, size and single-file delta. It does not commit, stage or modify source.

Scope check at dispatch:34paths,639non-test+1043test=1682 within750/1750.
Plans3active/sprints116current126archived checks passed. Fresh Fable review and
independent functional execution/evidence remain required. The nonblocking
transcript duplication follow-up is recorded in fable-review-02-disposition.md.
No install or live session modification; no old artifact overwritten.
