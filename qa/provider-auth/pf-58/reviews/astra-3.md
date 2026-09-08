autoreview findings: 1
[P2] Make the advertised OpenAI reauthentication action start sign-in
codex-rs/tui/src/chatwidget/provider_recovery.rs:58
With a saved OpenAI account still marked Configured, `/providers` → OpenAI → `r` offers “Sign in to OpenAI again”, but selecting it immediately returns to management without starting login. The handler passes the configured status to `start_openai_account`, whose `ManagedAccount` branch completes immediately without emitting `StartLogin`. This prevents manual reauthentication when a rejection has not been observed in the current session. Either route this action through explicit reauthentication or restrict the menu item to statuses where it actually starts sign-in.

overall: patch is incorrect (0.94)
The new recovery menu exposes an OpenAI sign-in action that silently completes without authenticating for configured accounts. Review used read-only inspection; tests were not run and pending TMUX qualification was not assumed to pass.
