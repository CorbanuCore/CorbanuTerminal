# PFTerminal 0.1.14

## Fixed

- Incoming hierarchy reports now remain queued until the active provider response reaches its
  terminal event. They no longer interrupt Claude after a signed thinking or commentary block and
  leave a partial assistant response in durable history.
- Existing Claude sessions affected by that partial-history failure now perform one bounded,
  protocol-specific recovery attempt. The retry omits only the incomplete latest assistant
  response and its matching tool results while preserving newer user input and unrelated history.

## Qualification status

- Two crash-shaped integration tests cover hierarchy mail arriving after reasoning and commentary;
  both passed and verify that the complete provider response precedes the follow-up request.
- Three Anthropic history-recovery tests passed, including preservation of valid history and
  rejection of unrelated provider errors. Scoped Clippy and a fresh Linux debug build also passed.
- The complete `codex-core` run was not green in this workspace: 2,795 passed, 145 failed, and 14
  were skipped. The failures were outside this patch's focused test paths and included missing test
  helper binaries, stale model-catalog expectations, and command timeouts. This release does not
  claim a fully green repository suite.

Previous release: 0.1.13.

The changelog can be found on the [releases page](https://github.com/agtico/PfTerminal/releases).
