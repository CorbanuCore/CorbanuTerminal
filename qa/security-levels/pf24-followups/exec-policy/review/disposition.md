# Review disposition (Opus 5.5 High, approve with fixes)

| # | Severity | Disposition |
| - | -------- | ----------- |
| 1 | Medium | Deferred to #197. A `.rules` file broken after launch still drops the rule for threads started later. Agent commands can't cause this, because the Corbanu home and the project `.codex` are read-only under the profile. |
| 2 | Medium | Fixed. A `host_executable` entry for any vault program fails the check. Test added. |
| 3 | Low | Fixed for the trusted-project broken file, the untrusted project being ignored, and `host_executable`. Not added: requirements-only forbid and a reload into a broken project cwd. Both use the same `verify_exec_policy` path. |
| 4 | Low | Fixed. Parse errors and read errors get separate messages. |
| 5 | Low | Kept on purpose. A missing rule file reports the file and the loaded-policy result. |
| 6 | Low | Fixed. A TODO in `legacy_core` points to an app-server RPC. |
| 7 | Nit | Fixed. `VAULT_PROGRAMS.into_iter()` and `[*program, …]`. |
| 8 | Nit | Fixed. The launch test now uses `assert_eq!` on line structure. The parse-error text comes from starlark, so a prefix match on the file path is kept. |
| 9 | Nit | No change needed. |
