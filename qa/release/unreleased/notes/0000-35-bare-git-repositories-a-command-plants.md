## Bare git repositories a command plants

- **New advice in `docs/sandbox.md`** (#1775). A sandboxed command can make a
  bare repository in any folder it may write to, and git run there or in a
  folder below it later runs what its config names. Corbanu Terminal
  doesn't check for these, on Windows or elsewhere. The guide recommends
  `git config --global safe.bareRepository explicit` (git 2.45 or later)
  and explains what it doesn't cover.
