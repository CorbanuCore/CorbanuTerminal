## Moderate and Aggressive: no git `ext::` transport after untrusted content

- **Commands run after untrusted content get git's `protocol.ext.allow=never`**
  (D447, #1876, #1910). It is set in the command's environment
  (`GIT_CONFIG_COUNT`, `GIT_CONFIG_KEY_<n>`, `GIT_CONFIG_VALUE_<n>`), after any
  entries you set there yourself, and `ext` is dropped from a
  `GIT_ALLOW_PROTOCOL` list. It outranks every git config file, so an
  `ext::` remote stored in `.git/config` or `.gitmodules` can't run its
  command even where the repository's own config allows `ext`. It is an
  extra layer under the command checks, and needs git 2.31 or later. It
  doesn't stop `git remote-ext` or `--upload-pack`, which run a program
  without the `ext` transport (the checks from #1897 ask for those), nor
  config git reads after it: `git -c protocol.ext.allow=...` or
  `--config-env` on the line, or `GIT_CONFIG_PARAMETERS`. A
  `GIT_ALLOW_PROTOCOL` list that held only `ext` is left empty, which
  allows no git transport. A command session started before the untrusted
  content keeps its environment. Git variables that only your shell's
  startup files set (and so only the shell snapshot restores) aren't seen:
  a `GIT_ALLOW_PROTOCOL` set there keeps `ext`, and the entry may take the
  place of a `GIT_CONFIG_KEY_<n>` set there.
