Yes: a worker can run on a different Claude Plan account if it uses its own Corbanu home. I tested this live with the installed corbanu 0.1.48. One catch: workers must not be started through the normal `corbanu` command (`~/.local/bin/corbanu`). It quietly switches back to your main home, so the worker would use the coordinator's account.

**Why the normal command breaks it:** `~/.local/bin/corbanu` is a wrapper script. It loads `activate.sh`, which sets the home variables back to the real home. Passing a different home to it still gave me the real home's token fingerprint. It also isn't enough to set only `CODEX_HOME`: the worker shells inherit `CORBANU_HOME`, which takes priority.

**What the code shows:**
- **Vault key:** the keychain entry is separate for each home. Its name is built from the home's path (service `codex`, account `secrets|<hash of the home path>`).
- **Claude Plan selection:** the chosen source and the saved token are stored encrypted in that home's own vault.
- **Token lookup:** `internal-claude-oauth-token` reads only the current home. The one exception is a home with no selection yet, where a `CLAUDE_CODE_OAUTH_TOKEN` environment variable wins.
- **Auth helper:** when a request needs the token, corbanu starts a helper process. That helper finds the real binary, not the wrapper, so it stays on the worker's home.
- **"Claude Code login" source:** this follows `CLAUDE_CONFIG_DIR`, not the Corbanu home. If a worker has a different `CLAUDE_CONFIG_DIR`, it refuses rather than using the wrong account. I didn't test this one live.

**Live test (throwaway home, cleaned up):**
- I ran the real binary directly with keychain access switched off, so the throwaway home's vault key went to a file inside that home. **No keychain entries were created, removed or probed.**
- I stored a clearly fake token through the onboarding screen. `corbanu exec` on that home then failed with `401 … OAuth access token is invalid`, which proves it used the second home's token.
- With your real home, `exec` replied `pong`. Its token fingerprint, vault file (size, timestamp, hash) and selection file were the same before and after.
- The throwaway folder has been removed.

**Setting up the second account:**
1. Run `claude setup-token` while your browser is signed into the second account, and copy the token.
2. Create a small wrapper, `corbanu-acct2`. It loads `activate.sh`, then sets `CORBANU_HOME` and `CODEX_HOME` to a new `home-acct2` folder, then runs the real binary. The exact script is in the report.
3. Run `corbanu-acct2`. In onboarding or `/providers`, choose **Anthropic Claude Account** and paste the token. This creates one new keychain entry for that home, made by the same installed binary, so it should behave like your current one with no prompts.
4. Start workers in tmux with `corbanu-acct2 exec …` instead of `corbanu`. Anything those workers start also stays on the second account. Agents the coordinator spawns inside its own process stay on your main account.

**Caveats:**
- **Other saved keys:** each home has its own vault. Workers on the second account won't see your other stored credentials, such as the ZAI key for GLM tmux tests or Task Node. You'd either add them to the new home or have workers call `~/.local/bin/corbanu vault auth-helper` explicitly.
- **Usage tracking:** sessions, usage and rate-limit status are kept per home, so the coordinator's views won't include these workers.
- **Caches:** Anthropic prompt caching is per account, so the two accounts won't share cache hits. Build and temp caches are still shared, which is fine.

The report also covers each other sign-in type (API keys, auth commands, Claude Code login, OpenAI login, AWS) and what a general "named accounts per provider" feature would need to change. Only Claude Plan was tested live. Minor UI issue: after setup, the provider list showed two rows, "Anthropic Claude Account" and "Claude Account", both marked configured.

Full details with file:line references are in `.codex-work/workers-20261002/multiacct1.result.md`.