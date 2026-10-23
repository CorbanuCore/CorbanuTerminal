You are a security bug-fix worker. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md first
(binding: keychain rule, disk hygiene, per-fix gate, report format).

Fix GitHub issue #310 in CorbanuCore/CorbanuTerminal (`gh issue view 310`): a provider API key supplied through its
environment variable (e.g. ZAI_API_KEY) is inherited by the model's shell/tool commands, so any command the model runs
can read it.

Required end state:
- Provider credential variables are excluded from the environment of model-run commands (shell, unified exec, MCP stdio
  servers launched for tools unless the user explicitly passes the variable to that server, subagent/tool children) by
  default at every security level, including permissive. At minimum: every configured or built-in provider's `env_key`,
  plus well-known provider key names (OPENAI_API_KEY, ANTHROPIC_API_KEY, ZAI_API_KEY, OPENROUTER_API_KEY, etc. — find
  the list from the built-in provider table, don't hand-maintain a second one if avoidable).
- The provider client itself must still read the variable (corbanu keeps working with ZAI_API_KEY set).
- An explicit user opt-in to pass a specific variable through (e.g. existing `shell_environment_policy` include/set)
  still works and is documented; check how upstream `shell_environment_policy` / default excludes already behave and
  build on that instead of adding a parallel mechanism.
- Check whether the vault / onboarding path ever puts a key into the process environment; if so, cover it too.
- Regression tests: a tool command run with ZAI_API_KEY (and one other provider key) set in corbanu's environment
  reports ABSENT; the provider request still authenticates (mock server).
- Update the user-facing docs (docs/integrations/zai-glm-52.md and the shell environment docs) in plain language.

Gate: focused tests, Linux clippy on the RTX box (with -D warnings), one independent Opus 5.5 High review via
`corbanu exec` (installed corbanu, read-only), then a functional check with the freshly built debug binary in a
disposable home (CORBANU_TEST_NO_NATIVE_KEYRING=1): `corbanu exec -s read-only` asking the model to print PRESENT/ABSENT
for $ZAI_API_KEY (never the value), key passed only as `ZAI_API_KEY="$(corbanu vault auth-helper provider/zai_api_key)"`
on the consuming command. PR to main, merge when green with `gh pr merge N --merge --match-head-commit <sha>`, close
#310. Never print secrets, never `rm -f`, never force-push. Work in a new worktree under
/Volumes/CorbanuDrive/Corbanu/worktrees/ and remove your build target when done. Report as in sec-common.md.
