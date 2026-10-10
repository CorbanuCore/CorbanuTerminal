# #398: credentials in sandbox command logs

The Windows sandbox writes its own daily command log
(`CODEX_HOME/.sandbox/sandbox.<date>.log`) outside the `tracing` sinks, so
`curl -H "Authorization: Bearer ..."`, `--password=...`, `X-Api-Key: ...` or
`?api_key=...` in a sandboxed command was written verbatim. `/feedback`
attaches that log.

## Audit of command logs

| Log | Platforms | Now |
| --- | --- | --- |
| Windows sandbox log: `START`/`SUCCESS`/`FAILURE` lines, notes, setup helper lines, `SBX_DEBUG` lines (`CreateProcess` failures print the command line) | Windows | Every line goes through `codex_log_guard::redact_credentials` (`log_writer` returns a line-buffering redacting writer). The 200-byte preview is redacted before it is truncated. Older logs are masked once at TUI and app-server start (`codex_state::scrub_sandbox_logs_once`). |
| `tracing` sinks: TUI log file, logs DB, `/feedback` buffer, stderr of exec, app-server, mcp-server, exec-server and helpers | all | Already redacted by `codex-log-guard` (#387). `spawn_child_async` logs the full sandbox argv (Seatbelt, Linux sandbox helper); the redaction now also covers credential options (`--password x`, `-Token x`, `--api-key=x`), `curl -u user:x`, env assignments (`OPENAI_API_KEY=x`, `$env:GH_TOKEN = 'x'`), URL passwords and GitHub, GitLab, Slack, Google, Hugging Face and AWS key formats. Seatbelt, the Linux sandbox and bwrap write no logs of their own. |
| Exec policy rules file (`rules/default.rules`), written for "don't ask again" | all | A command carrying a credential gets no amendment proposal; a client-sent one applies for the session only and is not written. |

### Excepted by design: the user's own transcript

Not redacted, because they exist to replay or resume exactly what the user and
model saw, and the model already received the command text:

- Rollouts (`sessions/**.jsonl`) and spawned-agent rollouts, which record each
  command's begin and end events.
- Message history (`history.jsonl`): the user's own prompts and `!` commands,
  kept for recall. A redacted entry would recall a broken command.
- Rollout trace bundles: opt-in (`CODEX_ROLLOUT_TRACE_ROOT`) raw request and
  response payloads, a superset of the rollout.

Not local command logs: the OTLP exporter (a collector the user configures) and
`execpolicy-legacy`'s `env_logger` (a dev CLI's stderr).

## Enforcement follow-up from #387

`clippy.toml` disallows `tracing::subscriber::set_global_default`,
`tracing::dispatcher::set_global_default`, `SubscriberInitExt::{init, try_init}`
and `tracing_subscriber::fmt::{init, try_init}` (function and builder). The
only allowed path is `codex_log_guard::guard(subscriber).{init, try_init}()`,
inherent methods that shadow the trait's. Bazel's clippy job uses the same
`clippy.toml` (`.bazelrc` `build:clippy`), so this holds under Bazel without a
source scan. Scoped `set_default`/`with_default` stay allowed: tests use them
to capture events in memory.

## Gate evidence

See the PR.
