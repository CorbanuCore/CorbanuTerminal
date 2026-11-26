## Fixes

Telegram: when a turn ended interrupted or failed and Telegram refused the "Turn interrupted." or "Turn failed." notice (for example with flood control), the chat stayed in `Working`. New messages were queued behind the finished turn and only `/new` or a restart recovered. The turn is now cleared before the notice is sent (#2921).
