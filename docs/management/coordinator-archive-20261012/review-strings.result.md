**VERDICT: APPROVE**

The commit does what it set out to do. Every changed line is a user-facing approval, auto-reviewer or Full Access string. I could not run the tests because the sandbox is read-only. Insta updated the `assertion_line` values in the snapshots, which suggests the author ran and accepted them; I also checked the new wrapping by hand.

1. **The changed strings are correct.**
   - **Approval lines:** all of them in `history_cell/approvals.rs` now say Corbanu. That covers run, network access, always-run, session, persisted, denied, did not approve, timed out, canceled, and patch denied/timed out.
   - **Auto-reviewer and Full Access:** the auto-reviewer MCP, network and permissions lines in `chatwidget/tool_requests.rs` and the Full Access text in `chatwidget/permission_popups.rs` also say Corbanu.
   - **English:** every resulting sentence reads correctly. "Codex" was capitalized in the old persisted line and lowercase elsewhere; all of them are now "Corbanu".

2. **Nothing else was renamed.** No protocol fields, ids, config keys, metric names like `codex.windows_sandbox.*`, crate paths or logs changed. `codex_home` and the `codex_approval_kind` keys are untouched.

3. **The snapshot and test changes match the string change.**
   - **Width 28 test in `approval_overlay.rs`:** the line break moved and the result grew from 4 lines to 5. I checked the line lengths and the new wrap is correct.
   - **Session-allowance and Full Access snapshots:** they re-wrap as expected from the longer word.
   - **Other snapshots:** only the word changed.

4. **Small nit:** the snapshots also picked up `assertion_line:` metadata, and one header changed from `source: tui/src/chatwidget/tests.rs` to `tui/src/chatwidget/tests/approval_requests.rs`. Insta ignores this metadata when comparing, so it's harmless, just extra diff.

5. **No user-facing "codex"/"Codex" is left in the approval or auto-reviewer files you named** (`approval_overlay.rs`, `approvals.rs`, `tool_requests.rs`, `permission_popups.rs`, `permissions_menu.rs`, `app/permission_confirmation.rs`, `auto_review_denials.rs`, `approval_events.rs`). Nearby permission strings that still say Codex, which you could do as a follow-up:
   - `codex-rs/tui/src/history_cell/session.rs:184`: `"/permissions - choose what Codex is allowed to do"` (the session-header hint for the permissions command).
   - `codex-rs/tui/src/chatwidget/windows_sandbox_prompts.rs:246`: "Set up the Codex agent sandbox…"
   - `codex-rs/tui/src/chatwidget/windows_sandbox_prompts.rs:250`: "…requires the default Codex agent sandbox…"
   - `codex-rs/tui/src/chatwidget/windows_sandbox_prompts.rs:406`: "Use Codex with non-admin sandbox"
   - `codex-rs/tui/src/app/event_dispatch.rs:4425` and `:4457`: "Codex can now safely edit files and execute commands in your computer". This one also has a grammar issue: "in your computer" should be "on your computer".

6. **Test-only leftover (optional):** `codex-rs/tui/src/history_cell/tests.rs:1055` and `:1064` build their own `" codex to run "` line to test wrapping. No production code feeds it, so it isn't wrong, but changing it to Corbanu would keep the example consistent.