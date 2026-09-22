Corbanu Terminal 0.1.43 lets you replace a configured provider credential from the Providers menu, so a saved invalid token no longer strands a session.

- Configured providers keep their Activate/Deactivate controls and now also list their existing "Replace with …" account or API-key methods.
- Claude Plan uses credential-scoped reauthentication: renewing a known credential source, otherwise starting a full Replace.
- No authorization, vault, or credential-storage boundaries change; masked entry, cancellation, persistence, and cache invalidation are unchanged.

Restart Terminal after updating. This release preserves your existing profile and credentials.

The fix passed focused automated tests (codex-tui provider manager and Claude intent, 11 tests) with refreshed menu snapshots, plus the keyboard-driven terminal recovery checks recorded for the 2026-09-12 provider-replacement hotfix. Hosts that were running the local `0.1.42-provider-replacement` candidate can return to normal `corbanu update` behaviour with this release. [Release evidence](https://github.com/CorbanuCore/CorbanuTerminal/blob/rust-v0.1.43/qa/release/0.1.43/RELEASE.md).

After updating, open `/providers`, pick a configured provider, and confirm a "Replace with …" entry is listed alongside Deactivate.
