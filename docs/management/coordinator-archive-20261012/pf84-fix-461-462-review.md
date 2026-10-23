You are an independent, read-only code reviewer (do not edit files, do not run builds or tests, do not run the TUI).

Repository worktree: /Volumes/CorbanuDrive/Corbanu/worktrees/pf84-fix-461-462-20261011 (branch fix/pf84-461-462-onboarding-text-fields).
Review the change with: `git -C <worktree> diff origin/main...HEAD` (one commit, cc4f800989 or later on that branch).

The change fixes two GitHub issues in Corbanu Terminal's Rust TUI (codex-rs/tui):
- #461 (security-relevant): in first-run onboarding's "Add another account" (named accounts, PF-84-S04), typing `q` in
  the account-name or masked key/token field quit the TUI; the rest of a typed or non-bracketed-pasted secret then
  landed on the user's shell in clear. Required: every onboarding text field consumes all printable keys (no global
  shortcuts while a text field has focus), across all masked-entry/text fields in onboarding and /providers; nothing
  typed may reach the shell after exit. The old code deliberately let `q` quit from an EMPTY API-key field; the new
  code removes that exception (a value starting with `q` would otherwise leak).
- #462: (a) /status showed "OpenAI API key configured (run corbanu login ...)" in a home whose only credential is a
  Z.AI key; (b) onboarding looped back to key entry when config.toml already named an API-key provider (e.g.
  model_provider = "kimi-code", which forces the API login path): the provider-setup state machine began setup for
  "openai" (or not at all for the initial key-entry screen), so the saved key's AuthConfigured completion was ignored.

Key files: tui/src/onboarding/onboarding_screen.rs (quit routing), tui/src/onboarding/auth.rs (is_text_entry_active,
api_key_entry_provider_id, begin_initial_api_key_entry), tui/src/onboarding/auth/named_account.rs (context, unchanged),
tui/src/onboarding/provider_setup.rs (state machine, unchanged), tui/src/tui.rs (stdin flush on exit),
tui/src/status/card.rs, tests in onboarding_screen.rs, status/tests.rs, tui/tests/suite/multi_provider_onboarding.rs.
For the /providers audit, the relevant views are bottom_pane/vault_secret_entry.rs, bottom_pane/custom_prompt_view.rs,
and the key routing in app/input.rs, chatwidget/interaction.rs and bottom_pane/mod.rs.

Please check, with file:line evidence:
1. Correctness and completeness of #461: any onboarding or /providers text/masked field where a printable key can
   still trigger a shortcut or exit; any other pre-chat text input that routes keys through a global quit; whether the
   exhaustive SignInState match is right; whether flushing stdin in restore_after_exit is safe (non-tty, Windows,
   job control, other exit paths that skip it).
2. Correctness of #462: can begin_provider now be called for the wrong provider (e.g. selected_api_key_option falls
   back to index 0 when config's provider is not in the options), double Begin, or regress the OpenAI API-key / ChatGPT
   / picker paths? Is the /status change right for all provider kinds (OpenAI with requires_openai_auth, custom
   env_key providers, claude-plan, Bedrock)?
3. Test quality: do the new unit and tmux tests fail on the old code and pass on the new? Flakiness risks?
4. Anything that leaks secrets (logs, test artifacts, screen).

Output: a verdict line `VERDICT: APPROVE` or `VERDICT: REQUEST CHANGES`, then numbered findings, each tagged
BLOCKING or NON-BLOCKING, with file:line and a concrete fix. Keep it under 600 words.
