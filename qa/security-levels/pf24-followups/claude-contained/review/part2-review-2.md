Verdict: approve with fixes

All nine round-1 findings are fixed. I found nothing High, and no path where a single stray key, a held key, a paste or a dropped popup allows a tool. What remains is narrower: a few ways a deliberate-looking key sequence can land on the wrong popup or on unseen content, and one conditional way to run code outside the sandbox through the new version check.

I only read code and the recorded evidence; I didn't build, run tests or run Claude Code.

### Round-1 findings
1. **Enter allows by default — fixed.** The popup opens on Deny. Allow needs ←/→/Tab and then an Enter press, after a 600 ms guard that starts at first draw; a never-drawn popup ignores keys. Esc and Ctrl-C deny, paste is ignored, and a dropped popup denies. Tests cover these.
2. **Can't see what's approved — fixed.** Every field is shown, wrapped and scrollable, with hidden characters escaped. `updatedInput` is the same parsed `Value` the person was shown. Past 60,000 characters only Deny is offered, and `finish` enforces that too (`claude_approval_view.rs:95`).
3. **Unverified routes — fixed.** In the regression runs, Bash goes from not asking (old flags) to asking (new flags), so `--settings` still applies alongside `--safe-mode`. Planted agents, skills, commands, hooks, MCP servers and `.claude.json` had no effect, and the macOS tmux run shows the real flags working inside Seatbelt.
4. **Wording overclaim — fixed.** The text is accurate now, given the `ask` rules.
5. **Stale popups and leaked waits — fixed.** The token is a child of the turn's token, a drop guard cancels it on early exit, and it is also cancelled after the read loop. A cancel drops the receiver, so `is_settled` is true and the popup is removed or never shown. The 15-minute timeout denies, the writer task ends once waiters finish, and a missing `request_id` or a second `result` ends the turn.
6. **Request IDs — fixed.** Repeated IDs get no popup and no answer, and `tool_use_id` is shown.
7, 8, 9. **Fixed.**

### Findings
1. **Medium — typing can still allow.** The 600 ms guard is the only defence, and it doesn't restart while the person keeps typing (`claude_approval_view.rs:88-92, 113-133`). The popup opens immediately (`interaction.rs:237-249`), unlike Codex approvals, which wait for typing to pause (`bottom_pane/mod.rs` `approval_prompt_delay_remaining`, `require_fresh_choice`).
   - Someone editing a draft who presses Tab (completion) or ←, and then Enter, more than 600 ms after an unnoticed popup appeared, allows the tool.
   - Keys typed into a terminal without bracketed paste arrive as separate keys, so pasted text with a tab followed later by a newline allows it.
   - **Fix:**
     - Count the guard from whichever is later: the first draw, or the last composer keystroke plus `APPROVAL_PROMPT_TYPING_IDLE_DELAY`.
     - Restart the guard on every key it ignores and on terminal focus regained.
     - Don't toggle on Tab, since the composer uses it.
     - Add a test: Tab and Enter typed continuously across the guard boundary must not allow.

2. **Low–Medium — a new popup covers the one being answered** (`bottom_pane/mod.rs:1465` → `push_view`).
   - Request B appears over A, which looks almost the same. A Right and Enter meant for A, pressed after B's 600 ms, can allow B.
   - When B closes, A comes back unguarded, with whatever choice it had (possibly Allow). A quick second Enter then confirms A. `remove_views_settled_elsewhere` can uncover A the same way.
   - **Fix:** queue Claude requests behind the open popup (first in, first out, like `ApprovalOverlay`'s queue) instead of stacking them. Or, whenever a popup becomes the top view again, reset it to Deny and restart its guard. Add a BottomPane-level test for two requests.

3. **Medium (only with certain PATH settings) — the version check can run a planted `claude` outside the sandbox** (`execution.rs:727-770`). `Command::new("claude")` looks the name up in PATH from Corbanu's own working directory, which is often the same repository as the pane folder.
   - If PATH has an empty entry (a stray `::` or trailing `:`), `.`, or a relative entry such as `node_modules/.bin`, a `claude` file from an untrusted clone runs with no sandbox. So does one written by an earlier contained turn, since the pane folder is writable; the cache of passed checks lives only for one Corbanu process.
   - The contained launch looks the name up separately, inside the sandbox, from the pane folder. So the binary that was checked may not be the binary that runs, and a planted one skips the approval prompts (though the sandbox still holds).
   - **Fix:** look up an absolute path once and refuse it if:
     - it came from a relative or empty PATH entry, or
     - it sits under the pane folder, the state folder or any other writable root.

     Use that same path for the check and the launch, and cache the result by canonical path and file identity, not by the name "claude". Run `--version` inside the same sandbox profile, or read the version without running anything.

4. **Low — the person can allow before seeing the details.** On a short terminal, `detail_rows` can be 0 while Allow still works (`claude_approval_view.rs:307-313`). More generally, Allow works after seeing only lines 1–16 of a 400-line Write.
   - **Fix:** turn Allow on only after the last detail row has been on screen at least once, and never when `detail_rows == 0`. Add a test.

5. **Low — the deny list breaks on newer Claude Code versions** (`command_plan.rs:517`). The check only sets a minimum version, so a newer build (the user's own `claude` updates itself) may add tools that start agents and aren't on the list.
   - **Fix:** list the allowed tools with `--tools` (Bash, Read, Edit, Write, NotebookEdit, WebFetch, WebSearch, TodoWrite and so on) instead of blocking named ones. Warn, or rerun the regression, when the version is newer than the one tested.

6. **Low — prompts starting with `/` aren't covered.** Built-in skills and commands are still listed (`/batch`, `/loop`, `/workflow-launch-exec`, `/update-config`…), and prompts can come from automated whips into Claude panes (`orchestrate.rs:3517`), not only the person. Tool-wide `ask` rules should still beat a skill's `allowed-tools`, but nothing tests that.
   - **Fix:** add regression cases that send such prompts and check that every Bash or Edit still asks. Otherwise, escape or refuse a leading `/` in contained panes.

7. **Low — some text isn't counted toward the 60,000 limit** (`approval.rs:289-313`). Field names, `edit N:` labels and empty values cost nothing. Total rows are capped at `u16::MAX` (`claude_approval_view.rs:217`), so in extreme inputs or very narrow widths the end can't be scrolled to, yet Allow stays on. This isn't practical within a model's output limits.
   - **Fix:** count names, labels and one character per line toward the limit. Count rows as `usize` and turn off Allow above `u16::MAX`.

8. **Low/Info — the display can mislead in edge cases** (`approval.rs:237-266, 380-409`).
   - Edit and MultiEdit mark `old_string`/`new_string`/`edits` as shown even when the value isn't text, which then shows as empty. Claude Code's input check should block this, but the field should be shown either way: only mark it shown when it really is text.
   - Some invisible characters aren't escaped, for example U+180B–180D and U+1BCA0–1BCA3. Escape every character that takes up no width, plus the character classes for format, line and paragraph separators, non-ASCII spaces, and private-use or unassigned code points.
   - Text values aren't quoted, so `"true"` looks like `true`. A long value can wrap so its next row looks like a new `name:` line. Quote or indent values.

9. **Info — whole-stack clears answer a buried popup as Deny.** `view_stack.clear()` on paste or tick completion (`bottom_pane/mod.rs:757, 797`) and `show_shutdown_in_progress` drop a buried Claude popup, which counts as "the person denied". That's safe, but the message isn't accurate. Consider removing only the top view.

### Test gaps
- No BottomPane or ChatWidget test for two popups, keys typed in the composer just before a popup, or a short terminal.
- Regression gaps:
  - It ran on Linux only, with the provider URL also set in the process environment.
  - It doesn't cover `--resume` turns or prompts starting with `/`.
  - Deny rules and `--disallowedTools` are never tested separately.