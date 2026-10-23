All three items are merged and both issues are closed with evidence. Each PR merged with every check green, pinned to its head commit, and each got an Opus 5.5 High review that ended at APPROVE.

| PR | Item | Merge commit |
|---|---|---|
| #577 | #548: credential helper output kept out of all logs | `d5c980df3a` |
| #585 | Mid-session account switch: connectors, accounting and model list follow the active account | `a6f30dad0f` |
| #576 | #549: concurrent first launches no longer race to create the databases | `84cd04e5b6` |

**What changed**
- **#548:**
  - **AWS SDK logging:** all of the AWS SDK's own logging is now dropped at every level, in every log sink. Corbanu still reports AWS failures to the user with the helper's output removed.
  - **AWS error text:** the AWS error type now prints only the redacted message, even when logged as debug output.
  - **Third-party auth helpers:** a failing provider auth command's error output stays out of the error text. Corbanu's own internal helpers keep their message, redacted and capped at 500 characters.
- **Account switch:** every provider change now moves the session to that provider's account (named or default), not only model requests:
  - **Connectors** reconnect with the new account.
  - **Plugin installs** use the new account.
  - **Accounting** prices each request under the account that actually served it.
  - **The process model list** follows the account of the user's session; each named account has its own cached list.
  - **The TUI** refreshes the list once core confirms the switch, and says so.
  - **Docs:** the gate's known limits, `docs/provider-accounts.md` and the sprint file are updated.
- **#549:** creating and migrating the databases now runs under one file lock per startup, with one migration retry for older builds that don't take the lock.

**Gate evidence**
- **Regression tests fail before and pass after:**
  - #548: a fake `credential_process` prints a canary to stderr. With the real log sinks installed (trace log file, `/feedback` buffer, logs database and its files on disk), the canary appears in none of them.
  - Account switch: 3 of 4 switch tests fail with the switch disabled. Two later tests (the connector worker across repeated switches, a turn that changes provider) failed on the round-1 code.
  - #549: 10 processes launched at once on a fresh home. On main, 7 of 10 had no state database; now all succeed.
- **Linux (glitch, `-j10`):** clippy `-D warnings` is clean for all three, with and without developer-accounting, and the focused tests pass both ways. The full `codex-core --lib` run has 2 timing failures; `origin/main` fails the same way under load, and they pass when run alone.
- **macOS:** clippy is clean and the focused tests pass.
- **tmux check (disposable home, `CORBANU_TEST_NO_NATIVE_KEYRING=1`):**
  - Named account `work` used the `openai-api-key` vault label; the default path ran on GLM 5.3 Flash. Both keys were fetched inline and never printed.
  - In four runs, switching between OpenAI on `work` and the default account was logged by core before each model-list refresh, and the accounting ledger attributes each request to the account that served it.
  - Leak scans for both real keys: 0 files.
  - Evidence is in `qa/provider-auth/pf-84/s06/switch-follow/`.
- **Video (final code head, leak-scanned):** [pf84-switch-follow](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s06-pf84-switch-follow-40cbb034ec7e-2026-10-12.mp4).
- **Reviews:**
  - #549: 3 rounds.
  - #548: 4 rounds.
  - Account switch: 7 rounds. Rounds 1 and 3 requested changes; the round-3 issue was the model-list cache check not matching how the list is loaded, and is fixed.

**Still open**
- **#578 (new issue):** after switching GLM → OpenAI in the same session, OpenAI rejects every turn with a 404 for a GLM reasoning item left in the history. This change doesn't cause it; I hit it during the tmux check.
- **#586 (new issue):** add the new `pfterminal_db_init.lock` file to the protected-surface list. This needs its own security classification.
- **Remaining account limits (in the gate):**
  - `/status` rate limits before the first reply, and the `/apps` directory, still come from the default login.
  - There is one model list per process; with several sessions on different accounts, the last one to start or switch decides it.
  - A process that starts on a non-OpenAI provider (e.g. Z.AI) keeps that provider's list.
  - Analytics keep the account the session started with.

**Cleanup:** worktrees, Mac build folders, the glitch checkout and build folder, tmux sessions and temp homes are removed. Logs and review notes remain in `.codex-work/pf84-followups-20261012/` and `~/corbanu-rtx/pf84fu-logs/`. I never touched the concurrent S06 acceptance files.