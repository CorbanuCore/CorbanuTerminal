# #196: provider secrets in the trace log

PF-24-S03 reported that the provider key showed up in the private TUI log with
trace logging on.

## Reproduction on main `93707fbb1b`

- **The PF-24-S03 hits were #179.** Every run the demo tool redacted used a
  build from before #183, and the redacted line was the `spawn_child_async`
  environment dump. The final PF-24-S03 runs on #183 code redacted nothing.
- **Fake `ZAI_API_KEY` sentinel, `RUST_LOG=trace`:** covered startup, a model
  turn (401), a `!` command, `/status`, `/providers` and `/exit`. The sentinel
  appeared 0 times in `codex-tui.log`, `pfterminal_logs_*` (including WAL) and
  the profile.
- **A new core regression test found a different leak:**
  `Configuring session: …; provider={:?}` (DEBUG) printed the derived `Debug`
  of `ModelProviderInfo`. That includes `experimental_bearer_token`, the
  `http_headers` values and the `query_params` values. Keys read from `env_key`
  were not affected.

## Gate evidence

- **Tests:** the new core test (now
  `suite::log_redaction::provider_credentials_and_tool_env_never_reach_trace_logs`)
  failed on main with "secret leaked into the trace log file" and passes with
  the fix. Unit tests cover `ModelProviderInfo` Debug and `redact_url`, and the
  transport logging test now expects `token=REDACTED`. The #179 test moved to
  the same module and still passes.
- **Review:** Opus 5.5 High, approve with fixes; see `review/disposition.md`.
- **tmux run on GLM 5.2:** a copy of the Z.AI provider (`zai-demo`) with
  `http_headers = { "X-Demo-Secret" = "<fake sentinel>" }`. One model turn ran
  `ls -a`, followed by `!echo hello` and `/exit`.

  | Build | Sentinel in log / profile | Header in log | Real key matches |
  | --- | --- | --- | --- |
  | Before (a main build; its only change is unrelated to logging) | 1 (`codex-tui.log`) | value printed | 0 |
  | After (`520621748f`) | 0 | `"X-Demo-Secret": "<redacted>"` | 0 |
  | After review fixes (`4e4431d4a7`), plus `query_params = { "demo_key" = "<fake>" }` | 0 (both sentinels) | `"<redacted>"`; request URLs log `demo_key=REDACTED` (4 lines) | 0 |

  Screens: `tmux-before-model-turn.txt`, `tmux-after-model-turn.txt` and
  `tmux-after2-model-turn.txt`. The
  real key was checked through the vault helper; only counts were printed.
- **Video:** `qa/demos/index/issue-196.md`.
