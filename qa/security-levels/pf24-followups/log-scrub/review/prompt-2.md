You are an independent senior security and Rust reviewer (Opus 5.5, high effort). This is a second pass. Review the branch diff against origin/main (`git diff origin/main...HEAD`), focusing on the last commit (`git show HEAD`), which answers your first review (the first-pass findings are summarised here). Do not edit files; read the code and report.

First-pass findings and what changed:
1. Detector missed #196/#179 forms -> added context rules: every value inside `http_headers: Some({…})` / `query_params: Some({…})`, every map value on `spawn_child_async` / `ExecOneOffCommand` lines; quoted `NAME="v"` / `NAME=\"v\"` assignments; more secret name fragments (pass, pwd, mnemonic, seed, private, jwt, dsn); names ending `_id/_ids/_count/tokens/_mode/_kind/_type` are not redacted; `basic` only after `authorization`.
2. Synchronous scan + VACUUM in `StateRuntime::init` -> now a background task (`spawn_log_scrub`) after init; batches capped at 1000 rows and 8 MiB of bodies.
3. Marker could make the pass repeat -> staged `user_version`: 1 after rows+VACUUM, 2 after a non-busy TRUNCATE checkpoint; at 1 only the checkpoint is retried.
4. Regex cost -> ASCII mode (`(?-u)`) and a `RegexSet` prefilter.
6. Truncation by another process -> the file pass stops if the file got shorter.
7. Non-UTF-8 rows -> bodies selected as BLOB, written back lossily.
8. `codex-login.log` in `log_dir` is now scrubbed too.
9. The file scrub thread starts after the subscriber is installed.
10. Tests: secrets asserted present before; WAL checked while the runtime is open; busy-checkpoint test; more forms.
12. DB code moved to `state/src/runtime/log_scrub.rs`; module private, only `scrub_log_file_once` exported; secure_delete removed; Bazel lock unchanged (`bazel mod deps --lockfile_mode=update` made no change).
Not changed: 5 (1 MiB line pieces, documented), the state DB's old dropped `logs` table pages, daemon stderr logs.

Look for remaining correctness or safety problems in the new code (background task lifetime, races with concurrent inserts/pruning/VACUUM, staged marker logic, the batch selection, regex correctness of the new patterns, test flakiness).
