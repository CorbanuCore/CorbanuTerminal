## Slash commands typed while the session starts

- **A slash command entered during startup always waits for it** (D376,
  #1748). While the header still reads `model: loading`, a command such as
  `/model` is queued and runs once startup completes, like an ordinary prompt.
  Before, it was queued only when its keys arrived in one quick burst; typed
  a little more slowly (so the command list opened before Enter), it ran at
  once and was refused with "Model selection is disabled until startup
  completes."
- **`/quit` and `/exit` don't wait** (D390, #1833). Typed during startup,
  they exit at once instead of joining the queue, so a startup that hangs can
  still be left without Ctrl+C. Every other command still waits.
