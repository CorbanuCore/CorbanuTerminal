## Named accounts (behind `named_accounts`)

- **App servers in homes with only the default login no longer wait for a
  conversation.** With named accounts on, a standalone app server or daemon
  held back its own ChatGPT requests (apps, plugins, rate limits, the model
  catalog) until a conversation started, even in homes without named
  accounts. It now does that only when the home has a named OpenAI sign-in
  account or `[provider_accounts]` selects one. A server started without one
  starts holding them from the first conversation on an account added while it
  runs, and never goes back to the default login (#1085).
- **`/status` names the account only where named accounts exist.** With named
  accounts on, the provider line read "OpenAI · account default" (or
  "Z.AI · account default") in every home. It now names the account only when
  that provider has a named account in the home, or the session is on or used
  one; otherwise the line is what it was without the feature (D86).
- **`/status` no longer reads the vault itself.** The summary of a named
  ChatGPT sign-in account (kind, email hint, plan) is now read in the
  background and cached. Until it is known, `/status` shows just
  `account <name>` and doesn't treat the account as removed. A sign-in change
  made by another process shows on the next `/status` (each one starts a
  refresh) (#1572, #1251).
