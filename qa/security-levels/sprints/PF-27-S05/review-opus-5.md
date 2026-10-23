# PF-27-S05 review 5 (Opus 5.5 High via OpenRouter, final re-check; commit 7f741b441)

VERDICT: APPROVE

All seven review-4 fixes hold, and I found no regression with the flag off. Tests weren't run (read-only sandbox); `docs/sprints/check.py` passes.

**Review 4 fixes:**
1. **Command and AWS auth go direct:** fixed. The final match in `resolve_provider_auth` (`model-provider/src/auth.rs:273`) now sends command and AWS auth through `direct_auth_provider_from_auth`. Config validation keeps `auth` and `aws` apart from `env_key` and `experimental_bearer_token`, so nothing can get past the broker branch above it. `pf_27_s05_command_auth_is_sent_as_before` checks the header goes out and the broker is never used.
2. **Install when required:** fixed. `install_for_config` (`core/src/model_broker_auth.rs:47-53`) now installs when the flag is on or the process is already brokered, unless a broker is already installed.
3. **Keys scrubbed on a failed start:** fixed. When the broker fails to start, `scrub_env_keys` runs before the broker is marked Failed. A refused hand-over also removes every key first.
4. **Only the first config's keys are handed over:** listed under Known limits as you said.
5. **Stale registration race:** fixed as asked. `seen` is captured under the lock, and the result is only replaced when the cached version still equals `seen`; otherwise Core drops its own copy and returns the cached one.
6/7. **Record and claims:** fixed. The Done list, Known limits and `write_scope` now match the diff.

**Flag off:** nothing changes.
- `model_key_broker_required()` is false, so `auth_provider_from_auth` and `direct_auth_provider_from_auth` behave the same. The placeholder, account-state, websocket and realtime branches stay inactive, and `install_for_config` returns early.
- The transport only takes the broker route when a request carries the frame header.
- `HttpClient::from_parts` sets `broker_socket: false`.
- The seatbelt profile is unchanged when there are no extra files.
- One small change applies whatever the flag: `secrets/src/local.rs` no longer re-runs chmod on files that are already private. This is safe.

**Nonblocking notes (record as follow-ups):**
1. **Low (docs), `core/src/model_broker_auth.rs:10-11`:** the module doc still says stored vault provider keys are "read by the broker, never by Core". Narrow it to "for model-provider requests" to match the Known limits.
2. **Low (docs), `qa/security-levels/sprints/PF-27-S05/README.md:21`:** the slice-1 text says the paths that still send keys directly are "listed under Remaining in the sprint record". Remaining now lists only the review and merge. Mark that section as historical, or point to Known limits.
3. **Low (availability, not security), `core/src/model_broker_auth.rs:375-420`:** the `seen` check stops concurrent races. A stale snapshot that starts *after* a newer token is cached still replaces it, because versions are hashes with no ordering. That costs one failed request per swap and is already covered by the "fails once" limit. A monotonic version for sign-in slots, such as an issued-at time, would end it if this ever shows up in practice.
