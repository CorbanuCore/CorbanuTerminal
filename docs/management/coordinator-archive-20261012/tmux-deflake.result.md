I've opened PR #154 and all its checks pass: 29 passed, 9 skipped, 0 failed, including Ubuntu tmux smoke (70 tests) and CodeQL. Not merged. None of the flakes came from a too-short exit timeout.

**PR:** https://github.com/CorbanuCore/CorbanuTerminal/pull/154 (branch `codex/tmux-deflake-20261004`)

## Root causes and fixes

**Flakes 1 and 3 (`pf55-fresh-1` / `env-alias-2` "timed out waiting for session to exit"): a product race.** In all three uploaded failure artifacts the TUI was still running because it had run the wrong command:

| Failed run | What the screen shows | What happened |
|---|---|---|
| 37140229695 | the model picker | `/` arrived alone, Enter picked `/model` |
| 37159432492 | the experimental-features screen | `/ex` arrived, Enter picked `/experimental` |
| 37175536496 | `t/exit` sent as a chat message | `/walle` ran `/wallet`; the late `t` stayed in the input box |

- **Mechanism:** the input box briefly holds back fast-arriving characters (8 ms) to tell typing apart from pasting. Under load, `/` can display and open the command menu while the rest of the command is still held back. Enter then picked from the menu using only the partial text.
- **Fix:** in `ChatComposer::handle_key_event`, when the command menu is open and the key is Enter or Tab, the held characters are inserted before the selection is used.

**Flake 2 (`tmux_resumed_main_session_retains_exact_runtime_identity`, ENOENT): a test race.**
- **Missing file:** something under the test's private home directory's `.tmp/plugins` cache. The TUI's background plugin sync clones into `.tmp/plugins-clone-*` and then swaps `.tmp/plugins` by renaming and deleting directories. The test's secret scan walks that directory while the TUI is still running, and the old walker failed on any file that vanished mid-walk, without naming it.
- **Evidence:** a 25-minute probe on Linux repeated the same walk over live test home directories and caught 99 vanished reads/listings under `.tmp/plugins` and 5 under `.tmp/plugins-clone-*`.
- **Fix:** one shared walker in `support::secret_scan` replaces four copies. A file that disappears mid-walk counts as absent; any other error still fails and now names the path. Every file that exists is still read in full, and each test keeps its rules for which credential files are skipped.

**A fourth race showed up on the PR's own first tmux run.** `claude_auth::tmux_claude_auth_managed_success_cancel_failure_recovery_and_resume` sent `/model` before startup had finished, and the TUI correctly refused it. The test was waiting for the footer, which appears before startup completes. A new `support::chat_ready::session_configured` waits until the header shows a real model instead of `loading`. That check now gates the Claude test and pf55's `wait_chat_ready` (same gap). This only makes the wait stricter; no assertion changed. The rerun passed.

## Before/after (RTX box, fresh clones of main 5ed6686b8 and the branch)

| Check | main | branch |
|---|---|---|
| New unit test `slash_popup_enter_materializes_rapid_input_tail` | fails | passes |
| Real TUI in tmux: `/`, then `exit`+Enter back-to-back, 20 trials | 0/20 exited (20/20 opened the model picker, same as CI) | 20/20 exited |
| Stress: 3 tests ×20, pinned to 4 CPUs with 8 busy loops | 60/60 pass | 60/60 pass |
| Stress: 3 tests ×10, 4 CPUs with 16 busy loops | 30/30 pass | 30/30 pass |

Plain CPU load never reproduced the flakes on either tree; the CI failure rate is too low to surface in 90 runs. The evidence for the fix is the forced tmux timing test and the unit test. I also tried 24 busy loops on 2 CPUs; both trees just starved into nextest's 180 s timeout, so I discarded that setup.

The RTX workspace, its tmux sessions and the temp files are all removed.

**Follow-up, not in this PR:** every TUI launch in these tests runs the plugin sync, which clones from GitHub over the network. Turning it off in the test setup would make the tests self-contained and faster.