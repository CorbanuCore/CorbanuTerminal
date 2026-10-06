**Verdict: approve.** I found nothing that blocks this commit. The four items below are optional polish or process notes. This was a read-only review; I didn't run tests or the demo.

### What checks out

- **Context is installed before the popup can appear.** There is only one launch path: `run_main` calls `security_launch.finish(&mut config)` at `lib.rs:1235`, and `run_ratatui_app` is called later, at `lib.rs:1403`. Onboarding, resume/fork and the remote/daemon target are all handled after `finish`, which always calls `install_context`. If the launch fails, the process exits.
  - A stored Aggressive level on a remote server is refused earlier, at `lib.rs:1065`.
  - Unit tests and paths with no context get `None`, so they show the read-only description, which is the right default.
- **The description and the command can't disagree.** `security_description` and `SecurityView::new` (`bottom_pane/security_view.rs:47`) both read the same `level::context()` and filter on `picker_enabled`. That context is set once at launch, so turning the flag on with `/experimental` or reloading config changes neither of them.
- **No other place shows this text.** Only `command_popup.rs:208/258` calls `SlashCommand::description()`. No docs, help text or `insta` snapshot contains the old wording. The only hits are historical PF-83 evidence logs, which should stay as they are.
- **The wording is accurate.** It doesn't promise Moderate, which the picker shows as "Moderate is not available yet." It also matches the picker's "takes effect at the next start" behaviour.

### Findings

1. **Low: wording differs from the picker.** `slash_command.rs:336`. The picker's notes say "Takes effect when you restart Corbanu Terminal" (`security_level_picker.rs:154,222,242`), while the popup says "takes effect at the next start".
   - **Fix:** use "choose a security level (applies after restart)", or match the picker's "restart" wording. Update the test and `qa/demos/specs/pf24-security-description.toml:26` to match.

2. **Low: no snapshot covers this text.** `codex-rs/tui/AGENTS.md` requires `insta` coverage for user-visible UI changes, but the new test only uses `assert_eq!` on strings. The existing `command_popup_default_items` snapshot runs only on macOS, is already stale (it still says "Codex" and has no `/gpu` or `/security`), and never renders this row.
   - **Fix:** either record in the PR that the snapshot is skipped because the context is a process-wide `OnceLock`, or pass the context into the popup so a snapshot can render both descriptions. I'd accept the note for this bounded fix.

3. **Nit: test style.** `slash_command.rs:393-411`. The test has function-local `use` statements and calls `super::security_description`, while the rest of the module imports at the top (`use super::SlashCommand;`). The read-only string is also copied by hand into the test.
   - **Fix:** add `use super::security_description;` and `use crate::security::level::{ChosenLevel, LevelContext};` at the top of the module, one item per line to match the file. Optionally move the two strings into private `const`s shared by the code and the test, so the test checks the branching rather than repeating the literals. Using `/*context*/ None` and comparing whole arrays already follows `codex-rs/AGENTS.md`.

4. **Info (process): the change record is missing.** This is a bounded fix, so the root `AGENTS.md` requires a product-spec heading plus a short excerpt in the issue, PR or release record. The commit message has neither.
   - **Fix:** in the PR body, cite **P0 `/security` levels**, for example the excerpt "`/security` opens one focused tab showing the current security level…". Also record the TUI run: the demo spec covers flag-on and the unit test covers flag-off, but the PTY evidence and video row aren't in this commit.