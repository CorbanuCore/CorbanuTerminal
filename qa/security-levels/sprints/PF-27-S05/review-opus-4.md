# PF-27-S05 review 4 (Opus 5.5 High via OpenRouter, re-check; commit b0e6785fb9)

**VERDICT: CHANGES REQUESTED**

I checked fixes 1–10 against `origin/main...HEAD` (HEAD `b0e6785fb9`), read-only. The core changes hold up:

- **Fix 1:** Core fails closed before the broker starts. Websockets, realtime and header-only users are off.
- **Fix 2:** A sign-in used with an env-key provider is brokered with that provider's header.
- **Fixes 5–8 and 10:** registration runs outside the lock, the lock file is safe, a refused hand-over marks the broker Failed, slots are stable, and the transport test cleans up after itself. All verified.
- **Fixes 3 and 9:** accepted as known limits.

I found no path that sends a raw key or the placeholder with the flag on. Tests were not run because the sandbox is read-only.

Findings:

1. **Medium: command-backed provider auth (Claude Plan, any `provider.auth`) now sends no credential when the flag is on.** `model-provider/src/auth.rs:250-272`.
   - `brokered_auth_request` returns `None` because `provider.auth` is set (line 345).
   - The code then falls through to `auth_provider_from_auth(CodexAuth::ApiKey(<command token>))`. Because the broker is required, that returns no credential (lines 568-578).
   - Requests go upstream with no auth and get a 401. The module docs say command auth is "not brokered", which implies it is sent as before.
   - **Fix:** when `provider.auth.is_some()`, use `direct_auth_provider_from_auth`, or broker the token as a `Value` in a `command:<provider>` slot. Add a `pf_27_s05` test for command auth with the flag on.

2. **Low–Medium: the broker may never be installed even though the process requires it.** `core/src/model_broker_auth.rs:46-51`.
   - `install_for_config` only checks the flag on the session's config. But `require_model_key_broker()` is set by whichever config loads first with the flag on.
   - If a config reload or profile turns the flag on, or a later session's config has it off, every brokered credential fails with "not running yet" for the rest of the process.
   - **Fix:** also install when `codex_model_provider::model_key_broker_required()` is true.

3. **Low: if the broker fails to start, the provider keys stay in Core's environment.** `model_broker_auth.rs` `start`, the `Err` branch of `spawn`.
   - Core never uses those keys once the broker is required, but MCP servers and hooks still inherit them.
   - **Fix:** scrub the keys on this path too: call `take_env_var` for each name and drop the value.

4. **Low: only the first config's provider key variables are handed over.** `BrokerSettings::for_config` (`model_broker_auth.rs:79-86`).
   - A provider added by a later config keeps its key variable in the environment, where child processes can read it, and requests for it fail with `MissingKey`.
   - **Fix:** hand over newly seen names on later installs or uses, or document this as a known limit.

5. **Low: a token refresh race can replace the newer registration with the older one.** `credential_for` (`model_broker_auth.rs`, the final `insert`).
   - The insert is unconditional. A request holding an older snapshot that registers after a newer one replaces it and unregisters the newer credential.
   - Two stale snapshots can keep doing this to each other, so failures can repeat rather than happen "once".
   - **Fix:** only replace when the cached version still equals the version seen before `register`; otherwise unregister your own copy and return the cached one.

6. **Low (record and process): the sprint record does not match what was done.** `docs/sprints/current/p0-security-levels/pf-27-s05-model-client-auth-broker.md`.
   - It does not contain the known limits your fix 3 disposition says it records.
   - Done still says "Sign-in tokens … not brokered (sent as before)".
   - Remaining still lists env scrubbing, vault-in-broker and the other key paths as open.
   - `write_scope` leaves out files the diff touches: `model-provider/src/{provider,model_key_broker}.rs`, `http-client`, `login`, `secrets/src/local.rs`, `process-hardening`, `arg0`, `core/src/config/mod.rs` and `core/src/realtime_conversation.rs`.
   - **Fix:** update Done, Remaining, Known limits and `write_scope` (via the plan's sprint map), then run `docs/sprints/check.py`.

7. **Low (claim accuracy): "provider keys stored in the vault are read by the broker, never by Core" overstates it.**
   - The in-process TUI still decrypts the Corbanu key for wallet, usage and the campaign tracker (`tui/src/chatwidget/wallet_http.rs:18-27`, `campaign_tracker.rs:82-94`).
   - Those features now also lose a key supplied only as an environment variable, because the broker took it.
   - Agent-identity registration still sends the ChatGPT access token directly (`agent-identity/src/lib.rs:377`).
   - **Fix:** narrow the docs to model-client requests and record these as known limits or follow-ups.
