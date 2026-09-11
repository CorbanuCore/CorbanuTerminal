# Feature intent: provider setup, recovery and model selection

Corbanu Terminal is a keyboard-driven AI terminal. Users configure providers
through `/providers`, select a provider/model/effort with `/model`, and continue
their conversation. Users should be able to tell which credentials need attention
and recover that credential without disturbing other configured providers or
restarting the terminal. Saved setup and selections should remain usable in later
sessions. Failure messages should identify the affected service and offer useful
next steps, without presenting credentials in chat.

OpenAI account login, Claude subscription login/managed token and Anthropic API
key are distinct credential routes. Subscription and API billing are not
interchangeable. Other provider API keys and configured custom providers are
supported. A provider can be enabled, disabled, unconfigured, checking, healthy
or need authentication recovery. A selected provider is not necessarily healthy.
The current provider/model/effort should be unambiguous in menus and chat.

The candidate is intended for macOS and Linux, for both new users and existing
profiles. Mac users launch it through an Applications shortcut and use macOS
Keychain; Linux users also run it in tmux over SSH. Normal native security consent
may be needed, but ordinary use should not demand repeated completed setup.
Credentials must remain masked and confined to the intended credential route.
Terminal size may vary. Chat and packaged tools should work after setup/recovery.

The three screenshots are historical user captures of the interface on September
4, 5 and 10 respectively. They are UI context, not a claim about the current
candidate's correctness. There is no supplied expected test count. Prioritize
ordinary use and consequential failures. Identify ambiguities in the intended
behavior rather than inferring requirements from implementation.
