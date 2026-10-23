# Fixes that changed AGG-25 and AGG-32 behaviour (integrator note, written after execution)

No owner decision in `design/packet/owner-decisions.md` and no amendment covers these two cases; the frozen
expectations stand and both are dispositioned **failed** against them. This note records why the product now
behaves differently, so the owner can decide whether the case or the product is wrong.

- AGG-25 (screen E1, [design/first-run-screens/E1-preflight-blocked.txt](../design/first-run-screens/E1-preflight-blocked.txt)):
  E1 was produced by launching with `-c 'cli_auth_credentials_store="file"'`. Issue #438 (closing comment
  2026-10-11T11:55:28Z, fixes #448 and #457) made the preflight stop treating the top-level
  `cli_auth_credentials_store` / `mcp_oauth_credentials_store` enum values as secrets. G15 applied that flag and the
  preflight passed.
- AGG-32 (screen E2, [design/first-run-screens/E2-startup-boundary-warning.txt](../design/first-run-screens/E2-startup-boundary-warning.txt)):
  E2 was produced by `/etc/codex/config.toml` containing `openai_base_url` and `cli_auth_credentials_store = "file"`
  (verified present in G21, [common/g21-fixture-on.txt](../common/g21-fixture-on.txt)). Issue #444 (closing
  comment 2026-10-11T13:43:09Z) says #452 changed the warning wording and "#448/#457 remove the
  /etc/codex/config.toml false positive behind it". G21 showed "preflight clean at launch" and no warning.

Whether a readable extra config layer should still make the boundary "not clean" is an owner question; the record
does not answer it.
