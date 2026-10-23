I fixed all six issues in one PR, #422. It merged after every check was green (merge `ee530a0e26`), and issues #414–#419 are closed, each with an evidence comment.

**What changed**

| Issue | Change |
|---|---|
| #414 | When the Claude Plan helper is called for a named account, the account now also goes in its arguments (`--account <name> --enable named_accounts`), not only in an environment variable. A `corbanu` on PATH that predates named accounts rejects that argument. The session then fails once with an error saying the helper is too old, and never falls back to the default account. Before running the helper, Corbanu also checks that the account exists. Claude panes now use the selected account the same way; before, they always used the default. |
| #415 | With `named_accounts` off, `internal-claude-oauth-token` refuses a named account, whether it comes from `--account` or `CORBANU_PROVIDER_ACCOUNT`. |
| #416 | A 401 on a named account names that account and tells you to run `corbanu account add <provider> <name>`. The default credential is no longer marked "needs attention" in `/providers`. |
| #417 | A provider running on an enrolled named account counts as configured, so onboarding no longer asks for a default key. |
| #418 | `corbanu-debug` keeps a `CORBANU_HOME`, `PFTERMINAL_HOME` or `CODEX_HOME` that you set yourself. `CORBANU_DEBUG_HOME` still wins, with a one-line warning. The install wrapper and `docs/authentication.md` are updated. |
| #419 | The "ignored" config warning now shows once (exec and TUI). An unenrolled `auth.command` account no longer sends unauthenticated `GET /models` requests. An unenrolled Claude account no longer starts the helper (it ran 28 times before). `--kind claude-config-dir` must be an existing absolute path. The kind-mismatch error wording is fixed. |

**Gate evidence**
- **Regression tests:** each issue has its own. The focused `just test` suites pass with and without developer-accounting: 2573 tests on the final merged code, plus 1424 TUI tests run earlier. The installer tests pass 23/23.
  - Ten TUI onboarding tmux tests fail on this Mac only because of a too-long socket path. With `TMPDIR=/tmp` they pass, both before and after the fix.
  - Most new tests depend on new code, so the "fail before" check rests on the live before/after runs rather than reruns of the old code.
- **Clippy (`-D warnings`):** clean on macOS and on Linux, with and without developer-accounting. `ambient@100.99.88.49` was reachable after all; it is the x86_64 Ubuntu host "glitch". The PR's Ubuntu clippy job also passed.
- **Live tmux checks:** fresh debug build, disposable homes, `CORBANU_TEST_NO_NATIVE_KEYRING=1`, a real Claude Plan token and GLM 5.3 Flash.
  - **#414 before:** with a pre-PF-84 `corbanu` on PATH, account `fake` got `pong` from the default account.
  - **#414 after:** that old helper is refused with a clear error. With the S02-era or the current helper, `fake` gets a 401 and `real` answers `pong`.
  - **#416 (GLM 5.3 Flash):** the new hint names `fake`, and the default Z.AI key stays "configured".
  - **#417 (GLM 5.3 Flash):** a home with only named accounts goes straight to chat and answers `pong`.
  - **#415, #418 and #419:** before and after both checked.
  - Everything is in `qa/provider-auth/pf-84/acceptance-fixes-20261010/`. A leak scan of the diff and captures against both real keys found nothing.
- **Review (Opus 5.5 High, read-only, installed `corbanu exec`):** round 1 asked for changes.
  - I fixed the Claude-panes gap and the other points it raised.
  - Its other blocking finding, the TUI worker pre-check, was already handled by the S03 PR (#421), now merged.
  - Round 2 on the follow-up: **approve with nits**. That second pass goes over the one-review rule; both rounds and my responses are in `REVIEW.md` there.
- **CI:** the Windows security probe failed twice with an access-denied error in an unrelated sandbox test. A manual run of the same workflow on `main` and the next PR run both passed, so I treated it as flaky.
- **Overlap with S03 (#421):** it touched the same pre-check call. I kept that function's signature so #421 compiled unchanged, merged `main` after #421 landed with no conflicts, and re-ran tests and clippy. I left a note on #421.

**Cleanup:** the worktree, the build target, the scratch homes (including the ones holding real keys) and my files on the Linux host are all removed.

**Still open (low severity, recorded in `REVIEW.md`)**
- If a TUI session is resumed on a recorded account that differs from the one it was launched with, the 401 hint can name the wrong account.
- If the first thread fails to start and the TUI exits, startup config warnings are not shown.
- If someone overrides the Claude Plan auth command, a named account fails safely, but the error suggests `--kind command`, which `corbanu account add` refuses for Claude Plan.
- A Claude pane with an outdated helper shows only the generic "open Providers to recover" message, not the specific cause.