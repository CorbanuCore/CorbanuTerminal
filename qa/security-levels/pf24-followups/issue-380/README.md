# #380: websocket bearer key in the trace log

With `RUST_LOG=trace`, tungstenite's client handshake TRACE record (a `log`
crate record) wrote the upgrade request, `authorization: Bearer <key>`
included, to `codex-tui.log`.

## Fix

- `codex-log-guard::guard` wraps every subscriber (TUI, exec, app-server,
  mcp-server, exec-server, login, cloud-tasks, execve wrapper, code-mode-host).
  It disables HTTP, websocket, TLS and AWS SDK library targets above their cap
  (aws-sigv4 also logged the SigV4 canonical request, session token included,
  at TRACE, which the logs DB and `/feedback` capture by default)
  before any layer or `RUST_LOG` filter runs, `log` records included.
- Sink writers (TUI log, feedback buffer, logs DB, stderr) also redact
  credential header values, bearer tokens, `sk-` keys, JWTs and credential
  query parameters.

## Gate evidence

- **Tests:** the core websocket test and the exec websocket test fail with the
  guard removed (handshake request in the log, key in exec stderr) and pass
  with it. The aws-auth test shows the session token in an unguarded sink and
  not in a guarded one.
- **Linux clippy (RTX):** `-D warnings` on every touched crate, 0 warnings.
- **Functional, real OpenAI key, debug `corbanu` built from `fd92e6799e`
  (also run on `1a84d9fb79` and `1674316048`), disposable home,
  `CORBANU_TEST_NO_NATIVE_KEYRING=1`, `RUST_LOG=trace`:** one exec turn on
  gpt-5.4 (`EXEC_OK`) and one TUI turn in tmux (`TUI_OK`), both over the
  Responses websocket ("Client handshake done." logged, no handshake request
  line). A `grep -F` scan of all 5532 files in the home, key read from the vault
  helper, found 0 matches. The home was deleted.
- **Review:** Opus 5.5 High. Round 1 requested changes and round 2 approved
  with fixes. Every item is fixed or recorded in `review/disposition.md`.
