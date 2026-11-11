## Windows sandbox

- **The planted-folder watch lists only the folders it checked** (#1909,
  both sandboxes). The watch now opens each folder below the writable
  folders, and each folder and file inside a nested `.codex` or `.agents`
  folder, through the folder it was found in, following no link. Before, a
  process a command left running could swap a folder for a junction or
  symbolic link after the check, and Corbanu would list what it points to,
  which could be a network share. Such a folder is now recorded as
  unreadable, so it is flagged.
