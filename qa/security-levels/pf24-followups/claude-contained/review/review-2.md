**Verdict: approve with fixes.** Findings 1, 3 and 4 from my last review are fixed correctly, and finding 2 is fixed for `CODEX_HOME/panes`. But the same leak still exists through a path this PR adds. I only read the code; I didn't build or run anything.

**Fixes I checked**
- **Finding 1, bridge (`bridge.rs:722-752`, `:757-768`): fixed.**
  - A target that doesn't start with `/` is now refused.
  - The path must be exactly `/v1/messages` or `/v1/messages/count_tokens`, so `..`, `@`, `#` and `\` can't get into it.
  - The query may only contain letters, digits and `=&_.-`.
  - For absolute-form targets, the host is thrown away and only the path is kept, so the bridge isn't an open proxy.
  - The scheme, host, port and userinfo check is redundant now that the path must match exactly, but it does no harm.
  - Base URLs that carry a path (Z.AI's `/api/anthropic`) still work.
  - `?beta=true` still passes, and the accounting check still counts only `/v1/messages` requests.
  - The Ambient bridge builds its upstream URL itself and never uses the request target, so it's unaffected.
- **Finding 2, `CODEX_HOME/panes` (`containment.rs:186-201`): fixed.**
  - The new deny entry and the read-only `settings.json` entry are kept after the launch contract adds its own rules (`protect_permissions` only adds entries).
  - The more specific entry wins, so the settings file can be read but not written.
  - `--settings` now points to the copy in the state folder (`command_plan.rs:226-243`, and the test checks this).
  - The sandbox can't replace that file with a symlink, so it can't redirect the parent's next write.
- **Finding 3, order (`execution.rs:75-95`): fixed.**
  - `contain()` now runs straight after the level check, before the Claude Plan helper and before the vault is read.
  - When the plan is built, it only binds the port and writes the settings file; it reads no credentials.
  - Leaving `claude_config_dir_override` out of the contained launch is correct.
- **Finding 4, environment (`containment.rs:232-234`): fixed.** It now uses `vars_os` and drops any variable that isn't valid UTF-8.

**New findings**

1. **Medium: a contained pane can still read the Claude transcripts of other contained panes.**
   - **Where:** `containment.rs:126-152` and `:178-201`.
   - **Problem:** Each pane's `CLAUDE_CONFIG_DIR` is `<base>/<home_key>/<pane>/config`, and Claude Code writes its full session transcripts under `config/projects/`. The sandbox allows reading everywhere except the denied paths. So pane A can read pane B's state folder, which is the same leak the `panes/` denial closed, just one folder over. This PR adds those folders, so the leak is new.
   - **Fix:**
     - Add a `Deny` entry for `state_dir.parent()` (`<base>/<home_key>`). The more specific `Write` entry on the pane's own state folder keeps it usable, and the policy test at `permissions.rs:2692` shows deny-then-write nesting works.
     - Extend `base_profile_*` to check that the pane can't read `<home_key>/other-pane/config/x` but can still write its own folder.
     - Add an `ls <base>/<home_key>` probe so the nested deny is confirmed under both Seatbelt and bwrap.

2. **Low (evidence): the disposition claims a probe the recorded runs don't contain.**
   - **Where:** `tmux-run/1-macos-probes.txt` and `3-linux-probes.txt`.
   - **Problem:** The disposition says the tmux runs probe `panes/`, but the recorded macOS output shows only probes 1-5 and the Linux one only reads `config.toml`. There is no `ls {home}/panes` result and no raw request with a hostile target sent straight to the bridge.
   - **Fix:** Re-run the updated spec on the final tree for both platforms and record the result of probe 6. Add a raw `curl --request-target '@x/v1/messages'` probe that should return 404, and the sibling-folder probe from finding 1. Or correct the disposition.

3. **Low: the demo script can paste the API key into the chat and send it to the model.**
   - **Where:** `demo-launch.sh:13-18`.
   - **Problem:** The script waits for the vault popup with fixed sleeps, and it carries on even if the 60-second startup wait never sees `model: glm`. If the `/vault credential add` popup doesn't open in time, the pasted key lands in the message box and the following Enter sends it to Z.AI as a prompt, where it is also saved in session history.
   - **Fix:**
     - Wait for the popup's label and stop if it never appears; also stop if the startup wait times out.
     - Before pressing Enter, check that the pane shows the masked field.
     - Run `unset ZAI_API_KEY` before starting `$REAL`, so the candidate doesn't inherit the key.

4. **Info: the settings file is written in a way that would follow a symlink.**
   - **Where:** `command_plan.rs:236`.
   - **Problem:** `std::fs::write` follows symlinks, and the parent process now writes into a folder the sandbox can write to. This is safe today only because of the read-only entry on `settings.json`.
   - **Fix (optional hardening):** Write to a temporary file in the same folder, then rename it over `settings.json`. Or open it with `O_NOFOLLOW`.

Findings 5-7 from the first round are recorded as described, and I accept them.