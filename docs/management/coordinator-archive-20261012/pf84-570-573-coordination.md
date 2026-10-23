# PF-84 #570–#573 worker → pf84-fix6 (switch-follow): overlap note (2026-10-11)

#571 root cause (reproduced on origin/main with a mock, glitch): a named OpenAI ChatGPT account's
AuthManager copies `forced_login_method` from the session's default manager, and `shared_from_config`
fills that with the EFFECTIVE value, which a third-party session provider (Z.AI, Kimi, ...) sets to
`api` for onboarding. The named ChatGPT sign-in is then refused ("holds a ChatGPT sign-in, but this
configuration requires an API key"), auth is None and the request goes out with no Authorization.
It is not a switch-follow problem; it happens with or without ActiveAuthManager.

Fix (branch `fix/pf84-named-login-restrictions`, PR to follow): `AuthManagerConfig::configured_forced_login_method`
(Config → `configured_forced_login_method`) feeds the manager; core `sign_in_restriction_error`, `corbanu login
--account` and `/providers` add-methods use the configured value; a named sign-in provider with no usable sign-in
is refused (`CodexErr::Fatal` with the reason) instead of being sent unauthenticated.

Files I touch that you also touch: none of your 55 files except possibly `tui/src/app/provider_accounts.rs`
(one line in `provider_account_methods`) and `core/src/thread_manager.rs` (one added check after
`thread_start_account_error`). #572 (separate PR, `fix/572-bedrock-named-model`) adds
`ModelCatalog::include_provider_models` in `tui/src/model_catalog.rs` (new method, no change to yours).
