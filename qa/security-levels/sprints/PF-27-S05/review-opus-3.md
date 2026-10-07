# PF-27-S05 review 3 (Opus 5.5 High via OpenRouter, independent, read-only, keyring-isolated profile; commit b49e636f3c)

VERDICT: CHANGES REQUESTED

I read the diff and the surrounding code only. I didn't run any tests or the product.

1. **High: credentials still go out directly during startup, before the broker exists.** Files: `core/src/model_broker_auth.rs:45-63`, `core/src/session/session.rs:564`, `model-provider/src/auth.rs:249,549`, `provider.rs:342,465`.
   - The broker is only started inside `Session::new`. Every check is "is a broker installed?", not "is the flag on?".
   - Before the first session, the app-server's models refresh worker (`app-server/src/message_processor.rs:307`, which runs an online refresh straight away) sends the raw ChatGPT token to `/models` through `BearerAuthProvider`.
   - In the same window, `account_state` and `provider_env_auth` decrypt vault keys inside Core. `AuthManager::provider_api_key_cache` then keeps the decrypted key indefinitely.
   - `cli doctor` never starts a session, so it always sends keys directly.
   - **Fix:** start the broker at process start, after config loads and before the app-server, TUI or refresh worker. Separately, when the flag is on but no broker is installed yet, refuse the request (frame-only or fatal) instead of attaching the key.

2. **Medium: env-key providers fall back to direct auth when the login isn't an API key.** File: `model-provider/src/auth.rs:359-361`.
   - `brokered_auth_request` returns `Ok(None)` when `env_key` is set and the auth is ChatGPT, ChatGPT auth tokens, a personal access token or agent identity. `resolve_provider_auth` then continues to `auth_provider_from_provider_key_auth`, which attaches `auth.get_token()` in Core (Bearer or `x-api-key`).
   - This is reachable: `OpenAiModelsEndpoint::auth` (`models_endpoint.rs:61-66`) passes the main login, not the provider-key placeholder. Any provider with an `env_key` and `requires_openai_auth = true` sends the raw ChatGPT token to that provider.
   - **Fix:** with the broker installed, build a `ProviderKey` request and ignore `auth`. More generally, once a broker exists, every fall-through after the broker branch must either attach no Core-held secret or return an error. Add a test for this case.

3. **Medium: the environment scrub is unsound in a multi-threaded process.** File: `network-proxy/src/credential_broker/env_scrub.rs:24-56`.
   - `take_env_var` runs on a `spawn_blocking` thread while the tokio, MCP and DNS threads are live.
   - `overwrite_in_place` walks `environ` without Rust's env lock. A concurrent `set_var` can reallocate the array mid-walk (use-after-free).
   - `remove_var` (glibc `unsetenv`, which shifts the array) races with C-level `getenv` calls that don't take that lock (getaddrinfo, TLS/CA lookups). The SAFETY comment only covers Rust callers.
   - It also misses one case: a launch-env value that was later replaced via `.env`/`set_var`. That original stack copy is no longer reachable through `environ`, but `/proc/<pid>/environ` and `ps -E` still read it.
   - **Fix:** move the provider-key variables out of the environment in `arg0` while the process is still single-threaded (that's where `load_dotenv` runs). Hold them in `Zeroizing` until they are handed to the broker, then wipe them. Also scrub the original envp block.

4. **Medium: one profile turning the flag on changes behaviour for everything else in the process.** Files: `model-provider/src/model_key_broker.rs:94-103`, `core/src/model_broker_auth.rs:49`.
   - The broker is a process-wide `OnceLock`. Once any session's config enables `broker_model_auth`, every other session or thread in that app-server or TUI process loses websockets and realtime.
   - Those sessions also lose header-only credentials: MCP apps, uploads, analytics, cloud-tasks.
   - The environment scrub also changes what later child processes inherit (user shell tools no longer see `OPENAI_API_KEY`-style variables).
   - This breaks "flag off is unchanged" for flag-off sessions in a mixed process. **Fix:** make the flag process-scoped and decide it once at startup (this follows from fix 1), or disclose it and test the mixed case.

5. **Medium: async worker threads block on the broker while holding a mutex.** File: `core/src/model_broker_auth.rs:326-380`.
   - The synchronous `ModelKeyBroker::auth` is called from async request setup and holds `BrokerState`'s `std::sync::Mutex` during `register`/`register_stored`.
   - That call can block for up to the 10s `CONTROL_TIMEOUT`, plus a vault scrypt decrypt and an OS keyring IPC on the broker side. It stalls tokio workers and serializes every request.
   - **Fix:** use `spawn_blocking` (or an async control call), and release the lock across the call by using a per-key in-flight once-cell.

6. **Medium (Linux): keys saved after startup can't be read, and the lock-file rule follows symlinks.** File: `process-hardening/src/broker_containment.rs:215` and `server.rs:117-126`.
   - The Landlock rule for the vault lock file is only added if `.vault.lock` already exists. If the vault is first created during the session (first key saved via /providers), the broker can't open the lock, every request fails with StoreUnavailable, and only a restart fixes it.
   - `is_file()` follows symlinks, and so does Landlock's path rule. A pre-planted symlink would give a compromised broker write access to the symlink's target.
   - **Fix:** in the broker, before containment, run `create_dir_all(secrets)` and open the lock file with `O_CREAT|O_NOFOLLOW`. Require `symlink_metadata().is_file()`, then add the rule.

7. **Low: `take_env_keys` doesn't do what its doc comment says.** File: `network-proxy/src/credential_broker/model_auth.rs:166-183`.
   - The variable is removed before `stash_env`. If the broker refuses it, the key is lost from the environment rather than "left in place" as documented.
   - The `?` also stops the loop, so later variables stay in Core's environment, and `install_for_config` (`model_broker_auth.rs:235-242`) only logs a warning.
   - **Fix:** keep going through all names, and on any failure mark the broker as `Failed` so requests fail closed.

8. **Low: old copies stay in the broker.**
   - When a stored key is deleted, the re-register gets NotFound, but the broker's old credential is never unregistered and stays there until exit.
   - Value-slot entries (`value:<fingerprint>`) pile up whenever the API-key login changes.
   - Unregistering on a token refresh can fail an in-flight request that was signed with the old reference.
   - **Fix:** unregister on NotFound, use a stable slot for API-key logins, and delay unregistering the replaced credential until in-flight requests finish.

9. **Low: the memory-scan test claims more than it checks.** Files: `isolated_tests.rs:1201-1302`, `memory_scan_tests.rs`.
   - The test design is sound: the key is held masked, there's a positive control, and on Linux `/proc/self/environ` is checked.
   - But it runs a network-proxy stand-in, not Core's `install_for_config` → `provider_env_auth` path. It can't catch copies made by `.env` loading, config/telemetry `env::var` calls, or pre-install reads (finding 1).
   - **Fix:** rename it or document that scope. Add a positive control that `/proc/self/environ` contains the key before hand-over. Ideally add a core-level scan after a brokered request.

10. **Low: test coverage gaps.**
    - The removed `client_tests` case was the only explicit flag-off "direct auth unchanged" check. Restore it at the `resolve_provider_auth` level.
    - `install_model_broker_client` in `transport_tests.rs` is process-global and never reset. Add a reset or a serial guard.
    - Add tests for findings 2 and 6.

11. **Low (scope to disclose): these paths still attach credentials directly with the flag on.**
    - The Bedrock `AWS_BEARER_TOKEN_BEDROCK` bearer token.
    - Agent-identity bootstrap, which sends the raw access token from `agent-identity/src/lib.rs:377`.
    - Command auth.
    - TUI-side reads of `PFTERMINAL_PLAN_API_KEY` in the same process (`wallet_http.rs`, `campaign_tracker.rs`, `pfterminal_plan_status.rs`, the Claude bridge). Once that variable has been scrubbed, these features also lose keys that were only in the environment.
    - Record these in the sprint as explicitly not brokered, or refuse them under the flag.
