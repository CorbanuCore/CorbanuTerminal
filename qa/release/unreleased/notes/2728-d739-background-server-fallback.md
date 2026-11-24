## Upstream Codex rust-v0.167.0

- When Corbanu Terminal cannot start its shared background server on its own
  (for example when you built it from source), or cannot connect to the server
  it just started, it now opens the session without the server and says so in
  one line, instead of exiting or asking you to rerun with `--no-daemon`.
  `corbanu agents` and an explicit `--remote` to the background server still
  report the error.
