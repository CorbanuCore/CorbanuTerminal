# Legacy settings-notification timeout diagnosis

The repeated failure is retained, not classified as timing flakiness.
`focused-final-2.log` records two failed attempts; both diagnostic logs retain
their two failed attempts. No timeout or assertion was weakened.

`tui-settings-diagnostic-2.log` exposes the actual `confirm=false` RPC failure:
code -32600, incompatible explicit model/provider pair `gpt-5.4` + `ambient`.
Configuration validation rejects before Core submission. The App's existing
best-effort settings route displays an error and returns handled; the original
test ignores that error and waits for a settings notification. ThreadStarted is
observed, but no settings operation was queued. This is not evidence of a lost
subscription or deduplicated settings notification.

Read-only base comparison against `005cc644f59b1e762e5497b329e106c67925d4ed`:

| Source | Byte-for-byte base parity | SHA256 |
| --- | --- | --- |
| `codex-rs/core/src/config/mod.rs` | yes | `99c373ec284cde23447844b6b715f013a2723bdc7a3e9bff8ee388b24df28944` |
| `codex-rs/tui/src/chatwidget/tests/helpers.rs` | yes | `bf1edf5694dbfa36759121c4f52f2bd067555afef8deece610c5f23bbb5f9fd7` |
| `codex-rs/tui/src/app/thread_settings.rs` | yes | `6e8278129f6f47d2229914400aa20b27909a92f206f6d1554a533ed3e9f80d39` |

The entire `build_thread_settings_overrides` method also matches base after
excluding comment-only lines (SHA256
`24e41045ac8e3058f1af2062631bda88abbc14b212c93eb8757ce4ae2f8c053c`).
The original test at base uses the default model/provider fixture and requests
GPT without changing provider. The PF83 confirm branch occurs only after this
shared validation and is not taken for this request.

Correction is confined to the existing allocated test: start with the existing
manual widget fixture for `gpt-5.3-codex`/OpenAI, then retain the original update
to `gpt-5.4` and all settings-notification/cache assertions. No runtime provider,
authorization, subscription, or persistence policy changed. No inference is
requested by this test. Final paired corrected-fixture execution is recorded in
`focused-final-4.log` (`focused-final-3.log` stopped at a corrected import error).

Evidence limit: this is current-tree runtime diagnosis plus exact base-source
comparison, not an isolated baseline executable run. That distinction remains
available to independent review; the original failed attempts are not passes.
