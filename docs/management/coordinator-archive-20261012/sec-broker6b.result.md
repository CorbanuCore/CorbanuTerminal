Both PRs are merged with all checks green: the keychain-isolation fix (#234) and the rest of PF-27-S05 (#237), whose record is archived. The keychain problem wasn't quite what we thought. The vault (and every other user of the OS keychain) already obeyed `CORBANU_TEST_NO_NATIVE_KEYRING`. The gap was that `corbanu` child processes never received it.

**Why the keychain was reached**
- All keychain access goes through one module that refuses it in debug builds when the variable is set; the vault then keeps its key in a file inside the disposable home.
- The variable's name contains `KEY`, so the filters that strip secret-looking variables from what Core passes to agent commands, hooks and MCP servers removed it. `CODEX_HOME` was kept. So a `corbanu` started inside a throwaway profile ran without isolation and could write a vault key into Travis's real keychain.
- In this lane's own vault demo, the isolation held: its key stayed in the throwaway profile's fallback file (the native keychain write was refused, which the logs also show).
- In the same window, the TUI-lane worker (sec-tui5) added and read a test item in the real login keychain and ran its Claude panes against the real Claude login. That may be what caused Travis's prompts.

**#234: keyring isolation reaches child processes (merged)**
- Debug builds now pass the variable to every child: shell and unified-exec commands, hooks and MCP servers. It is re-added even when the user's env policy is `inherit = "none"` or `include_only`. Release builds are unchanged.
- `scripts/demo_video.py` and every demo wrapper now refuse to run without the variable, and refuse release builds.
- The vault credential lookup must use the installed, signed `corbanu`. The new `--vault-helper` option picks it explicitly. The candidate or any build output is refused.
- New tests cover the shell tool and unified exec, MCP, the vault falling back to its file, and the demo script's checks.
- Opus 5.5 High reviewed it three times; rounds 1 and 2 asked for changes, which are fixed, and round 3 approved.
- **For Travis, in Keychain Access (I didn't read or delete anything):** none of these is expected to exist. Any you find is safe to delete.
  - service `codex`, accounts `secrets|a65542c029bf992d`, `secrets|57c3ccc977967732`, `secrets|4c5936ca97d3c20d`, `secrets|7498c47eca809cb7`, `secrets|06b1841283d841c2`, `secrets|d6c188e4c3f9cd22`
  - service `Codex Auth`, account `cli|` plus the same 16 characters
  - Don't delete the `codex` item that belongs to his real home; that one is his vault key. The PR body shows how to work out which it is.
  - The TUI lane's test item `corbanu-sec-tui5-probe` / `probe` was deleted by that worker, according to its log.

**#237: PF-27-S05, model-client auth in the broker (merged, flag `broker_model_auth`, off by default)**
- Once a loaded config turns the flag on, Core stops reading provider keys and stops attaching keys or sign-in tokens itself.
  - Until the broker is running, those requests fail instead of going out with the raw key.
  - Websockets and realtime conversations are switched off under the flag.
- Model requests, web search, image generation and the model catalog all go through the broker. That covers env and vault provider keys, the OpenAI API-key login, `experimental_bearer_token` and ChatGPT access tokens. Command, AWS, header and agent-identity auth are still sent as before.
- Provider keys stored in the vault are now decrypted inside the broker. The first live vault demo failed: the secrets code changed file permissions on every read, which the broker's sandbox forbids. It now skips that when the file is already private.
- Provider keys in the environment are handed to the broker and overwritten in Core's own environment.
- New tests cover:
  - a memory scan showing the raw key isn't left in Core;
  - the non-Unix path refusing;
  - failing closed before the broker starts;
  - unchanged behaviour with the flag off;
  - the vault lock file never being followed through a symlink.

**Gate evidence (#237)**
- **Tests:** 1312 passed on the affected crates. On the related Core test subsets 1099 passed and 2 `suite::client::skills_*` tests failed: they pick up the real `~/.agents/skills` folder, not anything in this change.
- **Linux:** clippy with warnings as errors is clean on the RTX box, and the PF-27-S05 tests pass there.
- **CI:** the macOS credential canary failed once on the push run. The pull-request run passed on the same commit, the same tests passed 5 of 5 locally, and the rerun passed.
- **Videos:** two GLM 5.2 tmux demos in a throwaway profile with keychain isolation on: the environment key handed to the broker, and the vault key read inside the broker. Both are published; the links are in `qa/demos/index/PF-27-S05.md`.
- **Review:** Opus 5.5 High; rounds 3 and 4 asked for changes, which are fixed, and round 5 approved. The reviews are in `qa/security-levels/sprints/PF-27-S05/`.
- **Reviewer setup:** the reviews ran through OpenRouter in a throwaway, keychain-isolated profile rather than the usual Claude-plan route, so nothing touched the real keychain.

**Open, recorded as known limits in the archived PF-27-S05 record**
- **Decision for Travis:** other features still open the encrypted vault inside Core, which decrypts the whole vault file in Core's memory. These are provider status, `/vault`, Task Node, Telegram, the wallet and the campaign tracker. Only the provider key used for requests stays in the broker alone. My recommendation is a follow-up sprint that keeps the vault's list of entries separate from the secret values, or serves that list through the broker, rather than reopening PF-27-S05.
- Removing environment keys happens when a session starts, while other threads are running. Doing it before `main` would need the flag decided at process start.
- Core still holds and refreshes ChatGPT sign-in tokens, and agent-identity registration sends its token directly.
- Requests made before the first session starts fail rather than start a broker. That covers the first model-catalog refresh and `corbanu doctor`.
- `just test` sets the isolation variable, so `auto_store_remains_pinned_across_session_recovery` in rmcp-client now fails there, because it relies on a mocked native keychain that the isolation refuses. It already failed that way before #234.

Worktrees, build folders and the RTX folders are removed. The short result file is `.codex-work/workers-20261002/sec-broker6b.result.md`.