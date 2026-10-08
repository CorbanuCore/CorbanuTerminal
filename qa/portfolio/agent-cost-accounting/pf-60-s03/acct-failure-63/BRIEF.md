# Worker brief: "Native Anthropic accounting failed; request stopped without a repair send" keeps recurring

You are an Opus 5.5 High worker. Work autonomously; do not ask questions. This is Travis's top-pain bug: it has
killed his interactive sessions repeatedly and just killed a long-running `corbanu exec` worker.

## Latest occurrence (evidence)
- Worker `ci-bazel-test` (`corbanu exec -m claude-opus-5-5-plan -c model_provider="claude-plan"`), log
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/ci-bazel-test.log`, died around 2026-10-03 21:05–21:09 UTC
  after ~4 h and 575k tokens with two `ERROR: Fatal error: Native Anthropic accounting failed; request stopped without
  a repair send`, EXIT:1.
- Same time window in `CODEX_HOME=/Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/home`
  (`pfterminal_logs_2.sqlite`, table `logs`): repeated `slow statement` WARNs for the connection PRAGMA batch taking
  5.2 s (≈ a 5 s busy timeout) on 21:04:24/29/34, a 2.6 s slow pool acquire, and a TUI input-watchdog warning. The
  state DB `pfterminal_state_5.sqlite` is 72 MB with a 4.5 MB WAL, on an external drive (`/Volumes/CorbanuDrive`), and
  is shared by the interactive TUI, several concurrent `corbanu exec` workers, the owner loop's tmux workers and the
  manager lane.
- Earlier fixes already on the integration branch: lock-hold fix, ephemeral exec turns not collected (`1aca553905`),
  clock reads under the write lock via `AsOf::Now` (`04fdea374e`/`9aabf01ded`), acct-scope-62 `/cost` scope fix
  (`704650ccd8`). See `.codex-work/workers-20261002/acct-scope-62.md` and `.result.md` for the last round's context.
- In `codex-rs/core/src/accounting.rs` (and `accounting_chat.rs`, `accounting_extensions.rs`) many distinct errors
  collapse into `CodexErr::Fatal(FAILURE)` / `anyhow!(FAILURE)` and `Sampling::reject()` poisons the sampling, so the
  real cause is never recorded.

## Do
1. **Make it diagnosable first.** Wherever an underlying error becomes `FAILURE` or rejects a sampling, log the full
   error chain (tracing `error!`/`warn!` with a stable target such as `codex_core::accounting`, no secrets, no prompt
   content) including which step failed (open/admit/observe/settle/repair, etc.).
2. **Find the root cause from evidence.** Search the logs DB for any existing accounting-related records around the
   failures (today and earlier days), inspect the accounting tables in the state DB (read-only, `sqlite3 -readonly`)
   for rejected/incomplete attempts at those times, and reproduce: run several concurrent `corbanu exec` sessions
   against the same CODEX_HOME copy (use a scratch copy of the home, never the live one, and a cheap/fake provider or
   the repo's mock-server test harness) while another process holds write transactions, and see whether
   admission/observation fails on SQLITE_BUSY / "database is locked" / pool-acquire timeouts or something else.
3. **Fix the cause, not the symptom.** E.g. if contention: make accounting writes wait correctly (busy_timeout /
   bounded retry with backoff on BUSY for the accounting transactions, short transactions, no long reads holding the
   write lock), and make a transient store error retryable without permanently poisoning the sampling, while still
   failing closed when accounting truly cannot be recorded (keep the existing fail-visible contract and its tests).
   If it's something else, fix that. Add regression tests that fail before and pass after (unit + the existing
   `core/tests/suite/accounting_*` style).
4. Work on a fresh branch off the tip of `integrate/management-workstreams-20260911` in a new worktree
   `/Volumes/CorbanuDrive/Corbanu/worktrees/acct-failure-63` (CorbanuCore/CorbanuTerminal). Run `just fmt`, the
   accounting tests, `cargo test -p codex-core` accounting-related suites, and clippy for touched crates.
5. Independent review: `~/.local/bin/corbanu exec -m claude-opus-5-5-plan -c model_provider="claude-plan" -c
   model_reasoning_effort="high"` on the diff; fix findings.
6. Push fast-forward to `integrate/management-workstreams-20260911` (pull/rebase your branch first; never force;
   never stage the four uncommitted `scripts/initiative_control/` files in the main management worktree — don't work
   there at all).
7. Rebuild and install per the SOP: `.codex-work/integration-build-20260922a/dev-build.sh` (repo at
   `.../integration-build-20260922a/repo`, update it to the new tip), copy `target-dev-active/debug/corbanu` to
   `integration-package-dev/bin/corbanu.new`, `codesign --force --options runtime --timestamp --identifier
   com.corbanu.corbanu --sign DB43D92853B5EF2BCE87DA9B2F28360DC44BBEBD`, verify, `mv` over `bin/corbanu`, update
   `INSTALL.json`. Then RTX: `ssh ambient@100.99.88.49`, update `~/corbanu-rtx/integration` to the tip and run
   `~/corbanu-rtx/build.sh`. Note: replacing the binary doesn't affect already-running processes.
8. Functional check: run 3 concurrent `corbanu exec` sessions with the installed build on the real setup for a few
   short turns each while the TUI is running, confirm no accounting failure and that `/cost` still accounts them.

## Never
Never print secrets. `rm -f` is blocked by policy. No force-push. Don't edit the live state DB by hand.

## Final report (your last message)
Root cause with evidence, the fix (commits), tests before/after, review verdict, build/install receipts (local + RTX),
functional check results, and anything still open.
