# PF-84 acceptance fixes (#414–#419), 2026-10-10

Fixes for the defects found by the [independent acceptance run](../independent-acceptance-20261010/).
Each live check ran in tmux on a private socket, in disposable homes, with `CORBANU_TEST_NO_NATIVE_KEYRING=1`.
Builds were macOS arm64 debug:

- **before:** `origin/main` 85e9c43ac4
- **after:** this branch
- **pre:** a pre-PF-84 build (31b2937aa7), used as the outdated `corbanu` on PATH

The parent binaries were named `corbanu-before` and `corbanu-after`, so the Claude helper resolves from PATH. That is the version-skew case.

Real credentials came from the operator vault through the installed signed `corbanu vault auth-helper`. They went only into env or stdin, never argv or output. The [leak scan](scripts/leak.py) found 0 hits in the diff and captures, checking the Claude token and the Z.AI key.

| Issue | Before | After | Evidence |
| --- | --- | --- | --- |
| #414 | Account `fake`, old helper on PATH: `pong` from the **default** account. | The helper gets `--account fake --enable named_accounts` in argv. An old helper rejects it and the turn fails with a Fatal error that names the cause. `fake` with a current helper gets a 401, and `real` answers `pong`. | [414](captures/414-claude-plan-path-skew.txt) |
| #415 | `--disable named_accounts` still prints `fake`'s token. | Exit 1, empty stdout, "named accounts are off". | [415](captures/415-internal-token-flag-off.txt) |
| #416 | The hint says "Open /providers … press r", and `/providers` then marks the default Z.AI key "needs attention". | The hint names account `fake` and `corbanu account add zai fake`. `/providers` still shows the default as configured. Run on GLM 5.3 Flash. | [before](captures/416-before.txt), [after](captures/416-after.txt), [providers before](captures/416-providers-before.txt) / [after](captures/416-providers-after.txt) |
| #417 | A home with only named accounts plus `-c provider_accounts.zai="main"` gets default-key onboarding. | Opens the chat directly, and GLM 5.3 Flash answers `pong`. | [before](captures/417-before.txt), [after](captures/417-after.txt) |
| #418 | `CORBANU_HOME=homeB corbanu-debug` runs on the debug home with no warning. | Runs on homeB. A conflicting debug or stable home gives one warning naming the winner. | [418](captures/418-corbanu-debug-home.txt) |
| #419.1 | `exec` prints the "ignored" warning twice. | Printed once. The TUI shows deferred config warnings after the session header, once. | [419.1](captures/419-1-duplicate-warning.txt) |
| #419.2 | An unenrolled `auth.command` account sends 2 unauthenticated `GET /models`. | 0 requests. | [419.2](captures/419-2-models-request.txt) |
| #419.3 | An unenrolled Claude account spawns the helper 28 times. | 0 spawns; it fails at once as not configured. | [414](captures/414-claude-plan-path-skew.txt), `ghost` rows |
| #419.4/5 | A relative or missing config dir is accepted. The error reads "does not take a claude_oauth_token account". | Both dirs are refused at add time, and resolving one gives a clear error. The message reads "does not take `claude_oauth_token` accounts". | [419.4/5](captures/419-4-5-account-add.txt) |

The scripts are in [`scripts/`](scripts/), and `<scratch>` stands for the scratch directory. The disposable homes holding real keys were deleted after the run.
