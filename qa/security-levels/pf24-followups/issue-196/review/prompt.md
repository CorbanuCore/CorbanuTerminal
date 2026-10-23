You are an independent senior security and Rust reviewer (Opus 5.5, high effort). Review the last commit on this branch (`git show HEAD`, base origin/main 93707fbb1b). Do not edit files; read the code and report.

Context: issue #196. PF-24-S03 reported a provider key in the private TUI log with trace logging on. The reported hits turned out to be #179 (fixed by #183). A new regression test then found that `core/src/session/session.rs` logs `Configuring session: model=…; provider={:?}` at DEBUG, and the derived `Debug` of `ModelProviderInfo` printed `experimental_bearer_token` and the values of `http_headers` and `query_params`. The logs DB and `/feedback` buffer record DEBUG/TRACE regardless of RUST_LOG.

The change has three parts:
- A hand-written `Debug` for `ModelProviderInfo` that redacts those values, keeps the names and destructures every field.
- A core integration test: a model turn with a sentinel bearer token, plus a sentinel env var used by a `shell_command` tool call. It checks the trace file, the feedback buffer and the log DB files.
- The sink setup is shared with the #179 test.

Look for:
- Other places where provider credentials or auth headers can still reach logs. Examples: `codex_api::Provider` or `HeaderMap` Debug, `Config`/`SessionConfiguration` Debug dumps, request-building debug, error paths that log request or provider structs, `env_http_headers` values, the `auth` command args, `base_url` userinfo.
- Whether redacting `query_params` and all `http_headers` values is right.
- Test validity: does the test really capture all tasks with `set_default` on a current-thread runtime? Would it catch regressions?
- Style against codex-rs/AGENTS.md.

Output Markdown: a verdict (approve / approve with fixes / request changes), then numbered findings each with severity (critical/high/medium/low/nit), file:line, the problem, and a concrete fix. Be concise.
