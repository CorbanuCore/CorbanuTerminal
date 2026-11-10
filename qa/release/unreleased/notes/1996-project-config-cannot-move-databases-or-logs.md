## Project config can't move Corbanu's databases or logs (D483, #1945)

- **A project's `.codex/config.toml` no longer sets `sqlite_home` or
  `log_dir`.** Corbanu ignores them there and shows a startup warning naming
  them, as it already did for provider URLs, `notify` and `otel`. Before, a
  trusted repository could choose where thread history, memories, state and
  logs were written (into the repository itself, for example), and have
  Corbanu open state databases it shipped.
- **A project's `.codex/config.toml` no longer sets `[debug.config_lockfile]`
  either.** Replaying a config lock replaces the whole config, so a project
  could use one to set `sqlite_home`, `log_dir` or any other setting a project
  can't set (provider URLs, `notify`); it could also choose where lock files
  are written.
- Not changed: `sqlite_home` and `log_dir` in your user config, with `-c`, in
  managed config or requirements, and `CODEX_SQLITE_HOME`, work as before, as
  does `[debug.config_lockfile]` in your user config. If a project of yours
  set any of them, move them to your user config.
