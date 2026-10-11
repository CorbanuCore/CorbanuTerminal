<!-- Reviewer: installed corbanu exec, -m claude-opus-5-5-plan, model_provider=claude-plan, reasoning high, -s read-only, on 0c74b5d37b. Output saved verbatim below; dispositions follow. -->

The fix closes #414 for the session/exec model path, but two TUI paths still break the "never another account" rule: Claude panes and the native-worker pre-check. **Request changes.** I couldn't run builds or tests here because the sandbox is read-only, so these findings come from reading the diff only.

1. **High (#414 not covered in panes)** — `tui/src/claude_panes/execution.rs:893-907` and `command_plan.rs:262-270`: Claude panes still run `internal-claude-oauth-token` with no `--account`, and nothing in `claude_panes` reads the account selected for `claude-plan`. If `[provider_accounts] claude-plan = "work"` is set, a pane quietly gets the **default** account's token, even with the current build as the helper. #417 makes this easier to hit: the status host now reports Claude Plan as configured from the named account alone, so panes become available on that basis.
   - Fix: carry the selected `NamedProviderAccount` into `DeferredClaudePlanAuth` and pass `--account <name> --enable named_accounts`. Until then, refuse a pane when an account is selected.
   - Also call `env_remove(CORBANU_PROVIDER_ACCOUNT)` when no account is selected, because the helper reads that variable as a fallback.
   - Add a regression test using the existing fake-helper setup in `claude_panes/tests.rs:2605`.

2. **Medium (worker pre-check checks the wrong account)** — `tui/src/spawn_orchestration.rs:2383`: `validate_provider_auth_command(auth)` runs the helper with no account, which reads the default credential.
   - If the default account exists, the check passes even when the named account is missing or the helper on PATH is outdated. The worker still refuses later, so this isn't a token leak, but the check proves the wrong thing and reads a secret it doesn't need.
   - In a home that only has named accounts (the #417 case), it wrongly blocks the native worker with a misleading message.
   - Fix: make `model-provider::auth::external_bearer_account` public (or add a login helper) and call `validate_provider_auth_command_for_account(auth, Some(&account))` with `registry_home = config.codex_home`.

3. **Low (fragile outdated-helper check)** — `login/src/auth/external_bearer.rs:326`: `stderr.contains("--account")` matches any mention of `--account`. A false match only gives a misleading "predates named accounts" message, and every failure path still refuses, so this is cosmetic. Match clap's actual text, `unexpected argument '--account'`, instead.

4. **Low (argv added based on display name)** — `external_bearer.rs:275` with `ExternalBearerAccountServer::ClaudePlanHelper`: this is chosen by `is_claude_plan()`, which compares `name`. If a user overrides the `claude-plan` auth command, the flags get appended to their own command. Only use `ClaudePlanHelper` when `args.first() == "internal-claude-oauth-token"`; otherwise treat it as `Command`.

5. **Low (deferred config warnings can be lost)** — `tui/src/chatwidget/turn_runtime.rs:551`: config warnings are now held until a session starts. If the thread never starts (start error, early exit), they are never shown, which is a change from today even with `named_accounts` off.
   - Flush the pending list when the thread fails to start, or on exit.
   - Separately, `should_display` removes a summary from the shown-warnings set the first time it matches. A summary that is never repeated stays in the set and hides the next identical warning, even an unrelated later one.

6. **Low (#418 drops non-UTF-8 paths)** — `cli/src/pfterminal_home.rs:28`: switching from `var_os` to `std::env::var(...).ok()` silently ignores a non-UTF-8 `CORBANU_DEBUG_HOME`/`CORBANU_HOME` on Linux and falls back to another home. Keep `var_os` and use `OsStr` comparisons, converting to `&str` only for the warning text.

7. **Low (#419 only covers command accounts)** — `model-provider/src/models_endpoint.rs:87`: the "no credential, no request" guard requires `provider_info.auth.is_some()`. A named **API-key** account with no stored key can still send an unauthenticated `GET /models`. Drop that condition so the guard applies to any named account.

8. **Low (missing home reported as "not configured")** — `model-provider/src/provider.rs:353`: when there is no auth manager, Claude Plan named accounts are now refused as "not configured". Before, the helper checked its own accounts. Failing closed is fine, but say "no Corbanu home available to verify account" instead.

9. **Low (heavy config load in the token helper)** — `cli/src/main.rs:2451`: `ensure_named_accounts_enabled` builds a full `Config`, including GPU providers from sqlite and project layers from the helper's cwd, on every named-token fetch. Token caching limits how often this runs, but reading only the feature flags would be cheaper.

10. **Nit (wording)** — `cli/src/account_cmd.rs:186`: the message shows the internal kind name (`claude_oauth_token`). Show the CLI value instead (`--kind claude-token`).

11. **Nit (recovery hint)** — `tui/src/chatwidget/provider_health.rs:136`: for a `claude-config-dir` account the hint still suggests `--kind claude-token`. Suggest re-running `claude auth login` with that config directory instead.

12. **Nit (evidence gap)** — the README tests a pre-PF-84 helper on PATH, but not an S02-era one. My reading is that an S02-era helper accepts `--account` and the global `--enable`, then serves the named account, so the rule holds. Record that run anyway, since it is the more common version mismatch.

**Points that check out:**
- **#414 model path:** argv plus the enrolled-account check run before the helper starts. An old helper's refusal becomes a fatal error, and a missing or unreadable account registry is refused.
- **`named_accounts` off:** behaviour matches today (no account means unchanged argv and env), apart from finding 5.
- **Secrets:** argv carries only the validated account name (`[a-z0-9-_.]`), so there is no shell-injection risk through Windows `.cmd` wrappers either. Tokens go only through stdout and are zeroized.
- **Regression tests:** the tests for #414, #415, #416, #417 and #419.2/.3 would fail on the code before the fix. The #414 test is Unix-only.

**Verdict: REQUEST CHANGES** (findings 1 and 2).
---

## Dispositions

1. **Fixed.** Claude panes pass the selected `claude-plan` account to the helper (`--account <name> --enable named_accounts`, plus `CORBANU_PROVIDER_ACCOUNT`), and remove that variable when no account is selected. Regression test: `claude_panes::tests::claude_plan_pane_passes_its_named_account_to_the_helper`.
2. **Left to PF-84-S03 (#421), which already changes this exact call.** It adds `selected_account_error` and calls `validate_provider_auth_command_for_account(auth, Some(name))`. To keep #421 compatible, this PR keeps that function's `Option<&str>` signature. Because the helper argv is now derived from the command's own args, #421's pre-check also passes `--account` to the Claude helper once both PRs are merged.
3. **Fixed.** The check now matches clap's `unexpected argument '--account'`.
4. **Fixed.** The account goes into argv only when the auth command's first arg is `internal-claude-oauth-token` (`is_claude_plan_token_helper`). The enrollment kinds follow the same rule.
5. **Partly fixed.** Deferred config warnings are also shown before any error message, so a thread that fails to start no longer swallows them. Kept as is: a TUI-only config warning that the session never repeats stays in the set, so it only suppresses a later identical warning, which would be a duplicate anyway.
6. **Fixed.** `corbanu-debug` reads the home variables with `var_os`; the warning text is converted lossily.
7. **Not changed.** For API-key providers the `/models` auth comes from the provider key path, not `auth()`, so the guard would block a valid named API-key account. The acceptance run found 0 requests for an unenrolled API-key account (C4).
8. **Fixed.** The message now reads "cannot be verified: no Corbanu home is available".
9. **Accepted.** The config is loaded only for a named account, and tokens are cached, so the cost is bounded.
10. **Fixed.** The message shows the CLI value: "does not take `--kind claude-token` accounts".
11. **Fixed.** The Claude hint also says to sign in again in the account's Claude Code config directory.
12. **Done.** The S02-era helper (`origin/main` on PATH) serves the named account: `fake` gets 401 and `real` answers `pong` ([414](captures/414-claude-plan-path-skew.txt)).
