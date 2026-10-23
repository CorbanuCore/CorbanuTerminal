**VERDICT: APPROVE**

I found no blocking issues. Both fixes look correct, and the /providers text fields can't be quit with a printable key. I only read the code; I didn't build or run any tests.

1. **NON-BLOCKING: `q` can still quit while a save is in progress.** The text-field check (auth.rs:552-577) covers all five text states, and the match lists every state with no catch-all, so the compiler forces a decision for any new state. The handlers at auth.rs:1960, auth.rs:1666 and named_account.rs:98 take in every printable key while a field has focus. But the three saving states (`ApiKeySaving`, `ClaudeManagedTokenSaving`, `AccountSaving`, auth.rs:565/568/574) still let a plain `q` quit. So if a non-bracketed paste contains a newline (the Enter starts the save) and a later `q`, the TUI exits. Fix: treat those three short-lived states as absorbing printable keys too.

2. **NON-BLOCKING: the stdin flush on exit is safe, but it can't catch everything.**
   - Safe: it only runs when stdin is a terminal (tui.rs:371-375). Every exit path I checked goes through it: the drop guard, `lib.rs:2068 restore()` and the panic hook (tui.rs:625). Windows uses `FlushConsoleInputBuffer`. The background-job signal risk is no worse than the terminal reset that already runs just before it.
   - Gap: bytes that arrive after the flush, such as a paste the terminal sends in chunks, can still reach the shell. Suspend (`job_control.rs:202`) doesn't flush, which is fine because it isn't an exit.
   - Real protection comes from item 1 plus the routing fix. State that in the PR rather than implying the flush guarantees nothing leaks.

3. **NON-BLOCKING: a pre-existing provider fallback now finishes without any warning.** When config names a provider that has no API-key option, `.unwrap_or(0)` (onboarding_screen.rs:229-237) picks the first option instead. `begin_initial_api_key_entry` and `SignInOption::ApiKey` (auth.rs:711-716, 761-779) now start setup for that first option, which matches where `save_api_key` writes the key (auth.rs:2096). So nothing loops and nothing starts twice; a second Begin does nothing because Begin only applies from the provider list (provider_setup.rs:180). The OpenAI, ChatGPT and picker paths are unchanged. The problem: when the provider has an `env_key` and doesn't require OpenAI auth, the screen still shows the configured provider's name (onboarding_screen.rs:242-248). The key is saved to the first option, and the flow now ends as if it worked. Fix: drop the `unwrap_or(0)` and set the selection to `None`, or skip Begin when the selected provider id differs from `api_key_provider_id`.

4. **NON-BLOCKING: the #462 unit test doesn't cover the actual call sites.** `preset_provider_key_entry_completes_setup_for_that_provider` (onboarding_screen.rs:1229) calls the helper directly. Deleting the call at onboarding_screen.rs:322 or reverting auth.rs:713 would still pass. Only the tmux test catches that, and it is skipped unless `TmuxServer::should_run` allows it. Add a unit test that goes through `handle_sign_in_option(SignInOption::ApiKey)` with a non-OpenAI selected option.

5. **NON-BLOCKING: /status change is correct.**
   - card.rs:741-750: the OpenAI wording now appears only when `uses_first_party_openai_auth()` is true.
   - Providers with `auth` or `aws` set return before this match (card.rs:717-722), which covers Bedrock. Claude-plan should fall under the `auth` case, though I didn't confirm that.
   - Custom `env_key` providers whose key is in the vault now show "<name> API key configured", which matches what the app server reports at account_processor.rs:1118-1133.
   - The changed assertion in status/tests.rs would fail on the old code.

6. **NON-BLOCKING: tests fail on the old code, with minor flakiness risk.**
   - The #461 unit test fails on the old code: an empty key field and both named-account fields would have quit on the first `q`.
   - The tmux #461 test fails on the old code too, because the key starts with `q`. The #462 tmux test uses `pf53-loopback`, which is in the provider list, so it runs the intended path.
   - Flakiness: the masked-value wait (multi_provider_onboarding.rs:681) counts `'•' >= 8`. Bullets elsewhere on screen, or a partly typed value, could pass it early. Wait for `count >= account_key.chars().count()` instead.

7. **NON-BLOCKING: no secret leaks found.**
   - Key fields are fully masked (auth.rs:196).
   - `text_entry_value` exists only in test builds.
   - The demo spec uses fake keys.
   - The tmux tests scan scrollback, and every file in the home directory except credential storage (log included), for the canary values.