## CLI and installation

- Installed Unix launchers preserve an explicitly selected `CORBANU_HOME`,
  `PFTERMINAL_HOME` or `CODEX_HOME` instead of inserting their default home.
  Conflicting home variables produce a warning. The debug launcher also
  preserves caller-selected homes, with explicit debug-home settings taking
  precedence (#408, #422).
