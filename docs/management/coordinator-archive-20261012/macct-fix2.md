You are a bug-fix worker for PF-84 (named accounts per provider). Read
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding; "Linux box" section: Linux work on
glitch, `ssh glitch`) and macct-fix1.result.md in that folder (how the S02 acceptance defects were fixed), then PR #429's
evidence qa/provider-auth/pf-84/independent-acceptance-s03-20261010/ (README.md, REVIEW.md, captures; on main once #429
merges, else from the PR branch) and issues #425, #426, #427 (`gh issue view N`).
Fix, highest severity first:
- #425 (core invariant "a missing account is an error, never the default account"): `--account <other-provider>:<name>`
  for a provider other than the session's is silently accepted and the session runs on the default account. Refuse at thread
  start with the same recovery text as the session-provider case when the account isn't configured; decide (and document in
  --help and docs) what an other-provider prefix means — prefer: it selects that provider's account for workers/spawns on
  that provider and is validated up front; if it can't apply to anything in the session, refuse rather than ignore.
- #426: TUI `--account <missing>` must refuse with recovery text, not loop into default-key onboarding.
- #427: resume messaging for named accounts (and flag-off resume) should name the right account / cause.
Do NOT change D3 behaviour (#428 is awaiting Travis's decision); leave it alone.
Another worker (macct-lane3) is implementing S04 in its own worktree; keep scope to these issues, merge origin/main before
pushing, and note overlap. Regression tests (fail before, pass after where feasible). Gate: focused tests with and without
developer-accounting; Linux clippy -D warnings on glitch; one independent Opus 5.5 High read-only review via installed
`corbanu exec`; live tmux checks with the fresh debug binary in disposable homes (CORBANU_TEST_NO_NATIVE_KEYRING=1) on GLM
5.3 Flash (`-m glm-5.3-flash -c model_provider="zai"`, `provider/zai_api_key`) and Kimi (`provider/kimi_api_key`), reproducing
#425 before/after (use a canary proxy or fake default to prove which account was used). Keys only via
`X="$(env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME ~/.local/bin/corbanu vault auth-helper <label>)" <cmd>`;
never print keys. New worktree off origin/main under /Volumes/CorbanuDrive/Corbanu/worktrees/. PR to main; merge only when
ALL checks are green, never with checks pending (`gh pr merge N --merge --match-head-commit <sha>`; never force-push; never
rm -f). Close the issues with evidence. Remove build targets when done. Report as in sec-common.md.
