You are a product fix worker. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding)
and auth-env1.result.md in that folder (PR #381, which made the built-in openai provider honour OPENAI_API_KEY as the
last-resort credential after CODEX_API_KEY, session sign-ins and saved sign-ins/keys).
Travis decided on 2026-10-10: when Corbanu is actually using OPENAI_API_KEY from the environment (e.g. after logout, or a
fresh home that skipped onboarding because the variable is set), show a VISIBLE notice. Required end state:
- TUI: a clear, non-blocking notice the first time in a session a request is authorised by the OPENAI_API_KEY fallback
  (and again if the active credential switches to it mid-session, e.g. right after /logout), e.g. "Using OPENAI_API_KEY
  from your environment — billed per use to that API key. Sign in or unset it to change." Also show the source in
  /status and keep /providers' "from the environment" label. Don't nag on every turn.
- exec: one stderr line at start when the fallback is the credential in use (not in --json output streams' payload in a
  way that breaks parsers; follow existing exec warning conventions). app-server: expose it the way other auth-source
  warnings are exposed, if such a channel exists; otherwise document the gap.
- No notice when a saved sign-in, CODEX_API_KEY or a session token is what's actually used, and never for non-OpenAI
  providers.
Regression tests (snapshot tests for the TUI notice per repo conventions; exec stderr test). Gate per sec-common: focused
tests with and without developer-accounting; Linux clippy -D warnings on RTX (ssh ambient@100.99.88.49); one independent
Opus 5.5 High read-only review via installed `corbanu exec`; a tmux functional check with the fresh debug binary in a
disposable home (CORBANU_TEST_NO_NATIVE_KEYRING=1) using
`OPENAI_API_KEY="$(env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME ~/.local/bin/corbanu vault auth-helper openai-api-key)"`
(gpt-5.4, one short turn: notice shown; then with a saved key: no notice), plus a short SOP demo video of the notice
(qa/demos/README.md; leak-scan before publishing). Never print the key. New worktree off origin/main under
/Volumes/CorbanuDrive/Corbanu/worktrees/. PR to main; merge only when ALL checks are green, never with checks pending
(`gh pr merge N --merge --match-head-commit <sha>`; never force-push; never rm -f). Remove your build target when done.
Report as in sec-common.md.
