## Security

- Update sqlx to 0.9.1 and the bundled SQLite to 3.54.1 (libsqlite3-sys 0.39.1) for RUSTSEC-2026-0459: SQLite before 3.54.0 could return uninitialized page-cache memory when opening a database together with a crafted `-wal` file of a different page size. The time-limited exception for this advisory is removed, and codex-state now refuses to build against an older bundled SQLite.
