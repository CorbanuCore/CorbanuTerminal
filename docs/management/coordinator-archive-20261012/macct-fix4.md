You are a bug-fix worker for PF-84 (named accounts per provider). Read
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding; Linux work on glitch, `ssh glitch`,
max -j10) and macct-lane4.result.md in that folder (S05, merged as #469), then the S05 independent acceptance report
qa/provider-auth/pf-84/independent-acceptance-s05-20261011/ (on main once PR #483 merges, else its branch) and issues #474,
#475, #476, #480, #481, #482 (`gh issue view N`).
Fix, highest severity first, one PR per issue or small group:
- #482 (privacy): the default-account throttle key is an unsalted sha256 prefix of the API key (a cross-machine
  correlator); use the same per-home salted fingerprint as named accounts, with migration of existing throttle rows.
- #476 (medium): 429 cooldown and request lease are never recorded (any provider, exec and TUI; also pre-S05). Find out
  why the throttle never records, fix it, and prove per-account throttling with a mock 429 server.
- #474: the first named-account request on a pre-S05 vault rewrites the vault, contradicting the gate's claim — either
  avoid the write or correct the claim with a reasoned decision (prefer not touching the vault).
- #475: a 401 on a fake-key account should show in /cost as a failed request with no charge, not "usage incomplete".
- #480: `corbanu exec` on a rejected named account must name the account and its recovery (as docs say).
- #481: spawn wait must report a child that failed with 401 as failed, not "Completed".
Regression tests for each. Gate: focused tests with and without developer-accounting; Linux clippy -D warnings on glitch;
one independent Opus 5.5 High read-only review via installed `corbanu exec`; tmux functional checks with the fresh debug
binary in disposable homes (CORBANU_TEST_NO_NATIVE_KEYRING=1), GLM 5.3 Flash
(`-m glm-5.3-flash -c model_provider="zai"`,
`ZAI_API_KEY="$(env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME ~/.local/bin/corbanu vault auth-helper provider/zai_api_key)"`)
with a fake-key named account for #475/#480/#481. Never pkill broadly. Another worker (bugs-466) works on #466-#468 and
macct-fix3 on #461/#462; stay in scope, merge origin/main before pushing. New worktrees off origin/main under
/Volumes/CorbanuDrive/Corbanu/worktrees/. Merge only when ALL checks are green, never with checks pending
(`gh pr merge N --merge --match-head-commit <sha>`; never force-push; never rm -f). Close issues with evidence. Remove build
targets when done. Report as in sec-common.md.
