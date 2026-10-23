# nightly-flakes progress (2026-10-05)

Worktree: /Volumes/CorbanuDrive/Corbanu/worktrees/nightly-flakes-20261005, branch codex/nightly-flakes-20261005.

- Root cause: Bazel ignores Cargo `[profile.*.package.scrypt/salsa20] opt-level = 3`; vault scrypt (log_n=16) runs
  ~50x slower (6.4s vs 0.13s per KDF, M-series). Explains login-unit-tests 290-606s, TUI embedded_provider_key_save
  (30s in-process request timeout -> StorageUnavailable), claude_cli_owns_token_exchange (30s health timeout includes
  vault persist), Windows login external-bearer tests (10s cmd.exe timeout under CPU starvation).
  Fix: MODULE.bazel crate.annotation rustc_flags -Copt-level=3 for scrypt+salsa20 (aquery: flag follows opt-level=0).
- safety_retry_* line 742: source goal time_used 13 vs 12; core accrues real wall-clock seconds. Test now bounds by
  measured active window.
- Commits 09a97c8a21 (bazel scrypt/salsa20 opt) + bfe6ed07a6 (tui test bounds). PR #166.
  Round 1 dispatched: run 37323160222 on bfe6ed07a6.
- Local Bazel login-unit-tests 67.5s -> 3.0s. Local oversubscribed (24 procs) runs also showed safety_retry
  line 680/722 failures (source thread_read missing items) — not seen in CI; noted as residual.
- Round 1 (37323160222, bfe6ed07a6): all green except Windows native (state accounting_inspect_public_reads_do_not_write:
  first poll Ready) and gnu app-server-all-test (plugin_list_honors_global_remote_catalog_cache_ttl, 200ms delay).
  TUI flaky once (existing_claude_login: ETXTBSY). login 565s->18s, vault 448->13, rmcp 310->10, cli ~250->8, tui 229->67.
- Fixes 4e7c8ac259 (state), 37ca5cfea0 (tui fixture writer), ed66c61331 (plugin delay 5s). Round 2 dispatched.
- Round 2 (37357182036, ed66c61331): Windows native: only keyring-store serialized_operation timeout (fixed 372a89298b,
  no round on it). TUI/login/state/vault all PASS on Windows. gnu job killed by runner shutdown signal (infra) at 43/414
  tests; will rerun that job once the run completes.
- Round 2 musl: core-all-test snapshot_rollback_past_compaction (hang at ThreadRolledBack wait). Cause: core emits
  TurnComplete before clearing active_turn; rollback rejected "in progress", test ignored Error. Reproduced on RTX box
  (taskset 4 cores, 7/32); fixed with rollback_thread helper 0b8587e23a (64/64). Rerunning round-2 gnu job (infra kill).
- Remote scratch: ambient@100.99.88.49:~/nightly-flakes-20261005 (src+target) — remove when done.
