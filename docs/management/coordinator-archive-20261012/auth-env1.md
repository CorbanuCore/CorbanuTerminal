You are a bug-fix worker. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding: keychain
rule, disk hygiene, per-fix gate, secrets handling) first.
Fix GitHub issues #365 and #366 (`gh issue view 365`, `gh issue view 366`) in CorbanuCore/CorbanuTerminal:
- #365: `corbanu exec -c model_provider='"openai"'` with only `OPENAI_API_KEY` set gets 401 at the Responses websocket
  handshake; `CODEX_API_KEY` works. Required end state: the built-in openai provider honours `OPENAI_API_KEY` (documented
  precedence vs CODEX_API_KEY and stored credentials, consistent with the PF-60-S05 defaults table and docs), and when no
  usable key is found the error names the variable(s) to set. Check that the TUI and app-server paths agree with exec.
- #366: in a fresh home with only `CLAUDE_CODE_OAUTH_TOKEN` set, the TUI marks the Claude credential as needing attention
  and Claude panes don't run until the token is saved via masked entry, while exec uses it directly. Required end state:
  the TUI and panes accept the environment token the way exec does, subject to existing authority-binding/security rules
  (read the PF-27 specs/decisions in-repo before changing anything security-relevant; if a rule deliberately forbids it,
  keep the rule, make the UI say clearly what to do, and report the conflict). Also fix the pane label mismatch (a pane
  labelled 'Opus 5 Claude Plan' was served Claude Opus 5.5): labels must match the served model.
Regression tests for each. Gate: focused tests for touched crates with and without `developer-accounting`; Linux clippy
-D warnings (RTX ssh ambient@100.99.88.49, else rely on PR CI and say so); one independent Opus 5.5 High read-only review
via installed `corbanu exec`; functional checks with the freshly built debug binary in disposable homes
(CORBANU_TEST_NO_NATIVE_KEYRING=1): OpenAI via `OPENAI_API_KEY="$(env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME ~/.local/bin/corbanu vault auth-helper openai-api-key)" <cmd>`
(gpt-5.4, one short turn), and Claude via `CLAUDE_CODE_OAUTH_TOKEN="$(... auth-helper claude-plan-test-token)"` driven
through the TUI with the repo's tmux harness (docs/tmuxHarness.md), opening a Claude pane and running one short turn.
Never print key values. New worktree off origin/main under /Volumes/CorbanuDrive/Corbanu/worktrees/. PR to main (one PR or
two); merge only when ALL checks are green, never with checks pending (`gh pr merge N --merge --match-head-commit <sha>`;
never force-push; never rm -f). If CI fails only on infra (Docker/ECR rate limits, runner outages), rerun failed jobs; see
PRs #377/#378 for in-flight CI fixes. Close the issues with evidence. Remove your build target when done. Report as in
sec-common.md.
