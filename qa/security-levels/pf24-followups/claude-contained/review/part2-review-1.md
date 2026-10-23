**Verdict: request changes.** The protocol plumbing fails closed in the places I traced, and the refusal is lifted only when containment is actually in force. But the approval popup has two consent problems: a stray Enter can allow a tool, and the person often can't see what they are allowing. Part 2's main claim, "a person approves each tool", doesn't hold until those are fixed.

I read the code only; I didn't run anything. I didn't check every keymap binding or the exact rendering of control characters.

### Findings

1. **High: Enter allows by default.** `approval.rs:147` puts "Allow once" first and sets no `initial_selected_idx`, so the list starts on it (`list_selection_view.rs:556`). The popup appears without warning (`event_dispatch.rs:5922`). Three ways to allow by accident:
   - Enter pressed to send a message in the composer just as the popup opens.
   - A key-repeat Enter.
   - A double Enter on stacked popups, which approves the next one unseen.

   Turning off digit shortcuts doesn't help. **Fix:** start on Deny (`initial_selected_idx: Some(1)`) or make "Allow" a separate explicit key, ignore input for about 300 ms after the popup opens, and add a test that Enter on a fresh popup denies.

2. **High: the person can't see what they approve.**
   - The summary is one `Line` with `desired_height` 1 (`renderable.rs:111`, used at `list_selection_view.rs:378`). Anything past the popup width is cut off, on top of the 400-character cap (`approval.rs:126`).
   - Newlines and control characters are dropped or joined, so `ls\nrm -rf .` reads as one innocent string.
   - Padding spaces, zero-width and bidi characters can hide the end of a command.
   - Write, Edit and MultiEdit show only the path, not the content (`approval.rs:117`). Bash hides `run_in_background`, `timeout` and `dangerouslyDisableSandbox`.

   **Fix:**
   - Show the summary as a wrapping `Paragraph`.
   - Escape non-printing, format and bidi characters so they are visible (`\n` shown as `⏎`, `escape_debug`).
   - Show the full command up to a limit, with a clear "N more characters" note (deny by default when cut).
   - Show the content or diff for file edits, the extra Bash fields, and pretty-printed JSON for other tools.

3. **Medium: other places a pane could gain tools without approval are untested.** The README checked only `.claude/settings.json` hooks (Claude Code 2.1.292). These are not covered:
   - Project and user subagents (`.claude/agents/*.md`, `$CLAUDE_CONFIG_DIR/agents`). Their front matter can set `permissionMode: acceptEdits/auto/dontAsk`, `hooks` and `mcpServers`. Claude Code ignores a subagent's `bypassPermissions` only in v2.1.267 or later.
   - Skills and slash commands with `allowed-tools`.
   - `.claude.json` in the writable `CLAUDE_CONFIG_DIR`.

   A cloned repo is untrusted, so if any of these load despite `--setting-sources ""`, tools run with **zero** approvals. **Fix:** add a real-binary regression with a pane folder and config folder seeded with each of these. Add `"disableAllHooks": true` to Corbanu's `--settings` file. Consider `--disallowedTools Task` (or an empty `--agents` override). Check `claude --version` and refuse a contained launch below the version that was verified.

4. **Medium/Low: the "asking you before each tool" wording overclaims** (`aggressive.rs:55`, `level.rs` message). In `default` mode, Claude Code runs read-only tools and Bash commands it considers read-only without asking. Its read-only classifier has had bypasses before. The sandbox limits the damage, but the claim is wrong. **Fix:** reword, or add `permissions.ask` rules (Bash, Read, Glob, Grep and so on) to Corbanu's settings and verify they force a prompt. Also name `secretless_agent_launch` in that `/security` row.

5. **Low: stale popups and leaked waits.**
   - On interrupt or early exit, popups stay up, and Claude Code's `control_cancel_request` is ignored.
   - Each waiting task holds a `ClaudeStdin` sender, so the writer task and `ChildStdin` live until someone answers (`execution.rs:587`).
   - There is no turn timeout (`timeout_ms: None`), so an unanswered or ID-less request hangs the turn forever.

   **Fix:** race `decision` against the turn's cancellation token, send an event to close that turn's popups when it ends, handle `control_cancel_request`, and end the turn on a `control_request` it can't parse.

6. **Low: request IDs are not tracked.** Duplicate or unknown `request_id`s each open a popup. On Linux, an already-approved sandboxed process can write to Claude Code's stdout (`/proc/<pid>/fd/1`). It could then fake a request using a pending ID with a different `tool_name`, for example a "Read" summary whose answer allows a Bash request. **Fix:** keep a set of pending IDs, drop duplicates, show `tool_use_id`, and treat a second `result` as an error.

7. **Low: two panes can look identical.** Panes with the same profile share a title (`approval.rs:141`), so the person can't tell which pane is asking. **Fix:** add the pane's short ID or folder.

8. **Low (docs): the README is wrong about smoke commands.** It says "smoke commands deny every request". In fact `containment::install` is only called from `security/launch.rs:172`. The smoke and workflow CLI (`cli/src/main.rs:2528`) never turns containment on, so it runs `bypassPermissions` without containment under Permissive. **Fix:** correct the README, or install containment in the CLI path.

9. **Info:** stdout lines are redacted before they are parsed (`execution.rs:272`). So the input sent back as `updatedInput` is the redacted version — consistent with what the popup shows, but it can differ from what the model asked for. Worth a comment.

### Your five questions

- **Q1 (unapproved tools):** Other control-request types get an error reply. Malformed requests fail closed, except a missing `request_id`, which hangs (#5). Permission suggestions and `updatedPermissions` are never sent back, and the permission mode is never changed. Project settings, hooks and MCP servers are off via `--setting-sources ""` and `--strict-mcp-config`. Subagents, skills and commands are unverified (#3). Read-only auto-allow is #4.
- **Q2 (decision path):** Esc and Ctrl-C deny. Number shortcuts are off. If the popup is dropped, the dropped sender also denies. Without a UI, the answer is deny. A responder answers only once. The problems are the Enter default (#1) and stale popups (#5).
- **Q3 (lifting the refusal):** Correct. `contained_launch_ready` (`containment.rs:96`) uses the same `enabled()` switch as `build_claude_command_plan`. If the state folder or the sandbox wrapping fails, the turn errors instead of running uncontained. Windows keeps the refusal. Smoke uses the level check per home folder, so it is still refused under protected levels. All TUI entry points go through `run_claude_command_plan`. I found no path that launches uncontained under Aggressive.
- **Q4 (stdin/stdout):** A single writer task means no deadlock on large prompts, and closing stdin on `result` works. Gaps are in #5 and #6; redaction is #2 and #9.
- **Q5 (tests):** Gaps listed below.

### Missing tests (Q5)

- The `level.rs:96` test override returns before `contained_launch_ready` is checked, so lifting the refusal is never tested. Add cases for feature off, contract not armed, and both on.
- No popup-level tests: Enter on a fresh popup, a dropped view leading to deny, a long multi-line command being fully visible.
- The approvals fixture includes a bridge secret, but no test checks that the summary hides it.
- No tests for duplicate or missing request IDs, an interrupt with a popup still open, Claude Code exiting before it reads the prompt, or a prompt larger than 64 KB.
- No real-binary regression for #3.