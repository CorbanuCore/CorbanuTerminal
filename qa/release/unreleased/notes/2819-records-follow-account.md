## Windows sandbox

- The sandbox's records of possibly planted project folders (`AppData\Local\CorbanuTerminalSandbox`) are now found from the Windows account Corbanu Terminal runs as, not from its `USERPROFILE` environment variable. Before, starting Corbanu Terminal with a changed `USERPROFILE` made it read an empty records folder, so project `.codex`/`.agents` folders and `.git` changes a sandboxed command may have planted loaded without a warning. Normal users see no change: the folder is the same one Windows reports for the account (#2819).
