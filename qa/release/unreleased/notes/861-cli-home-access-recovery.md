## CLI and installation

- `corbanu exec` can wait for access to a temporarily unreadable home to
  return, with bounded backoff and diagnostics. `--home-wait-secs` controls
  the wait window (default 3600 seconds); `0` fails immediately (#861).
