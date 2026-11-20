## Upstream Codex rust-v0.167.0

- When Corbanu Terminal cannot start its shared background server on its own
  (for example when you built it from source), it now opens the session
  without the server and says so in one line, instead of exiting and asking
  you to rerun with `--no-daemon`. `corbanu agents` and an explicit `--remote`
  to the background server still report the error.
