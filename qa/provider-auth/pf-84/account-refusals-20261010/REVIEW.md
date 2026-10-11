<!-- Reviewer: installed corbanu exec, -m claude-opus-5-5-plan, model_provider=claude-plan, reasoning high, -s read-only, on ef9c472d63. Output saved verbatim below; dispositions follow. -->

**Verdict: request changes.** The three fixes do what they claim for the common flows, and #428 (D3 security gating) is untouched. Two paths still go wrong, though: one quietly falls back to the default account, and the exec check uses the wrong signal for whether spawning is possible. I didn't build or run anything; this is from reading the code.

### Medium

1. **Exec's "nothing can use it" check reads the wrong signal** — `core/src/config/provider_accounts.rs:132`. It treats spawned agents as possible when the `Collab` or `MultiAgentV2` feature flag is on. The real logic (`Config::multi_agent_version_override` / `multi_agent_version_for_model`, `config/mod.rs:1813-1840`) works differently, so the check is wrong in both directions:
   - **Accepts when it should refuse.** With `agents.enabled = false`, agents are off even though `Collab` defaults on. The check accepts `--account kimi:x` and then ignores it, which is the #425 bug again.
   - **Refuses a valid setup.** With `--disable multi_agent` and a model whose catalog entry sets `multi_agent_version: "v2"` (6 entries in `models-manager/models.json`), spawned agents still work, but the account is refused.
   - **Fix:** refuse only when `!config.agents_enabled`, or when both flags are off and the resolved model declares no `multi_agent_version`. Put this in one helper (e.g. `Config::spawned_agents_possible()`) that shares the version logic, and test it through a real `Config`. Today only the pure helper is tested, and that is exactly where this bug lives.

2. **Resuming with an explicit account drops the thread's recorded account and silently runs on the default** — `app-server/src/request_processors/thread_processor.rs:3689`. The same pattern exists for workers at `:707`.
   - The recorded account is used only when `provider_account.is_none()`. The TUI and exec always send the qualified `config.provider_account_override` (`tui/src/app_server_session.rs:2174`, `exec/src/lib.rs:1251`). So `corbanu resume <zai thread recorded on zai:work> --account kimi:alt` runs the session on zai's config-table or default account. This is now the documented use of another provider's account.
   - Unprefixed names are also qualified against the TUI's configured provider, not the thread's. `corbanu resume <kimi thread> --account alt` becomes `zai:alt`, and the kimi thread runs on kimi's default.
   - Workers have the same problem: a zai worker spawned with `--account kimi:alt` doesn't inherit the parent's `zai:work`.
   - This logic existed before, but #425 makes it a supported flow, and it contradicts the doc's precedence line and "the session itself stays on its own provider's selection".
   - **Fix:** skip the recorded account (and the parent inheritance) only when the explicit selection's provider matches the thread's resolved provider. Otherwise, apply both (needs a second override slot), or refuse with a clear message. Add an app-server test for it.

### Low

3. **`--account <other>:default` is refused in exec when no spawned agent can run there** (`provider_accounts.rs:150`). Asking for default credentials is harmless, and `default` is meant to be accepted everywhere. **Fix:** return `None` when the name parses as `default`.

4. **The TUI startup check (`tui/src/lib.rs:1259`) is stricter than the server on resume.**
   - If `[provider_accounts]` names a missing account for the configured provider, `corbanu resume <thread>` now exits. That happens even when the thread would resume on its recorded account or on another provider.
   - `LocalDaemon` targets are checked against the TUI's local config, not the daemon's.
   - **Fix:** skip the check (or only check `--account`) for resume/fork launches, and skip `LocalDaemon` like `Remote`. At minimum, document both limits.

5. **Test gaps.**
   - No positive tests: a configured other-provider account starting a thread, `kimi:default` passing, or the `SubAgent` path skipping the other-provider check.
   - The #427 TUI test sets `chat.thread_provider_account` directly instead of going through `handle_thread_session`.
   - Nothing checks that a recorded `zai:default` beats a named account in the config.
   - No app-server test for `provider_account_recorded` (the flag-off resume message) or for `providerAccount` being filled in both resume paths (`thread_processor.rs:3919`, `thread_lifecycle.rs:718`).

6. **Doc accuracy (`docs/provider-accounts.md`).**
   - Line 10, the precedence line, is false until finding 2 is fixed.
   - Line 26 overclaims: only `--account` and the session provider's account are checked at start. Other providers' `[provider_accounts]` entries are checked only when a child starts on that provider.
   - Line 19: "spawned agents off" should name the actual controls (`agents.enabled`, the `multi_agent`/`multi_agent_v2` features).
   - Line 36: "before any sign-in screen" applies only to the embedded app server.
   - The AGENTS.md documentation rule requires the product-spec heading plus a requirement excerpt, and an opening that states the problem. Both are missing.

### Protocol and schema: no changes needed
- The new `ThreadResumeResponse.provider_account` is experimental with `#[serde(default)]`, matching the fields around it. Reusing the reason string `thread/resume.providerAccount` has precedent (`initialTurnsPage`, `runtimeWorkspaceRoots`).
- The experimental export is regenerated, and the stable schema stays clean.
- The TUI uses the field only when its provider matches the thread's; otherwise it falls back to config. That is correct for forks and provider switches.
- Optional nit: responses aren't filtered per field, so clients that haven't opted into experimental APIs also receive the account name. Sibling fields behave the same way.

### What holds up
- Thread start (`thread_manager.rs:1856`) covers start, resume and fork.
- The model/provider correction path stays fail-closed through `with_account_from`.
- The flag-off recorded-account message is accurate, and its recovery (`--account default`) works.
---

## Dispositions (round 1)

1. **Fixed.** The exec rule now uses the real signal: refused only when `Config::multi_agent_version_override()` is `Disabled` (`agents.enabled = false` without `multi_agent_v2`) or `agents.provider_allowlist` excludes the provider. With both feature flags off, a model's catalog `multi_agent_version` may still enable agents, so that case is accepted. Live P3 now uses `agents.enabled=false`.
2. **Fixed.** `ConfigOverrides.inherited_provider_account` is a second slot for the account a resumed thread recorded or a `thread/spawnAgent` worker's parent runs on. Config load applies it after `[provider_accounts]`, and an explicit account replaces it only for the same provider. Test `inherited_account_is_replaced_only_by_an_explicit_account_of_its_provider`; live R8 (before: default `pong`; after: recorded `fake` 401). Not changed: an unprefixed `--account` on resume is qualified with the launch provider (S03's design, so a thread on another provider cannot reinterpret it); the doc now says to write `<provider>:<name>` for a thread on another provider. A wrongly qualified name is refused if missing, never run on the default.
3. **Fixed.** `<provider>:default` is never refused by the exec rule (unit test, live P4).
4. **Partly.** Tried checking only `--account` on resume launches: the TUI then looped into default-key onboarding (onboarding cannot see the recorded account), the #426 defect. Kept the full check on resume, documented it, and showed the recovery (`--account default` resumes, live T8/T9). `LocalDaemon` stays checked: the implicit daemon serves the same home (its socket lives there).
5. **Mostly added.** Positive/spawned-agent/`default` cases (`other_provider_account_check_accepts_configured_accounts_and_spawned_agents`), the #427 TUI test now goes through `handle_thread_session`, and the config test covers a recorded `default` beating `[provider_accounts]` and the flag-off recorded message through a real `Config`. Not added: an app-server integration test for `providerAccount` on both resume paths; covered by the protocol serialization test, the TUI mapping test and live T4.
6. **Fixed** (precedence, what is checked at start, the exec controls, the TUI/remote limit). The product-spec heading and excerpt are in the PR body and the evidence README; the doc opens with the problem.
