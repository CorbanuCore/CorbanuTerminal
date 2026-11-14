## Windows sandbox

- **With network access off, git uses only local `file` transports** (D475,
  #1921). Sandboxed commands now get `GIT_ALLOW_PROTOCOL=file`, replacing
  any value they had, so git refuses `https`, `ssh`, `git://` and `ext::`
  remotes before connecting, while clones and fetches from a local folder
  or `file://` URL still work. The setting was misspelled
  `GIT_ALLOW_PROTOCOLS` before, which git ignores; the other no-network
  measures still applied.
- **Local-file submodules remain allowed** (D550 (b), #2155). The same
  allow-list permits recursive clones and submodule updates to copy local
  repository objects from files the sandbox can already read. Adding
  `protocol.file.allow=user` does not narrow it: git checks the allow-list
  before config (tested with Git 2.54.0.windows.1). This accepted limit keeps
  direct local clone, fetch and push available; #2156's alternative would
  refuse them too. See [Windows: git with network access off](https://github.com/CorbanuCore/CorbanuTerminal/blob/main/docs/sandbox.md#windows-git-with-network-access-off).
