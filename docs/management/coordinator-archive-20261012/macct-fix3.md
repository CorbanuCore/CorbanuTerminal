You are a bug-fix worker for PF-84 (named accounts per provider). Read
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding; "Linux box" section: Linux work on
glitch, `ssh glitch`, max -j10) and macct-lane3.result.md in that folder (S04, merged as #430), then the S04 independent
acceptance report qa/provider-auth/pf-84/independent-acceptance-s04-20261011/ (on main once PR #465 merges, else on its
branch) and issues #461 and #462 (`gh issue view N`).
Fix, highest severity first:
- #461 (security-relevant): in first-run onboarding's "Add another account", typing `q` in the account-name or masked
  key/token field quits the TUI and the rest of the typed key is echoed to the terminal in clear. Text fields must consume
  all printable keys (no global shortcuts while a text field has focus), across every masked-entry/text field in onboarding
  and /providers (audit them all, not just this one), and nothing typed may reach the shell after exit. Regression tests,
  including a tmux test that types a key containing `q`.
- #462 (pre-existing): /status claims an OpenAI key that doesn't exist; onboarding loops when config already names the
  provider. Fix both or, if one is out of scope, document why and file a follow-up.
Gate: focused tests with and without developer-accounting; Linux clippy -D warnings on glitch; one independent Opus 5.5 High
read-only review via installed `corbanu exec`; tmux functional checks with the fresh debug binary in disposable homes
(CORBANU_TEST_NO_NATIVE_KEYRING=1) reproducing both issues before/after (synthetic keys suffice for #461; for live turns use
GLM 5.3 Flash `-m glm-5.3-flash -c model_provider="zai"` with
`ZAI_API_KEY="$(env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME ~/.local/bin/corbanu vault auth-helper provider/zai_api_key)"`);
short SOP video of #461 fixed (leak-scanned). Never `pkill` broadly; kill only PIDs you started. Another worker (macct-lane4)
is implementing S05; keep scope to these issues and merge origin/main before pushing. New worktree off origin/main under
/Volumes/CorbanuDrive/Corbanu/worktrees/. PR to main; merge only when ALL checks are green, never with checks pending
(`gh pr merge N --merge --match-head-commit <sha>`; never force-push; never rm -f). Close the issues with evidence. Remove
build targets when done. Report as in sec-common.md.
