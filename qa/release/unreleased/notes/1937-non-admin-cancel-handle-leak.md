## Windows sandbox

- **Non-admin sandbox: a command cancelled while starting no longer leaks
  handles** (#1909). If a command was stopped while its preparation or start
  was still running, Corbanu kept the command's token open and, once the
  command had started, its process and thread handles, until Corbanu exited.
  The command itself was already ended. These handles are now closed too.
