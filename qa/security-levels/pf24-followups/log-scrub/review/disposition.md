# Review disposition (Opus 5.5 High)

## First pass (request changes)

| # | Severity | Disposition |
| - | -------- | ----------- |
| 1 | High | Fixed. Added context rules for provider maps and env dumps, quoted assignments and more name fragments, with tests. |
| 2 | Medium | Fixed. The pass runs in the background, and batches are capped by rows and bytes. |
| 3 | Medium | Fixed. Staged `user_version` (1 = rows and VACUUM, 2 = WAL truncated); at stage 1 only the checkpoint is retried. |
| 4 | Medium/low | Fixed. ASCII mode and a `RegexSet` prefilter: 55 MB takes 0.2 s in release. |
| 5 | Low | Documented (1 MiB line pieces). |
| 6 | Low | Fixed. The pass stops if the file is shorter than the line being written. |
| 7 | Low | BLOB select fixed; the old-binary limits are documented. |
| 8 | Low | `codex-login.log` is scrubbed too. The state DB's dropped-table pages and daemon stderr logs are documented as not covered. |
| 9 | Low | Fixed. The thread starts after the subscriber is installed. |
| 10 | Low | Fixed. Tests assert the secrets are present beforehand, check the WAL while the runtime is open, cover the busy path, and add more forms. |
| 11 | Nit | Mostly fixed: diagnostic names are skipped and `basic` is matched only after `authorization`. |
| 12 | Nit | Fixed. The code moved to `runtime/log_scrub.rs`, the module is private, and `secure_delete` was removed. The Bazel lock is unchanged (`bazel mod deps --lockfile_mode=update`). The pragma stays a literal because `sqlx` requires literal SQL. |

## Second pass (approve with fixes)

| # | Severity | Disposition |
| - | -------- | ----------- |
| 1 | High | Fixed. Double-quoted, single-quoted and escaped `NAME=` values are matched as a whole, with tests. |
| 2 | Medium | Fixed. Only the named token counters (`input_tokens` and others) are skipped; `refresh_tokens` is redacted. |
| 3 | Medium | Fixed. The task holds only the logs pool, and runs once per database per process. `close()` still waits for an in-flight VACUUM (documented). |
| 4 | Medium | Fixed. A new database is marked done at creation. |
| 5 | Medium/low | The checkpoint uses its own connection with a 100 ms timeout. The VACUUM lock duration is documented. |
| 6 | Low | Fixed. Batches are sized with `estimated_bytes`. |
| 7 | Low | Fixed. The busy test calls the pass directly and asserts `Ok` at stage 1. |
| 8 | Low | Fixed. The check compares sets. |
| 9 | Low | Fixed. A missing log directory writes no marker and raises no error. |
| 10 | Low | Documented in a code comment. |
| 11 | Low | Fixed. Any escaping depth is handled, and so is userinfo without a password. |
| 12 | Nit | Fixed. The doc comment is back on `StateRuntime`. |
