You are a security bug-fix worker. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md
(binding: keychain rule, disk hygiene, per-fix gate, secrets) first, then GitHub issue #380 (`gh issue view 380`) and
the earlier sentinel-replay work it references (#196; find it with `gh pr list --state merged --search 196`).
Fix #380: with `RUST_LOG=trace`, tungstenite's handshake TRACE event writes the websocket upgrade request, including
`authorization: Bearer <key>`, to codex-tui.log. Required end state: no credential header (Authorization, api-key,
x-api-key, cookies, ChatGPT tokens) from any HTTP/websocket client library reaches ANY log sink — codex-tui.log, the
`pfterminal_logs_*.sqlite` logs DB, the feedback buffer, exec stderr/OTel — under any RUST_LOG, including user-supplied
filters like `trace` or `tungstenite=trace`. Prefer a robust guard (force-cap tungstenite/tokio_tungstenite/hyper/h2/
reqwest/rustls-related targets that can dump headers, applied after parsing user filters, in every subscriber: TUI, exec,
app-server, mcp-server, and the Windows broker if it logs) plus a defence-in-depth redaction layer if practical. Audit
other libraries in the dependency tree that log headers at trace (reqwest/hyper/h2/eventsource) and cover them too.
Regression tests: sentinel key in a websocket turn and an HTTP turn under `RUST_LOG=trace` against a local mock server;
assert the sentinel appears in none of the sinks. Gate: focused tests; Linux clippy -D warnings on RTX
(ssh ambient@100.99.88.49); one independent Opus 5.5 High read-only review via installed `corbanu exec`; a functional check
with the fresh debug binary in a disposable home (CORBANU_TEST_NO_NATIVE_KEYRING=1) using the real OpenAI key via
`OPENAI_API_KEY="$(env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME ~/.local/bin/corbanu vault auth-helper openai-api-key)" <cmd>`
with RUST_LOG=trace, one TUI turn on gpt-5.4 via the tmux harness (docs/tmuxHarness.md) and one exec turn; then scan the
home for the key with grep -F reading the key from the same helper (never print it) and delete the home. New worktree off
origin/main under /Volumes/CorbanuDrive/Corbanu/worktrees/. PR to main; merge only when ALL checks are green, never with
checks pending (`gh pr merge N --merge --match-head-commit <sha>`; never force-push; never rm -f). Close #380 with evidence.
Remove your build target when done. Report as in sec-common.md.
