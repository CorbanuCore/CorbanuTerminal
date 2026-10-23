`codex-exec` `suite::trace_log_credentials::http_turn_keeps_the_key_out_of_trace_logs` (#380) started failing on postmerge-ci (ubuntu, shard 1) after #469 merged:
- run 38141373184 (#469): failed once, passed on retry (flaky);
- run 38142535540 (#470): failed on both tries.

Each time, `assert_key_not_logged` walks the exec home after `codex-exec` has exited, and reading one listed file fails with `NotFound`. Some file in the home was removed after the process exited. That points to something that outlives `corbanu exec`, a child process or detached writer, and still writes to the home. It could also be a temp file renamed during shutdown that the walk caught. Before #469 the test passed in every run.

It doesn't reproduce on glitch: 40/40 stress runs of the test, and 3/3 runs of the whole `codex-exec` suite.

PR #471 makes the panic name the file. The next CI failure will show which file it is, for example a SQLite `-wal`/`-journal`, an accounting store file, or a `tmp` entry. We shouldn't make the test skip vanished files until we know what is writing after exit. A transient file could hold the key, and skipping it would hide that.

Next step: read the file name from the next failing postmerge run. If the file comes from a writer that outlives exec, fix the writer to finish before exit.
