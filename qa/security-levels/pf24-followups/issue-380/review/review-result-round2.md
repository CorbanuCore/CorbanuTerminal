**Verdict: approve with fixes.**

Both round-1 blockers are fixed, and every should-fix and nit was either fixed or deferred with a stated reason. One fix introduced a new false negative (finding 1). There is also one AWS SDK credential leak, below the new cap, that neither round caught (finding 2). Neither is a default-path header dump, so I'm not blocking on them.

I didn't run the Rust tests because the checkout is read-only. To check the redaction logic, I ported the patterns, `is_redacted` and `followed_by_wrapper` to Python and ran test strings through them.

**What I verified:**
- **AWS cap:** `aws_*` at DEBUG (`log-guard/src/guard.rs:37`) caps aws-sigv4's canonical-request TRACE event and aws-smithy-runtime's "transmitting request" TRACE event. I read the DEBUG-level output of aws-config, aws-runtime, aws-smithy-runtime and aws-smithy-http-client: no headers or bodies. Identity, Credentials and Token `Debug` output hides the secrets.
- **Wrapper and tuple redaction:** `Some("…")`, `HeaderValue { _private: H0("…") }`, `Bytes(b"…")` and tuple forms are all redacted (`redact.rs:36-42`).
- **Install sites:** every non-test install is guarded, including `code-mode-host`. The remaining `set_default` / `with_default` calls are all inside `#[cfg(test)]`.
- **Libraries I re-checked:**
  - zbus `Message` `Debug` prints only the body signature, not the secret.
  - rmcp's `transport::common::auth` and streamable HTTP client log no tokens.
  - teloxide removes the bot token from its errors.
  - OTel log and trace exports are filtered to Codex targets plus spans.
- **Interest-cache warm-up in the aws-auth test (`aws-auth/src/lib.rs:301-304`):** harmless. `set_default` already rebuilds the interest cache, so the extra `rebuild_interest_cache()` is redundant. The guarded assertion can't pass without being tested, because the unguarded control runs first in the same test. After the warm-up, no other test registers these callsites or installs a subscriber.

## Findings

**1. Should-fix: the `followed_by_wrapper` skip now drops real values.** (`log-guard/src/redact.rs:97-99`, used at `:107`)
- Any value followed by `(` or `{` is skipped, even when the wrapper pattern (`:37`) doesn't match.
- These inputs now leak:
  - `x-api-key=abcd1234secret (from env)`
  - `authorization: Bearer abc123secretxyz {retry}`: the token is under 16 characters, so the bearer pattern misses it.
- In `x-api-key=KEY123 (source "env")`, the wrapper pattern redacts `"env"` and leaves `KEY123` visible.
- Round 1 redacted all of these, so this is a regression.
- **Fix:** skip the first pattern's capture only when all of these hold:
  - the value is a single identifier with no scheme;
  - the next byte is `(`, or the next two bytes are ` {` (Rust `Debug` shapes);
  - the wrapper pattern matched the same header occurrence (same match start).
- Add the three strings above as test cases.

**2. Should-fix: aws-config logs the ECS container authorization token at WARN.**
- **Where:** aws-config 1.8.12 `src/ecs.rs:117,125` logs `warn!(token = %auth_token, "invalid auth token")` when the token doesn't parse as a `HeaderValue`. A trailing newline in `AWS_CONTAINER_AUTHORIZATION_TOKEN_FILE` is enough to trigger it.
- **Why it matters:** this token is the `Authorization` header value for the container credentials endpoint. Codex uses the default credentials chain (`aws-auth/src/config.rs:14`), so the event reaches the logs DB, `/feedback` and OTel-independent sinks at the default level.
- The `aws_*` DEBUG cap doesn't stop it, and no redaction pattern matches `token=…`.
- **Fix:** add `("aws_config::ecs", Level::ERROR)` to `TARGET_CAPS` (`guard.rs:37`), plus a matching row in the `is_capped` table test.

**3. Nit: the two-word "already redacted" form accepts any first word.** (`redact.rs:88`)
- `[_, marker]` lets any word come before a marker. So these leak:
  - `cookie: sid=abc123 REDACTED` and `cookie: sid=abc123 *`, via the cookie pattern that runs to end of line;
  - `("authorization", "secret REDACTED")`.
- **Fix:** accept the two-word form only when the first word is a known scheme (`bearer|basic|token|digest|negotiate`).

**4. Nit: quoted values are cut to one token.** (`redact.rs:33`)
- `"x-api-key": "abc def"` becomes `"REDACTED def"`.
- `Authorization: Token token="abc123"` becomes `REDACTED"abc123"`.
- **Fix:** when the opening quote was consumed, capture up to the matching closing quote. Keep the one-token rule for unquoted values only.

**5. Nit: the wrapper pattern's search window crosses closing brackets.** (`redact.rs:37`)
- `api-key: Some(Sensitive), "x-request-id": "req-1"` redacts `x-request-id`. This is a harmless false positive.
- **Fix:** use `[^"'\r\n)}]{0,48}?` for the window.

**6. Nit: gaps in the query-parameter and form-body pattern.** (`redact.rs:55`)
- `&refresh_token=`, `id_token`, `code_verifier` and `client_assertion` are missed, as are JSON `"access_token":"…"` and `"refresh_token":"…"`.
- I found no library path that currently logs these: oauth2 doesn't log, and rmcp auth is capped. So this is defense in depth only.
- **Fix:** add these parameter names. Optionally, treat `access_token`, `refresh_token` and `id_token` as credential field names in the header and field pattern.

**7. Nit: the QA evidence doesn't say which build it tested.** (`qa/.../issue-380/README.md:23-29`)
- The functional run doesn't name the commit or binary. Round 2 changed redaction, so it isn't clear whether the run covers 1674316048.
- **Fix:** record the tested commit, and re-run the functional scan on the final tree.