Verdict: approve with fixes

I only read the code; I didn't run anything. Fixes 1 and 3 have gaps (findings 1 and 2). Finding 2 is the one that can still run code outside the sandbox. Fixes 4–8 are real. I checked the `seen_end` math, the u16/usize row handling, the line-break and label counting, the queue's first-in-first-out order, `remove_settled_requests` restarting the request at the front, and the leading space added to `/` prompts. Fix 9 still fails closed. Fix 2 holds for every path that goes through `pop_active_view_with_completion`, `dismiss_view_by_id` or `remove_views_settled_elsewhere`, but not for the paths in finding 1. I found nothing High.

1. **Medium-Low: two paths uncover the popup without resetting it** (fix 2 is incomplete). Paths: `bottom_pane/mod.rs:1711-1714` (`dismiss_app_server_request`) and `mod.rs:1302-1314` (`dismiss_active_view_if_id`).
   - Both remove the top view without calling `on_top_view_removed`.
   - A Codex approval, user-input or MCP elicitation overlay can be pushed over the Claude popup, because `push_approval_request` and the others only check `last_mut()`.
   - When the server resolves that overlay elsewhere, the Claude popup comes back unchanged: still on Allow if the person chose it, end already seen, guard expired. An Enter meant for the vanished overlay then allows the Claude tool.
   - **Fix:** call `on_top_view_removed()` whenever the removed index was the top, in both functions. More robust: track the top view's identity (a view counter or the `Box` address) in BottomPane, and call `on_uncovered` from `pre_draw_tick` whenever it changes, so future removal paths can't miss it.
   - **Test:** Claude popup on Allow, push an `ApprovalOverlay`, call `dismiss_app_server_request`, then assert the popup is back on Deny with `shown_at == None`.

2. **Medium (only with certain PATH settings): the version check still runs code outside the sandbox** (fix 3 is incomplete). Locations: `claude_panes/execution.rs:818-826` and `execution.rs:86`.
   - **Interpreter lookup:** the check passes the raw `PATH` and inherits Corbanu's working directory. An npm-installed `claude` (`#!/usr/bin/env node`, the common install) looks `node` up in that PATH. A `.`, empty or relative entry can therefore run a planted `node` from the repository with no sandbox.
   - **Other panes' folders:** only this pane's own folder and state folder are refused. Say PATH has an absolute entry inside another contained pane's folder, such as direnv's `/repo/node_modules/.bin` while this pane works in a different worktree. A `claude` written there by that pane's earlier turn is accepted and run outside the sandbox.
   - **Fix (preferred):** run `--version` through `contain()` with the same profile and no network, so nothing from PATH ever runs outside the sandbox.
   - **Minimum fix:**
     - give the check a PATH with only absolute entries outside every writable root;
     - set `current_dir("/")`;
     - refuse anything under `containment.panes_dir` or under the working folder of any contained pane.

3. **Low: ← then Enter can still allow after an unnoticed pause.** Locations: `claude_approval_view.rs:174-186` and `mod.rs:691-701`.
   - The composer counts only characters, Backspace, Delete, Enter and Tab as typing; arrow keys don't count.
   - Someone editing a draft who pauses for 600 ms, then presses ← (to move the cursor) and Enter (to send) toggles to Allow and confirms it. Neither key was ignored, so the guard never restarts.
   - **Fix:** after switching to Allow, ignore Enter for `INPUT_GUARD` and restart the guard if Enter arrives in that time. Also count arrow keys, Home and End as composer activity.
   - **Test:** ← then Enter 50 ms apart, both after the guard has passed, must not allow.

4. **Low (usability): the "keys work once you stop typing" hint stays stuck** (`claude_approval_view.rs:366-403`).
   - `shown_at` is set during `render`, after `schedule_active_view_frame` has already run. `next_frame_delay` returns `None` while `shown_at` is unset, so no redraw is scheduled for when the guard ends, both on first show and after each `restart()`.
   - Keys do work, because the guard is checked when the key arrives, but the hint stays stale until something else redraws.
   - **Fix:** return `Some(INPUT_GUARD)` from `next_frame_delay` while `shown_at` is `None`.

5. **Low: wrapped lines of long values aren't indented** (`claude_panes/approval.rs:349-390`).
   - The `Block` indent applies only to each new line in a value, not to rows that wrap. A long one-line value can still wrap so its next row looks like `name: …`. The bold name is the only clue.
   - **Fix:** pre-wrap the text with an indent on continuation rows (Codex's word-wrap options support `subsequent_indent`).

**Test gaps (not blocking):**
- There's no test for the uncover paths in finding 1.
- No regression case covers a contained prompt starting with `!` or `#`. Automated sends from other panes can reach these panes too, not just the person.