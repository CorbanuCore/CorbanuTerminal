## Windows sandbox

- **With network access off, git uses only local `file` transports** (D475,
  #1921). Sandboxed commands now get `GIT_ALLOW_PROTOCOL=file`, replacing
  any value they had, so git refuses `https`, `ssh`, `git://` and `ext::`
  remotes before connecting, while clones and fetches from a local folder
  or `file://` URL still work. The setting was misspelled
  `GIT_ALLOW_PROTOCOLS` before, which git ignores; the other no-network
  measures still applied.
