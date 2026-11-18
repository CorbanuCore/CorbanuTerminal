## Windows sandbox

- When the non-admin sandbox can't give its records folder
  (`%LOCALAPPDATA%\CorbanuTerminalSandbox`) its private integrity label, the
  refusal now names the Windows error and what the folder needs (Full control,
  including `WRITE_OWNER`, which a user's own `AppData\Local` grants), instead
  of only "label … Medium". Commands are still refused in that case (#2584).
