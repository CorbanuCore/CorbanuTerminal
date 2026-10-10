# Review disposition, round 1 (Opus 5.5 High, request changes)

| # | Severity | Disposition |
| - | -------- | ----------- |
| 1 | Blocking | Fixed. `aws_*` targets are capped at DEBUG: aws-sigv4 logged the canonical request (with `x-amz-security-token`) and aws-smithy-runtime every request at TRACE, which the logs DB and `/feedback` capture with no `RUST_LOG`. `x-amz-security-token` and `x-amz-sso_bearer_token` are redacted too. New `codex-aws-auth` test: an unguarded control sink records the session token; the guarded sink does not. |
| 2 | Should-fix | Fixed. Wrapped values (`Some("…")`, `HeaderValue { _private: H0("…") }`) and tuples (`("api-key", "…")`) are redacted; tests cover all three. |
| 3 | Should-fix | Fixed for the listed names. Runtime registration of user-configured header names is not added: their values are already marked sensitive (#196), and library dumps are stopped by the caps. |
| 4 | Should-fix | Fixed. Only an exact marker (after an optional scheme) counts as already redacted. |
| 5 | Nit | Fixed. Unquoted values are an optional scheme plus one token (cookies and SigV4 `authorization` still run to the end of the value). |
| 6 | Nit | Fixed. Credential query parameters (`key`, `api_key`, `access_token`, `token`, `sig`, `code`, `client_secret`, `password`, `x-amz-*`) are redacted. |
| 7 | Nit | `code-mode-host` is guarded. A source-scanning CI check is not added (it can't read the workspace under Bazel); noted as a follow-up. The Windows sandbox command log is outside #380. |
| 8 | Nit | Added a test that spans, `Span::current()` and registry downcasts pass through `Guarded`. The HTTP tests stay as canaries. |

# Round 2 (Opus 5.5 High, approve with fixes)

| # | Severity | Disposition |
| - | -------- | ----------- |
| 1 | Should-fix | Fixed. A type name is skipped only when it is directly followed by `(` or ` {` and the wrapper pattern matched the same occurrence; otherwise its bracketed contents are redacted (`Secret(x)` → `Secret(REDACTED)`). The three review strings are tests. |
| 2 | Should-fix | Fixed. `aws_config::ecs` is capped at ERROR, with cap-table rows. |
| 3 | Nit | Fixed. The two-word form needs a known scheme first. |
| 4 | Nit | Fixed. Quoted values are redacted to the closing quote; unquoted tokens may contain quoted segments (`token="x"`). |
| 5 | Nit | Fixed. The wrapper window stops at `)` and `}`. |
| 6 | Nit | Fixed. `refresh_token`, `id_token`, `code_verifier`, `client_assertion` query/form names, and `access_token`/`refresh_token`/`id_token` as field names. `None` counts as a marker so `refresh_token: None` stays readable. |
| 7 | Nit | Fixed. The README names the tested commit; the functional scan was re-run on the final tree. |
