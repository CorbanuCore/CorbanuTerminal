Issue #310 is fixed and closed. PR #313 is merged into main (merge commit `8131beefb2`) with all 30 CI checks passing. Provider API keys like `ZAI_API_KEY` no longer reach commands the model runs, at every security level including Permissive, and Corbanu still uses them to talk to the provider.

**What changed**
- **Where the leak was:** the shell tool already removed a hand-kept list of key names. Long-running exec sessions (`exec_command`) removed nothing, and neither did the exec-server, which rebuilds the environment from its own process. Both now remove the keys.
- **Which names:** the list now comes from the built-in provider table, plus every configured provider's `env_key`. It also covers a few other credentials Corbanu reads (`OPENAI_API_KEY`, `CODEX_API_KEY`, `CODEX_ACCESS_TOKEN`, `AZURE_OPENAI_API_KEY`, `ANTHROPIC_AUTH_TOKEN`, `CLAUDE_CODE_OAUTH_TOKEN`, `AWS_BEARER_TOKEN_BEDROCK`). The old hand list missed `KIMI_API_KEY`, `DEEPSEEK_API_KEY` and `CORBANU_API_KEY`.
- **Shell snapshots:** the review found a second leak. A configured key whose name doesn't look like a secret could come back through the shell snapshot that login-shell commands load first. This happened even when the key was exported from `~/.zshrc` or `~/.bashrc`. The snapshot now leaves these names out.
- **Opt-in:** this builds on the existing `shell_environment_policy`. Naming a key exactly in `include_only` (or the `filters` form) passes Corbanu's value through; a `set` entry gives commands only the value written in config. Wildcards like `*_API_KEY` never pass a key.
- **MCP servers:** unchanged. A stdio server already gets a key only if it's listed in that server's `env_vars` or `env`.
- **Vault and onboarding:** they never put keys into the process environment. A `.env` file in the Corbanu home does, and the new filter covers it.
- **Docs:** new "Shell Environment" section in `docs/config.md`, and a short note in `docs/integrations/zai-glm-52.md`.

**Gate evidence**
- **Focused tests:** 570 passed on macOS. The 121 snapshot, provider-key and environment tests also passed on the RTX box. A first run there without the repo's test wrapper crashed in unrelated tests; through the wrapper all passed.
- **Regression tests (`exec/tests/suite/provider_key_env.rs`):**
  - **Setup:** a mock provider authenticated by its own key variable, with `ZAI_API_KEY`, `OPENROUTER_API_KEY` and `CODEX_API_KEY` also set.
  - **Result:** both tools, with login and non-login shells, report ABSENT for every key, and every provider request still carries the key. An exact `include_only` entry reports PRESENT.
  - **Checked against old code:** the `exec_command` case fails on old main (it reproduces #310), and the login cases fail without the snapshot fix.
- **Linux clippy (`-D warnings`)** on the RTX box for `codex-core`, `codex-model-provider-info` and `codex-exec`: clean.
- **Review (Opus 5.5 High, read-only):** it asked for changes twice, about the snapshot leak, the extra credential names and how differently capitalised `set` entries were handled. All were fixed, and the final verdict was APPROVE.
- **Functional check:** a fresh debug binary in a disposable home (`CORBANU_TEST_NO_NATIVE_KEYRING=1`) ran against real Z.AI with the key supplied only through `corbanu vault auth-helper`. It reported ABSENT with exec sessions on and off, and the key didn't appear in any output.
- **CI:** the macOS credential canary failed once on a known flake (#222, already fixed on main by #309). I merged main into the branch (no force-push) and the rerun was all green.
- **Cleanup:** my worktree, build target, temporary homes and the RTX worktree, build target and logs are removed.

**Still open**
- **Commands not covered:** commands you type yourself with `!` and app-server `command/exec` requests are still unfiltered, since the model doesn't run them. Note that the output of a `!` command does land in the model's context.
- **AWS credentials:** standard AWS credential variables are still passed to commands; only the Bedrock bearer token is removed.
- **Trusted projects:** a trusted project's `.codex/config.toml` can use `include_only` to pass a key through. That's no wider than what such a config can already do by passing a key to an MCP server. The change record and the include/set opt-in decision are written into the PR.
- **PowerShell:** if PowerShell snapshots are ever turned on, they'll need the same name check as the other shells. They're disabled today.
- **Keychain variable (unrelated):** the installed `corbanu vault auth-helper` refuses to run when `CORBANU_TEST_NO_NATIVE_KEYRING=1` is set, which may not be what's intended for the installed build. It doesn't touch the keychain.