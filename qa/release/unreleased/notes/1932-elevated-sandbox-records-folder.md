## Windows sandbox: elevated commands can't edit the planted-folder records (D453, #1896)

- **A command in the default (elevated) Windows sandbox can no longer change
  `%LOCALAPPDATA%\CorbanuTerminalSandbox`** when one of its writable folders
  holds it (for example, when your whole user folder or `AppData` is
  writable). That folder keeps the records of which `.git`, `.codex` and
  `.agents` folders a command may have planted, and which of them you
  confirmed. Before, such a command could edit those records, for example to
  mark a folder it planted as confirmed. The non-admin sandbox already
  denied this.
- The elevated setup now denies those commands writing, deleting or renaming
  anything there, making the folder first if it doesn't exist yet. Your own
  programs, and Corbanu Terminal itself, are not affected. Writable folders
  that don't hold it are unchanged.
